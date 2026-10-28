use crate::registry::EngineRegistry;
use lingua_core::{AsrTranscriptEvent, CaptionFormat, CaptionSegment, LanguageCode, LinguaError};
use lingua_storage::StorageService;
use std::sync::Arc;

/// Coordinates streaming ASR events and real-time translation into synchronized bilingual captions.
pub struct CaptionCoordinator {
    registry: Arc<EngineRegistry>,
    storage: Option<StorageService>,
}

impl CaptionCoordinator {
    pub fn new(registry: Arc<EngineRegistry>, storage: Option<StorageService>) -> Self {
        Self { registry, storage }
    }

    /// Processes an incoming ASR transcript event, translating it to the target language
    /// and producing a synchronized `CaptionSegment`.
    pub async fn process_transcript_event(
        &self,
        event: &AsrTranscriptEvent,
        target_lang: LanguageCode,
    ) -> Result<CaptionSegment, LinguaError> {
        // Translate the source text into target language using registry routing
        let translated_text = if event.text.trim().is_empty() {
            String::new()
        } else if event.language == target_lang {
            event.text.clone()
        } else {
            let pair = lingua_core::LanguagePair::new(event.language, target_lang)?;
            let req = crate::traits::TranslationRequest {
                pair,
                text: event.text.clone(),
                allow_cloud: false,
            };
            match self.registry.translate(req).await {
                Ok(resp) => resp.translated_text,
                Err(e) => {
                    // In partial streaming, graceful fallback to untranslated
                    if event.is_final {
                        match e {
                            LinguaError::OfflineModelMissing(_) => {
                                format!("{} [offline model required]", event.text)
                            }
                            _ => return Err(e),
                        }
                    } else {
                        event.text.clone()
                    }
                }
            }
        };

        let segment = CaptionSegment {
            id: format!("cap-{}-{}", event.session_id, event.seq_id),
            session_id: event.session_id.clone(),
            seq_id: event.seq_id,
            source_text: event.text.clone(),
            translated_text,
            source_lang: event.language,
            target_lang,
            is_final: event.is_final,
            confidence: event.confidence,
            start_time_ms: event.start_time_ms,
            end_time_ms: event.end_time_ms,
            created_at: chrono::Utc::now(),
        };

        // If this segment is final, persist it to storage
        if event.is_final {
            if let Some(storage) = &self.storage {
                let _ = storage.record_caption_segment(&segment);
            }
        }

        Ok(segment)
    }

    /// Formats a list of caption segments into WebVTT (`.vtt`) format.
    pub fn export_vtt(segments: &[CaptionSegment]) -> String {
        let mut out = String::from("WEBVTT - LinguaFlow Live Captions\n\n");
        for seg in segments {
            if !seg.is_final && segments.len() > 1 {
                continue;
            }
            let start = format_vtt_timestamp(seg.start_time_ms);
            let end_ms = if seg.end_time_ms > seg.start_time_ms {
                seg.end_time_ms
            } else {
                seg.start_time_ms + 1000
            };
            let end = format_vtt_timestamp(end_ms);
            out.push_str(&format!("{start} --> {end}\n"));
            out.push_str(&format!("{}\n", seg.source_text));
            out.push_str(&format!("{}\n\n", seg.translated_text));
        }
        out
    }

    /// Formats a list of caption segments into SubRip (`.srt`) format.
    pub fn export_srt(segments: &[CaptionSegment]) -> String {
        let mut out = String::new();
        let mut counter = 1;
        for seg in segments {
            if !seg.is_final && segments.len() > 1 {
                continue;
            }
            let start = format_srt_timestamp(seg.start_time_ms);
            let end_ms = if seg.end_time_ms > seg.start_time_ms {
                seg.end_time_ms
            } else {
                seg.start_time_ms + 1000
            };
            let end = format_srt_timestamp(end_ms);
            out.push_str(&format!("{counter}\n"));
            out.push_str(&format!("{start} --> {end}\n"));
            out.push_str(&format!("{}\n", seg.source_text));
            out.push_str(&format!("{}\n\n", seg.translated_text));
            counter += 1;
        }
        out
    }

    /// Formats a list of caption segments into standard JSON format.
    pub fn export_json(segments: &[CaptionSegment]) -> Result<String, LinguaError> {
        serde_json::to_string_pretty(segments).map_err(|e| {
            LinguaError::Internal(format!("Failed to serialize captions to JSON: {e}"))
        })
    }

    /// Exports caption segments into the requested format.
    pub fn export_as(
        &self,
        segments: &[CaptionSegment],
        format: CaptionFormat,
    ) -> Result<String, LinguaError> {
        match format {
            CaptionFormat::Vtt => Ok(Self::export_vtt(segments)),
            CaptionFormat::Srt => Ok(Self::export_srt(segments)),
            CaptionFormat::Json => Self::export_json(segments),
        }
    }
}

/// Formats milliseconds into WebVTT timestamp: `HH:MM:SS.mmm`
fn format_vtt_timestamp(ms: u64) -> String {
    let hours = ms / 3_600_000;
    let mins = (ms % 3_600_000) / 60_000;
    let secs = (ms % 60_000) / 1_000;
    let millis = ms % 1_000;
    format!("{hours:02}:{mins:02}:{secs:02}.{millis:03}")
}

/// Formats milliseconds into SRT timestamp: `HH:MM:SS,mmm`
fn format_srt_timestamp(ms: u64) -> String {
    let hours = ms / 3_600_000;
    let mins = (ms % 3_600_000) / 60_000;
    let secs = (ms % 60_000) / 1_000;
    let millis = ms % 1_000;
    format!("{hours:02}:{mins:02}:{secs:02},{millis:03}")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_vtt_and_srt_formatting() {
        assert_eq!(format_vtt_timestamp(0), "00:00:00.000");
        assert_eq!(format_vtt_timestamp(3661500), "01:01:01.500");

        assert_eq!(format_srt_timestamp(0), "00:00:00,000");
        assert_eq!(format_srt_timestamp(3661500), "01:01:01,500");

        let seg = CaptionSegment {
            id: "cap-1".into(),
            session_id: "sess-1".into(),
            seq_id: 1,
            source_text: "ನಮಸ್ಕಾರ".into(),
            translated_text: "Hello".into(),
            source_lang: LanguageCode::KnIn,
            target_lang: LanguageCode::EnUs,
            is_final: true,
            confidence: 0.95,
            start_time_ms: 500,
            end_time_ms: 1500,
            created_at: chrono::Utc::now(),
        };

        let vtt = CaptionCoordinator::export_vtt(&[seg.clone()]);
        assert!(vtt.starts_with("WEBVTT"));
        assert!(vtt.contains("00:00:00.500 --> 00:00:01.500"));
        assert!(vtt.contains("ನಮಸ್ಕಾರ"));
        assert!(vtt.contains("Hello"));

        let srt = CaptionCoordinator::export_srt(&[seg]);
        assert!(srt.contains("1\n00:00:00,500 --> 00:00:01,500"));
    }
}
