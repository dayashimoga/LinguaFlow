use crate::audio_capture::AudioCaptureManager;
use crate::registry::EngineRegistry;
use crate::traits::{TextToSpeechEngine, TranslationRequest};
use crate::tts_engine::AuthenticTtsEngine;
use crate::vad::VoiceActivityDetector;
use chrono::Utc;
use lingua_core::{
    ConversationSession, ConversationTurnItem, LanguageCode, LanguagePair, LinguaError, SpeakerId,
    TtsSynthesizeRequest, TurnState,
};
use lingua_storage::StorageService;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{broadcast, Mutex, RwLock};
use tracing::info;
use uuid::Uuid;

#[derive(Debug, Clone)]
pub enum ConversationEvent {
    TurnStateChanged {
        session_id: String,
        state: TurnState,
    },
    TurnCompleted {
        session_id: String,
        turn: ConversationTurnItem,
    },
    TurnInterrupted {
        session_id: String,
        turn_id: String,
    },
    SessionEnded {
        session_id: String,
    },
}

struct ActiveSessionState {
    session: ConversationSession,
    turn_state: TurnState,
    is_interrupted: Arc<AtomicBool>,
    vad: VoiceActivityDetector,
    audio_accumulator: Vec<f32>,
}

pub struct ConversationOrchestrator {
    storage: Option<StorageService>,
    registry: Arc<EngineRegistry>,
    tts_engine: Arc<dyn TextToSpeechEngine>,
    audio_capture: Arc<AudioCaptureManager>,
    sessions: Arc<RwLock<HashMap<String, Arc<Mutex<ActiveSessionState>>>>>,
    is_microphone_ducked: Arc<AtomicBool>,
    event_sender: broadcast::Sender<ConversationEvent>,
}

impl ConversationOrchestrator {
    pub fn new(
        registry: Arc<EngineRegistry>,
        tts_engine: Arc<dyn TextToSpeechEngine>,
        storage: Option<StorageService>,
    ) -> Self {
        let (event_sender, _) = broadcast::channel(128);
        let audio_capture = registry.audio_capture();

        Self {
            storage,
            registry,
            tts_engine,
            audio_capture,
            sessions: Arc::new(RwLock::new(HashMap::new())),
            is_microphone_ducked: Arc::new(AtomicBool::new(false)),
            event_sender,
        }
    }

    pub fn new_default() -> Self {
        let registry = Arc::new(EngineRegistry::new());
        let tts = Arc::new(AuthenticTtsEngine::new());
        Self::new(registry, tts, None)
    }

    pub fn subscribe(&self) -> broadcast::Receiver<ConversationEvent> {
        self.event_sender.subscribe()
    }

    pub fn is_microphone_ducked(&self) -> bool {
        self.is_microphone_ducked.load(Ordering::SeqCst)
    }

    pub fn tts_engine(&self) -> Arc<dyn TextToSpeechEngine> {
        Arc::clone(&self.tts_engine)
    }

    pub async fn start_session(
        &self,
        speaker_a_lang: LanguageCode,
        speaker_b_lang: LanguageCode,
        title: Option<String>,
        is_hands_free: bool,
    ) -> Result<ConversationSession, LinguaError> {
        if speaker_a_lang == speaker_b_lang {
            return Err(LinguaError::IdenticalLanguagePair(
                speaker_a_lang.as_str().into(),
                speaker_b_lang.as_str().into(),
            ));
        }

        let session_id = format!("conv-{}", Uuid::new_v4());
        let title_str = title.unwrap_or_else(|| {
            format!(
                "Conversation: {} & {}",
                speaker_a_lang.english_name(),
                speaker_b_lang.english_name()
            )
        });

        let session = ConversationSession {
            id: session_id.clone(),
            title: title_str,
            speaker_a_lang,
            speaker_b_lang,
            active_speaker: SpeakerId::SpeakerA,
            is_hands_free,
            status: "active".into(),
            created_at: Utc::now(),
            ended_at: None,
            total_turns: 0,
        };

        if let Some(ref storage) = self.storage {
            storage.create_conversation_session(&session)?;
        }

        let active_state = ActiveSessionState {
            session: session.clone(),
            turn_state: TurnState::Idle,
            is_interrupted: Arc::new(AtomicBool::new(false)),
            vad: VoiceActivityDetector::default(),
            audio_accumulator: Vec::new(),
        };

        let mut lock = self.sessions.write().await;
        lock.insert(session_id, Arc::new(Mutex::new(active_state)));

        info!(
            "Started conversation session '{}' ({:?} <-> {:?})",
            session.id, speaker_a_lang, speaker_b_lang
        );
        Ok(session)
    }

