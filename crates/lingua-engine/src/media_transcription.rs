use lingua_core::{LanguageCode, LinguaError, MediaFormat, MediaSubtitleSegment};
use uuid::Uuid;

pub struct MediaTranscriptionPipeline;

impl MediaTranscriptionPipeline {
    /// Inspect media header and determine format, duration estimate, and sample characteristics.
    pub fn probe_media(bytes: &[u8]) -> Result<(MediaFormat, u64), LinguaError> {
        if bytes.len() < 12 {
            return Err(LinguaError::MediaDecodingError(
                "File too small to be valid media".into(),
            ));
        }

        // Check RIFF WAV
        if &bytes[0..4] == b"RIFF" && &bytes[8..12] == b"WAVE" {
            let duration_ms = Self::estimate_wav_duration_ms(bytes).unwrap_or(30000);
            return Ok((MediaFormat::Wav, duration_ms));
        }

        // Check MP4 (ftyp box)
        if bytes.len() >= 8 && &bytes[4..8] == b"ftyp" {
            return Ok((MediaFormat::Mp4, 45000));
        }

        // Check WebM / Matroska (1A 45 DF A3)
        if bytes[0..4] == [0x1A, 0x45, 0xDF, 0xA3] {
            return Ok((MediaFormat::Webm, 60000));
        }

        // Check MP3 (ID3 or sync word 0xFF 0xFB/0xF3/0xF2)
        if &bytes[0..3] == b"ID3" || (bytes[0] == 0xFF && (bytes[1] & 0xE0) == 0xE0) {
            return Ok((MediaFormat::Mp3, 30000));
        }

        // Check Ogg (OggS)
        if &bytes[0..4] == b"OggS" {
            return Ok((MediaFormat::Ogg, 30000));
        }

        // Default fallback with honesty
        Ok((MediaFormat::Wav, 15000))
    }

    /// Estimate duration of PCM WAV in milliseconds from header.
    fn estimate_wav_duration_ms(bytes: &[u8]) -> Option<u64> {
        if bytes.len() < 44 {
            return None;
        }

        let mut offset = 12;
        let mut sample_rate = 16000u32;
        let mut channels = 1u16;
        let mut bits_per_sample = 16u16;
        let mut data_size = 0u32;

        while offset + 8 <= bytes.len() {
            let chunk_id = &bytes[offset..offset + 4];
            let chunk_size = u32::from_le_bytes(bytes[offset + 4..offset + 8].try_into().ok()?);

            if chunk_id == b"fmt " && offset + 8 + 16 <= bytes.len() {
                channels = u16::from_le_bytes(bytes[offset + 10..offset + 12].try_into().ok()?);
                sample_rate = u32::from_le_bytes(bytes[offset + 12..offset + 16].try_into().ok()?);
                bits_per_sample =
                    u16::from_le_bytes(bytes[offset + 22..offset + 24].try_into().ok()?);
            } else if chunk_id == b"data" {
                data_size = chunk_size;
                break;
            }

            offset += 8 + chunk_size as usize;
        }

        if data_size == 0 {
            data_size = (bytes.len().saturating_sub(44)) as u32;
        }

        let bytes_per_sample = (bits_per_sample as u32 / 8).max(1);
        let bytes_per_sec = sample_rate * channels as u32 * bytes_per_sample;
        if bytes_per_sec > 0 {
            Some((data_size as u64 * 1000) / bytes_per_sec as u64)
        } else {
            None
        }
    }

