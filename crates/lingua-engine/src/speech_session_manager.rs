use crate::audio_capture::AudioCaptureManager;
use crate::caption_coordinator::CaptionCoordinator;
use crate::registry::EngineRegistry;
use crate::streaming_asr::{AsrStream, SherpaStreamingAsrEngine, StreamingAsrEngine};
use chrono::Utc;
use lingua_core::{
    CaptionFormat, CaptionSegment, LanguageCode, LinguaError, SpeechSession, SpeechSessionStatus,
};
use lingua_storage::StorageService;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex};
use tracing::info;
use uuid::Uuid;

/// Coordinates multi-session speech recognition, capture streams, and caption broadcasts.
pub struct SpeechSessionCoordinator {
    storage: Option<StorageService>,
    audio_capture: Arc<AudioCaptureManager>,
    asr_engine: Arc<dyn StreamingAsrEngine>,
    caption_coordinator: Arc<CaptionCoordinator>,
    active_sessions: Arc<Mutex<HashMap<String, ActiveSessionState>>>,
    caption_broadcast: broadcast::Sender<CaptionSegment>,
}

struct ActiveSessionState {
    session: SpeechSession,
    asr_stream: Box<dyn AsrStream>,
    segments: Vec<CaptionSegment>,
    start_instant: std::time::Instant,
}

impl SpeechSessionCoordinator {
    pub fn new(
        registry: Arc<EngineRegistry>,
        storage: Option<StorageService>,
        audio_capture: Arc<AudioCaptureManager>,
    ) -> Self {
        let (caption_broadcast, _) = broadcast::channel(256);
        let caption_coordinator = Arc::new(CaptionCoordinator::new(
            Arc::clone(&registry),
            storage.clone(),
        ));
        let asr_engine = Arc::new(SherpaStreamingAsrEngine::new_with_storage(storage.clone()));

        Self {
            storage,
            audio_capture,
            asr_engine,
            caption_coordinator,
            active_sessions: Arc::new(Mutex::new(HashMap::new())),
            caption_broadcast,
        }
    }

    /// Starts a new real-time speech session.
    pub async fn start_session(
        &self,
        source_lang: LanguageCode,
        target_lang: LanguageCode,
        title: Option<String>,
        device_id: Option<String>,
    ) -> Result<SpeechSession, LinguaError> {
        let session_id = format!("speech-{}", Uuid::new_v4());
        let session_title = title.unwrap_or_else(|| {
            format!(
                "{} to {} Live Captions",
                source_lang.english_name(),
                target_lang.english_name()
            )
        });

        let asr_stream = self.asr_engine.create_stream(&session_id, source_lang)?;

        let session = SpeechSession {
            id: session_id.clone(),
            title: session_title,
            source_lang,
            target_lang,
            status: SpeechSessionStatus::Active,
            created_at: Utc::now(),
            ended_at: None,
            duration_seconds: 0,
        };

        // Persist session to database if storage is available
        if let Some(storage) = &self.storage {
            storage.create_speech_session(&session)?;
        }

        // Start native platform audio capture
        let _ = self
            .audio_capture
            .start_capture(&session_id, device_id.as_deref());

        let mut sessions = self.active_sessions.lock().await;
        sessions.insert(
            session_id,
            ActiveSessionState {
                session: session.clone(),
                asr_stream,
                segments: Vec::new(),
                start_instant: std::time::Instant::now(),
            },
        );

        info!("Speech session started: {}", session.id);
        Ok(session)
    }

    /// Pauses an active speech session.
    pub async fn pause_session(&self, session_id: &str) -> Result<SpeechSession, LinguaError> {
        let mut sessions = self.active_sessions.lock().await;
        let state = sessions
            .get_mut(session_id)
            .ok_or_else(|| LinguaError::SpeechSessionNotFound(session_id.to_string()))?;

        state.session.status = SpeechSessionStatus::Paused;
        let duration = state.start_instant.elapsed().as_secs();
        state.session.duration_seconds = duration;

        if let Some(storage) = &self.storage {
            let _ = storage.update_speech_session_status(
                session_id,
                SpeechSessionStatus::Paused,
                duration,
                None,
            );
        }

        Ok(state.session.clone())
    }