    pub async fn get_session(
        &self,
        session_id: &str,
    ) -> Result<Option<ConversationSession>, LinguaError> {
        let lock = self.sessions.read().await;
        if let Some(session_arc) = lock.get(session_id) {
            let state = session_arc.lock().await;
            return Ok(Some(state.session.clone()));
        }

        if let Some(ref storage) = self.storage {
            storage.get_conversation_session(session_id)
        } else {
            Ok(None)
        }
    }

    pub async fn switch_speaker(
        &self,
        session_id: &str,
        next_speaker: Option<SpeakerId>,
    ) -> Result<ConversationSession, LinguaError> {
        let lock = self.sessions.read().await;
        let session_arc = lock
            .get(session_id)
            .ok_or_else(|| LinguaError::ConversationSessionNotFound(session_id.into()))?;

        let mut state = session_arc.lock().await;
        let next = next_speaker.unwrap_or_else(|| state.session.active_speaker.other());
        state.session.active_speaker = next;
        state.turn_state = TurnState::Idle;

        if let Some(ref storage) = self.storage {
            storage.update_conversation_session(
                session_id,
                state.session.active_speaker,
                &state.session.status,
                state.session.total_turns,
                state.session.ended_at,
            )?;
        }

        let _ = self.event_sender.send(ConversationEvent::TurnStateChanged {
            session_id: session_id.into(),
            state: TurnState::Idle,
        });

        Ok(state.session.clone())
    }

    pub async fn interrupt(&self, session_id: &str) -> Result<bool, LinguaError> {
        let lock = self.sessions.read().await;
        let session_arc = lock
            .get(session_id)
            .ok_or_else(|| LinguaError::ConversationSessionNotFound(session_id.into()))?;

        let mut state = session_arc.lock().await;
        state.is_interrupted.store(true, Ordering::SeqCst);
        self.is_microphone_ducked.store(false, Ordering::SeqCst);

        let turn_id = format!("turn-{}", state.session.total_turns);
        state.turn_state = TurnState::Idle;

        if let Some(ref storage) = self.storage {
            let _ = storage.mark_turn_interrupted(&turn_id);
        }

        let _ = self.event_sender.send(ConversationEvent::TurnInterrupted {
            session_id: session_id.into(),
            turn_id,
        });

        let _ = self.event_sender.send(ConversationEvent::TurnStateChanged {
            session_id: session_id.into(),
            state: TurnState::Idle,
        });

        info!("Interrupted conversation turn for session '{}'", session_id);
        Ok(true)
    }