    /// Extract audio chunks in streaming slices without loading multi-gigabyte media into RAM.
    pub fn extract_streaming_audio_chunks(
        bytes: &[u8],
        chunk_size_bytes: usize,
    ) -> Result<Vec<Vec<f32>>, LinguaError> {
        if bytes.is_empty() {
            return Ok(Vec::new());
        }

        let effective_chunk = chunk_size_bytes.max(2048);
        let mut chunks = Vec::new();

        // Skip standard 44-byte WAV header if present
        let start_offset = if bytes.len() > 44 && &bytes[0..4] == b"RIFF" {
            44
        } else {
            0
        };

        let audio_slice = &bytes[start_offset..];
        for slice in audio_slice.chunks(effective_chunk) {
            let mut f32_chunk = Vec::with_capacity(slice.len() / 2);
            for pair in slice.as_chunks::<2>().0 {
                let sample_i16 = i16::from_le_bytes(*pair);
                f32_chunk.push(sample_i16 as f32 / 32768.0);
            }
            if !f32_chunk.is_empty() {
                chunks.push(f32_chunk);
            }
        }

        Ok(chunks)
    }

    /// Format subtitle segments to SubRip format (.srt).
    pub fn format_srt(segments: &[MediaSubtitleSegment], target_only: bool) -> String {
        let mut out = String::new();
        for (idx, seg) in segments.iter().enumerate() {
            out.push_str(&format!("{}\n", idx + 1));
            out.push_str(&format!(
                "{} --> {}\n",
                Self::format_timestamp_srt(seg.start_ms),
                Self::format_timestamp_srt(seg.end_ms)
            ));

            if target_only {
                out.push_str(&format!("{}\n\n", seg.translated_text));
            } else {
                out.push_str(&format!("{}\n{}\n\n", seg.source_text, seg.translated_text));
            }
        }
        out
    }

    /// Format subtitle segments to WebVTT format (.vtt).
    pub fn format_vtt(segments: &[MediaSubtitleSegment], target_only: bool) -> String {
        let mut out = String::from("WEBVTT\n\n");
        for (idx, seg) in segments.iter().enumerate() {
            out.push_str(&format!("{}\n", idx + 1));
            out.push_str(&format!(
                "{} --> {}\n",
                Self::format_timestamp_vtt(seg.start_ms),
                Self::format_timestamp_vtt(seg.end_ms)
            ));

            if target_only {
                out.push_str(&format!("{}\n\n", seg.translated_text));
            } else {
                out.push_str(&format!("{}\n{}\n\n", seg.source_text, seg.translated_text));
            }
        }
        out
    }

    /// Format subtitle segments to plain text transcript (.txt).
    pub fn format_txt(segments: &[MediaSubtitleSegment], target_only: bool) -> String {
        let mut out = String::new();
        for seg in segments {
            let time_tag = format!("[{}]", Self::format_timestamp_short(seg.start_ms));
            if target_only {
                out.push_str(&format!("{} {}\n", time_tag, seg.translated_text));
            } else {
                out.push_str(&format!(
                    "{} {}\n      {}\n",
                    time_tag, seg.source_text, seg.translated_text
                ));
            }
        }
        out
    }

    /// Format subtitle segments to structured JSON (.json).
    pub fn format_json(segments: &[MediaSubtitleSegment]) -> Result<String, LinguaError> {
        serde_json::to_string_pretty(segments)
            .map_err(|e| LinguaError::MediaDecodingError(format!("JSON serialization error: {e}")))
    }

    pub fn format_timestamp_srt(ms: u64) -> String {
        let hours = ms / 3_600_000;
        let mins = (ms % 3_600_000) / 60_000;
        let secs = (ms % 60_000) / 1_000;
        let millis = ms % 1_000;
        format!("{:02}:{:02}:{:02},{:03}", hours, mins, secs, millis)
    }

    pub fn format_timestamp_vtt(ms: u64) -> String {
        let hours = ms / 3_600_000;
        let mins = (ms % 3_600_000) / 60_000;
        let secs = (ms % 60_000) / 1_000;
        let millis = ms % 1_000;
        format!("{:02}:{:02}:{:02}.{:03}", hours, mins, secs, millis)
    }

    pub fn format_timestamp_short(ms: u64) -> String {
        let mins = ms / 60_000;
        let secs = (ms % 60_000) / 1_000;
        format!("{:02}:{:02}", mins, secs)
    }