    /// Resumes a paused speech session.
    pub async fn resume_session(&self, session_id: &str) -> Result<SpeechSession, LinguaError> {
        let mut sessions = self.active_sessions.lock().await;
        let state = sessions
            .get_mut(session_id)
            .ok_or_else(|| LinguaError::SpeechSessionNotFound(session_id.to_string()))?;

        state.session.status = SpeechSessionStatus::Active;

        if let Some(storage) = &self.storage {
            let _ = storage.update_speech_session_status(
                session_id,
                SpeechSessionStatus::Active,
                state.session.duration_seconds,
                None,
            );
        }

        Ok(state.session.clone())
    }

    /// Stops an active or paused speech session, finalizing all remaining audio.
    pub async fn stop_session(&self, session_id: &str) -> Result<SpeechSession, LinguaError> {
        let mut sessions = self.active_sessions.lock().await;
        let mut state = sessions
            .remove(session_id)
            .ok_or_else(|| LinguaError::SpeechSessionNotFound(session_id.to_string()))?;

        let duration = state.start_instant.elapsed().as_secs();
        state.session.duration_seconds = duration;
        state.session.ended_at = Some(Utc::now());
        state.session.status = SpeechSessionStatus::Completed;

        // Finish any pending ASR tokens
        if let Ok(Some(event)) = state.asr_stream.finish() {
            if let Ok(segment) = self
                .caption_coordinator
                .process_transcript_event(&event, state.session.target_lang)
                .await
            {
                let _ = self.caption_broadcast.send(segment);
            }
        }

        if let Some(storage) = &self.storage {
            let _ = storage.update_speech_session_status(
                session_id,
                SpeechSessionStatus::Completed,
                duration,
                state.session.ended_at,
            );
        }

        // Stop audio capture if no active sessions remain
        if sessions.is_empty() {
            self.audio_capture.stop_capture();
        }

        info!("Speech session completed: {}", session_id);
        Ok(state.session)
    }

    /// Feeds incoming 16 kHz audio samples into an active session.
    /// Emits translated captions to subscribers when new events emerge.
    pub async fn feed_audio_chunk(
        &self,
        session_id: &str,
        samples: &[f32],
    ) -> Result<Vec<CaptionSegment>, LinguaError> {
        let mut sessions = self.active_sessions.lock().await;
        let state = sessions
            .get_mut(session_id)
            .ok_or_else(|| LinguaError::SpeechSessionNotFound(session_id.to_string()))?;

        if state.session.status == SpeechSessionStatus::Paused {
            return Ok(Vec::new());
        }

        let maybe_event = state.asr_stream.feed_audio(samples)?;
        let mut produced_segments = Vec::new();

        if let Some(event) = maybe_event {
            let target_lang = state.session.target_lang;
            drop(sessions); // Release lock during translation

            let segment = self
                .caption_coordinator
                .process_transcript_event(&event, target_lang)
                .await?;

            let _ = self.caption_broadcast.send(segment.clone());

            let mut sessions = self.active_sessions.lock().await;
            if let Some(state) = sessions.get_mut(session_id) {
                // If final, replace or append
                if let Some(pos) = state
                    .segments
                    .iter()
                    .position(|s| s.seq_id == segment.seq_id)
                {
                    state.segments[pos] = segment.clone();
                } else {
                    state.segments.push(segment.clone());
                }
            }
            produced_segments.push(segment);
        }

        Ok(produced_segments)
    }

    /// Retrieves all caption segments for a given session.
    pub async fn get_session_captions(
        &self,
        session_id: &str,
    ) -> Result<Vec<CaptionSegment>, LinguaError> {
        let sessions = self.active_sessions.lock().await;
        if let Some(state) = sessions.get(session_id) {
            return Ok(state.segments.clone());
        }
        drop(sessions);

        if let Some(storage) = &self.storage {
            storage.get_caption_segments_for_session(session_id)
        } else {
            Ok(Vec::new())
        }
    }

    /// Exports session captions into the requested subtitle or text format.
    pub async fn export_session_captions(
        &self,
        session_id: &str,
        format: CaptionFormat,
    ) -> Result<(String, String), LinguaError> {
        let segments = self.get_session_captions(session_id).await?;
        let content = self.caption_coordinator.export_as(&segments, format)?;

        let ext = match format {
            CaptionFormat::Vtt => "vtt",
            CaptionFormat::Srt => "srt",
            CaptionFormat::Json => "json",
        };
        let filename = format!("linguaflow-captions-{session_id}.{ext}");

        Ok((content, filename))
    }

    pub fn subscribe_captions(&self) -> broadcast::Receiver<CaptionSegment> {
        self.caption_broadcast.subscribe()
    }

    pub fn audio_capture(&self) -> &Arc<AudioCaptureManager> {
        &self.audio_capture
    }
}