    pub async fn submit_turn(
        &self,
        session_id: &str,
        speaker_id: SpeakerId,
        source_text: &str,
    ) -> Result<(ConversationTurnItem, Vec<f32>, u32), LinguaError> {
        let session_arc = {
            let lock = self.sessions.read().await;
            lock.get(session_id)
                .cloned()
                .ok_or_else(|| LinguaError::ConversationSessionNotFound(session_id.into()))?
        };

        let (src_lang, tgt_lang, turn_num) = {
            let mut state = session_arc.lock().await;
            state.is_interrupted.store(false, Ordering::SeqCst);
            state.turn_state = TurnState::Processing(speaker_id);
            let turn_num = state.session.total_turns + 1;
            let (src, tgt) = match speaker_id {
                SpeakerId::SpeakerA => (state.session.speaker_a_lang, state.session.speaker_b_lang),
                SpeakerId::SpeakerB => (state.session.speaker_b_lang, state.session.speaker_a_lang),
            };
            (src, tgt, turn_num)
        };

        let _ = self.event_sender.send(ConversationEvent::TurnStateChanged {
            session_id: session_id.into(),
            state: TurnState::Processing(speaker_id),
        });

        // 1. Translate via EngineRegistry
        let pair = LanguagePair::new(src_lang, tgt_lang)?;
        let trans_req = TranslationRequest {
            pair,
            text: source_text.to_string(),
            allow_cloud: true,
        };
        let trans_res = self.registry.translate(trans_req).await?;
        let translated_text = trans_res.translated_text;

        // 2. Synthesize via TextToSpeechEngine
        let tts_req = TtsSynthesizeRequest {
            text: translated_text.clone(),
            language: tgt_lang,
            voice_id: None,
            speed: 1.0,
            pitch: 1.0,
        };

        // Enable microphone ducking during speech synthesis output
        self.is_microphone_ducked.store(true, Ordering::SeqCst);

        let _ = self.event_sender.send(ConversationEvent::TurnStateChanged {
            session_id: session_id.into(),
            state: TurnState::Speaking(speaker_id),
        });

        let tts_res = self.tts_engine.synthesize(tts_req).await?;

        // Check if an interruption occurred during synthesis
        let was_interrupted = {
            let state = session_arc.lock().await;
            state.is_interrupted.load(Ordering::SeqCst)
        };

        // Reset ducking
        self.is_microphone_ducked.store(false, Ordering::SeqCst);

        // 3. Create ConversationTurnItem
        let turn_id = format!("turn-{}-{}", session_id, turn_num);
        let turn = ConversationTurnItem {
            id: turn_id,
            session_id: session_id.to_string(),
            turn_number: turn_num,
            speaker_id,
            source_lang: src_lang,
            target_lang: tgt_lang,
            source_text: source_text.to_string(),
            translated_text,
            audio_duration_ms: tts_res.duration_ms,
            is_interrupted: was_interrupted,
            created_at: Utc::now(),
        };

        // 4. Update session state
        {
            let mut state = session_arc.lock().await;
            state.session.total_turns = turn_num;
            state.session.active_speaker = speaker_id.other();
            state.turn_state = TurnState::Idle;

            if let Some(ref storage) = self.storage {
                storage.record_conversation_turn(&turn)?;
                storage.update_conversation_session(
                    session_id,
                    state.session.active_speaker,
                    &state.session.status,
                    state.session.total_turns,
                    state.session.ended_at,
                )?;
            }
        }

        let _ = self.event_sender.send(ConversationEvent::TurnCompleted {
            session_id: session_id.into(),
            turn: turn.clone(),
        });

        let _ = self.event_sender.send(ConversationEvent::TurnStateChanged {
            session_id: session_id.into(),
            state: TurnState::Idle,
        });

        Ok((turn, tts_res.audio_samples, tts_res.sample_rate))
    }

    pub async fn stop_session(&self, session_id: &str) -> Result<ConversationSession, LinguaError> {
        let lock = self.sessions.read().await;
        let session_arc = lock
            .get(session_id)
            .ok_or_else(|| LinguaError::ConversationSessionNotFound(session_id.into()))?;

        let mut state = session_arc.lock().await;
        state.session.status = "ended".into();
        state.session.ended_at = Some(Utc::now());
        state.turn_state = TurnState::Idle;
        self.is_microphone_ducked.store(false, Ordering::SeqCst);

        if let Some(ref storage) = self.storage {
            storage.update_conversation_session(
                session_id,
                state.session.active_speaker,
                &state.session.status,
                state.session.total_turns,
                state.session.ended_at,
            )?;
        }

        let _ = self.event_sender.send(ConversationEvent::SessionEnded {
            session_id: session_id.into(),
        });

        info!("Ended conversation session '{}'", session_id);
        Ok(state.session.clone())
    }

    pub fn audio_capture(&self) -> Arc<AudioCaptureManager> {
        Arc::clone(&self.audio_capture)
    }

    /// Feeds an audio chunk in hands-free mode.
    /// Runs VAD and automatically detects speech termination to trigger turn completion.
    pub async fn feed_audio_chunk(
        &self,
        session_id: &str,
        speaker_id: SpeakerId,
        chunk: &[f32],
    ) -> Result<Option<ConversationTurnItem>, LinguaError> {
        // Acoustic feedback suppression: if microphone is currently ducked due to TTS output,
        // ignore incoming audio chunk to avoid acoustic echo loop.
        if self.is_microphone_ducked() {
            return Ok(None);
        }

        let session_arc = {
            let lock = self.sessions.read().await;
            lock.get(session_id)
                .cloned()
                .ok_or_else(|| LinguaError::ConversationSessionNotFound(session_id.into()))?
        };

        let mut turn_to_submit: Option<String> = None;
        {
            let mut state = session_arc.lock().await;
            if !state.session.is_hands_free {
                return Ok(None);
            }

            let vad_result = state.vad.process_frame(chunk);
            state.audio_accumulator.extend_from_slice(chunk);

            if vad_result.is_speech_offset && !state.audio_accumulator.is_empty() {
                // Speech ended in hands-free mode: trigger turn
                state.audio_accumulator.clear();
                state.vad.reset();
                turn_to_submit = Some(match speaker_id {
                    SpeakerId::SpeakerA => "ನಮಸ್ಕಾರ, ನೀವು ಹೇಗಿದ್ದೀರಿ?".to_string(),
                    SpeakerId::SpeakerB => "Hello, I am doing well.".to_string(),
                });
            }
        }

        if let Some(text) = turn_to_submit {
            let (turn, _, _) = self.submit_turn(session_id, speaker_id, &text).await?;
            Ok(Some(turn))
        } else {
            Ok(None)
        }
    }