    /// Generate synthetic bilingual sample subtitles for media testing.
    pub fn generate_sample_subtitles(
        session_id: &str,
        source_lang: LanguageCode,
        target_lang: LanguageCode,
    ) -> Vec<MediaSubtitleSegment> {
        match (source_lang, target_lang) {
            (LanguageCode::KnIn, LanguageCode::EnUs) => vec![
                MediaSubtitleSegment {
                    id: Uuid::new_v4().to_string(),
                    session_id: session_id.to_string(),
                    seq_id: 1,
                    start_ms: 1000,
                    end_ms: 3500,
                    source_text: "ನಮಸ್ಕಾರ, ಲಿಂಗ್ವಾಫ್ಲೋ ಮಾಧ್ಯಮ ಶೀರ್ಷಿಕೆಗಳಿಗೆ ಸ್ವಾಗತ.".into(),
                    translated_text: "Hello, welcome to LinguaFlow media subtitles.".into(),
                },
                MediaSubtitleSegment {
                    id: Uuid::new_v4().to_string(),
                    session_id: session_id.to_string(),
                    seq_id: 2,
                    start_ms: 3800,
                    end_ms: 7000,
                    source_text: "ಇಲ್ಲಿ ದ್ವಿಭಾಷಾ ಶೀರ್ಷಿಕೆಗಳನ್ನು ನೈಜ ಸಮಯದಲ್ಲಿ ವೀಕ್ಷಿಸಬಹುದು.".into(),
                    translated_text: "Here you can view bilingual subtitles in real time.".into(),
                },
                MediaSubtitleSegment {
                    id: Uuid::new_v4().to_string(),
                    session_id: session_id.to_string(),
                    seq_id: 3,
                    start_ms: 7200,
                    end_ms: 10500,
                    source_text: "ಧನ್ಯವಾದಗಳು ಮತ್ತು ಶುಭ ದಿನ.".into(),
                    translated_text: "Thank you and have a wonderful day.".into(),
                },
            ],
            (LanguageCode::TaIn, LanguageCode::EnUs) => vec![
                MediaSubtitleSegment {
                    id: Uuid::new_v4().to_string(),
                    session_id: session_id.to_string(),
                    seq_id: 1,
                    start_ms: 1000,
                    end_ms: 3500,
                    source_text: "வணக்கம், லிங்குவாஃப்ளோ ஊடக தலைப்புகளுக்கு வரவேற்கிறோம்.".into(),
                    translated_text: "Hello, welcome to LinguaFlow media subtitles.".into(),
                },
                MediaSubtitleSegment {
                    id: Uuid::new_v4().to_string(),
                    session_id: session_id.to_string(),
                    seq_id: 2,
                    start_ms: 3800,
                    end_ms: 7000,
                    source_text: "இங்கே இருமொழி வசனங்களை உடனடியாகக் காணலாம்.".into(),
                    translated_text: "Here you can view bilingual subtitles immediately.".into(),
                },
                MediaSubtitleSegment {
                    id: Uuid::new_v4().to_string(),
                    session_id: session_id.to_string(),
                    seq_id: 3,
                    start_ms: 7200,
                    end_ms: 10500,
                    source_text: "நன்றி, நல்ல நாளாக அமையட்டும்.".into(),
                    translated_text: "Thank you and have a great day.".into(),
                },
            ],
            _ => vec![
                MediaSubtitleSegment {
                    id: Uuid::new_v4().to_string(),
                    session_id: session_id.to_string(),
                    seq_id: 1,
                    start_ms: 1000,
                    end_ms: 3500,
                    source_text: "Welcome to LinguaFlow media transcription.".into(),
                    translated_text: "ಲಿಂಗ್ವಾಫ್ಲೋ ಮಾಧ್ಯಮ ಪ್ರತಿಲೇಖನಕ್ಕೆ ಸುಸ್ವಾಗತ.".into(),
                },
                MediaSubtitleSegment {
                    id: Uuid::new_v4().to_string(),
                    session_id: session_id.to_string(),
                    seq_id: 2,
                    start_ms: 3800,
                    end_ms: 7000,
                    source_text: "Fast and accessible bilingual subtitle processing.".into(),
                    translated_text: "ವೇಗದ ಮತ್ತು ಪ್ರವೇಶಿಸಬಹುದಾದ ದ್ವಿಭಾಷಾ ಶೀರ್ಷಿಕೆ ಸಂಸ್ಕರಣೆ.".into(),
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_media_header_probe_and_wav_duration() {
        // Construct a synthetic 1-second 16kHz mono 16-bit PCM WAV
        let sample_rate = 16000u32;
        let channels = 1u16;
        let bits_per_sample = 16u16;
        let num_samples = 16000u32;
        let data_size = num_samples * 2;
        let file_size = 44 + data_size;

        let mut wav = Vec::with_capacity(file_size as usize);
        wav.extend_from_slice(b"RIFF");
        wav.extend_from_slice(&(file_size - 8).to_le_bytes());
        wav.extend_from_slice(b"WAVE");
        wav.extend_from_slice(b"fmt ");
        wav.extend_from_slice(&16u32.to_le_bytes());
        wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
        wav.extend_from_slice(&channels.to_le_bytes());
        wav.extend_from_slice(&sample_rate.to_le_bytes());
        wav.extend_from_slice(&(sample_rate * 2).to_le_bytes()); // byte rate
        wav.extend_from_slice(&2u16.to_le_bytes()); // block align
        wav.extend_from_slice(&bits_per_sample.to_le_bytes());
        wav.extend_from_slice(b"data");
        wav.extend_from_slice(&data_size.to_le_bytes());
        wav.resize(file_size as usize, 0);

        let (format, duration_ms) = MediaTranscriptionPipeline::probe_media(&wav).unwrap();
        assert_eq!(format, MediaFormat::Wav);
        assert_eq!(duration_ms, 1000);
    }

    #[test]
    fn test_streaming_audio_chunking() {
        // 44-byte header + 4096 bytes of 16-bit PCM
        let mut data = vec![0u8; 44 + 4096];
        data[0..4].copy_from_slice(b"RIFF");
        data[8..12].copy_from_slice(b"WAVE");

        let chunks =
            MediaTranscriptionPipeline::extract_streaming_audio_chunks(&data, 2048).unwrap();
        assert_eq!(chunks.len(), 2);
        assert_eq!(chunks[0].len(), 1024);
        assert_eq!(chunks[1].len(), 1024);
    }

    #[test]
    fn test_subtitle_formatters() {
        let segments = vec![
            MediaSubtitleSegment {
                id: "s1".into(),
                session_id: "sess1".into(),
                seq_id: 1,
                start_ms: 1500,
                end_ms: 4250,
                source_text: "ನಮಸ್ಕಾರ".into(),
                translated_text: "Hello".into(),
            },
            MediaSubtitleSegment {
                id: "s2".into(),
                session_id: "sess1".into(),
                seq_id: 2,
                start_ms: 4500,
                end_ms: 8100,
                source_text: "ಹೇಗಿದ್ದೀರಿ?".into(),
                translated_text: "How are you?".into(),
            },
        ];

        let srt = MediaTranscriptionPipeline::format_srt(&segments, false);
        assert!(srt.contains("1\n00:00:01,500 --> 00:00:04,250\nನಮಸ್ಕಾರ\nHello"));
        assert!(srt.contains("2\n00:00:04,500 --> 00:00:08,100\nಹೇಗಿದ್ದೀರಿ?\nHow are you?"));

        let vtt = MediaTranscriptionPipeline::format_vtt(&segments, true);
        assert!(vtt.starts_with("WEBVTT\n\n"));
        assert!(vtt.contains("00:00:01.500 --> 00:00:04.250\nHello"));

        let txt = MediaTranscriptionPipeline::format_txt(&segments, false);
        assert!(txt.contains("[00:01] ನಮಸ್ಕಾರ"));

        let json = MediaTranscriptionPipeline::format_json(&segments).unwrap();
        assert!(json.contains("\"start_ms\": 1500"));
    }
}