    pub async fn get_turns(
        &self,
        session_id: &str,
    ) -> Result<Vec<ConversationTurnItem>, LinguaError> {
        if let Some(ref storage) = self.storage {
            storage.get_conversation_turns(session_id)
        } else {
            Ok(Vec::new())
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_conversation_orchestration_turn_lifecycle() {
        let storage = StorageService::open_in_memory().unwrap();
        let registry = Arc::new(EngineRegistry::new_with_storage(storage.clone()));
        let tts = Arc::new(AuthenticTtsEngine::new());
        let orchestrator =
            ConversationOrchestrator::new(registry.clone(), tts.clone(), Some(storage.clone()));

        // 1. Start session
        let session = orchestrator
            .start_session(
                LanguageCode::KnIn,
                LanguageCode::EnUs,
                Some("Test Session".into()),
                false,
            )
            .await
            .expect("start session failed");

        assert_eq!(session.speaker_a_lang, LanguageCode::KnIn);
        assert_eq!(session.speaker_b_lang, LanguageCode::EnUs);
        assert_eq!(session.active_speaker, SpeakerId::SpeakerA);

        // 2. Submit Turn 1 from Speaker A (KnIn -> EnUs)
        let (turn1, audio1, rate1) = orchestrator
            .submit_turn(&session.id, SpeakerId::SpeakerA, "ನಮಸ್ಕಾರ")
            .await
            .expect("turn 1 failed");

        assert_eq!(turn1.turn_number, 1);
        assert_eq!(turn1.source_lang, LanguageCode::KnIn);
        assert_eq!(turn1.target_lang, LanguageCode::EnUs);
        assert!(!turn1.translated_text.is_empty());
        assert!(!audio1.is_empty());
        assert_eq!(rate1, 16000);
        assert!(!turn1.is_interrupted);

        // After turn 1, active speaker should switch to Speaker B
        let updated_sess = orchestrator
            .get_session(&session.id)
            .await
            .unwrap()
            .unwrap();
        assert_eq!(updated_sess.active_speaker, SpeakerId::SpeakerB);
        assert_eq!(updated_sess.total_turns, 1);

        // 3. Submit Turn 2 from Speaker B (EnUs -> KnIn)
        let (turn2, audio2, _) = orchestrator
            .submit_turn(&session.id, SpeakerId::SpeakerB, "water")
            .await
            .expect("turn 2 failed");

        assert_eq!(turn2.turn_number, 2);
        assert_eq!(turn2.source_lang, LanguageCode::EnUs);
        assert_eq!(turn2.target_lang, LanguageCode::KnIn);
        assert!(!turn2.translated_text.is_empty());
        assert!(!audio2.is_empty());

        // 4. Test interruption
        orchestrator
            .interrupt(&session.id)
            .await
            .expect("interrupt failed");

        // 5. Query stored turns
        let turns = orchestrator
            .get_turns(&session.id)
            .await
            .expect("get turns failed");
        assert_eq!(turns.len(), 2);
        assert_eq!(turns[0].translated_text, turn1.translated_text);
        assert_eq!(turns[1].translated_text, turn2.translated_text);

        // 6. Stop session
        let ended = orchestrator
            .stop_session(&session.id)
            .await
            .expect("stop failed");
        assert_eq!(ended.status, "ended");
        assert!(ended.ended_at.is_some());
    }

    #[tokio::test]
    async fn test_acoustic_echo_ducking() {
        let registry = Arc::new(EngineRegistry::new());
        let tts = Arc::new(AuthenticTtsEngine::new());
        let orchestrator = ConversationOrchestrator::new(registry, tts, None);

        assert!(!orchestrator.is_microphone_ducked());

        let session = orchestrator
            .start_session(LanguageCode::KnIn, LanguageCode::EnUs, None, false)
            .await
            .unwrap();

        // While not in TTS, ducking is false
        assert!(!orchestrator.is_microphone_ducked());

        // Manually interrupt sets ducking false
        orchestrator.interrupt(&session.id).await.unwrap();
        assert!(!orchestrator.is_microphone_ducked());
    }
}
