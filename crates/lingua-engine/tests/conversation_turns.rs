use lingua_core::{LanguageCode, SpeakerId};
use lingua_engine::{AuthenticTtsEngine, ConversationOrchestrator, EngineRegistry};
use lingua_storage::StorageService;
use std::sync::Arc;

#[tokio::test]
async fn test_two_way_kannada_english_conversation() {
    let storage = StorageService::open_in_memory().expect("in-memory db failed");
    let registry = Arc::new(EngineRegistry::new_with_storage(storage.clone()));
    let tts = Arc::new(AuthenticTtsEngine::new());
    let orchestrator =
        ConversationOrchestrator::new(registry.clone(), tts.clone(), Some(storage.clone()));

    let session = orchestrator
        .start_session(
            LanguageCode::KnIn,
            LanguageCode::EnUs,
            Some("Doctor-Patient Consultation".into()),
            false,
        )
        .await
        .expect("start session failed");

    // Turn 1: Speaker A (Kannada: "ನನಗೆ ಜ್ವರ ಬಂದಿದೆ")
    let (turn1, audio1, rate1) = orchestrator
        .submit_turn(&session.id, SpeakerId::SpeakerA, "ನನಗೆ ಜ್ವರ ಬಂದಿದೆ")
        .await
        .expect("turn 1 failed");

    assert_eq!(turn1.turn_number, 1);
    assert_eq!(turn1.source_lang, LanguageCode::KnIn);
    assert_eq!(turn1.target_lang, LanguageCode::EnUs);
    assert!(!turn1.translated_text.is_empty());
    assert!(!audio1.is_empty());
    assert_eq!(rate1, 16000);
    assert!(!turn1.is_interrupted);

    // Verify acoustic waveform properties
    let energy1: f32 = audio1.iter().map(|s| s * s).sum();
    assert!(energy1 > 0.5, "Audio must contain acoustic energy");

    // Check active speaker transitioned to Speaker B
    let sess_after_t1 = orchestrator
        .get_session(&session.id)
        .await
        .unwrap()
        .unwrap();
    assert_eq!(sess_after_t1.active_speaker, SpeakerId::SpeakerB);
    assert_eq!(sess_after_t1.total_turns, 1);

    // Turn 2: Speaker B (English: "Take this medicine twice a day.")
    let (turn2, audio2, rate2) = orchestrator
        .submit_turn(
            &session.id,
            SpeakerId::SpeakerB,
            "Take this medicine twice a day.",
        )
        .await
        .expect("turn 2 failed");

    assert_eq!(turn2.turn_number, 2);
    assert_eq!(turn2.source_lang, LanguageCode::EnUs);
    assert_eq!(turn2.target_lang, LanguageCode::KnIn);
    assert!(!turn2.translated_text.is_empty());
    assert!(!audio2.is_empty());
    assert_eq!(rate2, 16000);

    // Verify stored turns
    let turns = orchestrator.get_turns(&session.id).await.unwrap();
    assert_eq!(turns.len(), 2);
    assert_eq!(turns[0].speaker_id, SpeakerId::SpeakerA);
    assert_eq!(turns[1].speaker_id, SpeakerId::SpeakerB);
}

#[tokio::test]
async fn test_two_way_tamil_english_conversation() {
    let storage = StorageService::open_in_memory().expect("in-memory db failed");
    let registry = Arc::new(EngineRegistry::new_with_storage(storage.clone()));
    let tts = Arc::new(AuthenticTtsEngine::new());
    let orchestrator =
        ConversationOrchestrator::new(registry.clone(), tts.clone(), Some(storage.clone()));

    let session = orchestrator
        .start_session(
            LanguageCode::TaIn,
            LanguageCode::EnUs,
            Some("Travel Inquiry".into()),
            false,
        )
        .await
        .expect("start session failed");

    // Turn 1: Speaker A (Tamil: "வணக்கம்")
    let (turn1, audio1, rate1) = orchestrator
        .submit_turn(&session.id, SpeakerId::SpeakerA, "வணக்கம்")
        .await
        .expect("turn 1 failed");

    assert_eq!(turn1.turn_number, 1);
    assert_eq!(turn1.source_lang, LanguageCode::TaIn);
    assert_eq!(turn1.target_lang, LanguageCode::EnUs);
    assert!(!turn1.translated_text.is_empty());
    assert!(!audio1.is_empty());
    assert_eq!(rate1, 16000);

    // Turn 2: Speaker B (English: "water")
    let (turn2, audio2, rate2) = orchestrator
        .submit_turn(&session.id, SpeakerId::SpeakerB, "water")
        .await
        .expect("turn 2 failed");

    assert_eq!(turn2.turn_number, 2);
    assert_eq!(turn2.source_lang, LanguageCode::EnUs);
    assert_eq!(turn2.target_lang, LanguageCode::TaIn);
    assert!(!turn2.translated_text.is_empty());
    assert!(!audio2.is_empty());
    assert_eq!(rate2, 16000);
}

#[tokio::test]
async fn test_two_way_kannada_tamil_direct_conversation() {
    let storage = StorageService::open_in_memory().expect("in-memory db failed");
    let registry = Arc::new(EngineRegistry::new_with_storage(storage.clone()));
    let tts = Arc::new(AuthenticTtsEngine::new());
    let orchestrator =
        ConversationOrchestrator::new(registry.clone(), tts.clone(), Some(storage.clone()));

    let session = orchestrator
        .start_session(
            LanguageCode::KnIn,
            LanguageCode::TaIn,
            Some("Direct Indic Conversation".into()),
            false,
        )
        .await
        .expect("start session failed");

    // Turn 1: Speaker A (Kannada: "ನೀರು")
    let (turn1, audio1, _) = orchestrator
        .submit_turn(&session.id, SpeakerId::SpeakerA, "ನೀರು")
        .await
        .expect("turn 1 failed");

    assert_eq!(turn1.source_lang, LanguageCode::KnIn);
    assert_eq!(turn1.target_lang, LanguageCode::TaIn);
    assert!(!turn1.translated_text.is_empty());
    assert!(!audio1.is_empty());

    // Turn 2: Speaker B (Tamil: "தண்ணீர்")
    let (turn2, audio2, _) = orchestrator
        .submit_turn(&session.id, SpeakerId::SpeakerB, "தண்ணீர்")
        .await
        .expect("turn 2 failed");

    assert_eq!(turn2.source_lang, LanguageCode::TaIn);
    assert_eq!(turn2.target_lang, LanguageCode::KnIn);
    assert!(!turn2.translated_text.is_empty());
    assert!(!audio2.is_empty());
}

#[tokio::test]
async fn test_rapid_speaker_switching() {
    let storage = StorageService::open_in_memory().expect("in-memory db failed");
    let registry = Arc::new(EngineRegistry::new_with_storage(storage.clone()));
    let tts = Arc::new(AuthenticTtsEngine::new());
    let orchestrator =
        ConversationOrchestrator::new(registry.clone(), tts.clone(), Some(storage.clone()));

    let session = orchestrator
        .start_session(LanguageCode::KnIn, LanguageCode::EnUs, None, false)
        .await
        .unwrap();

    let mut current_speaker = SpeakerId::SpeakerA;
    for i in 1..=5 {
        let text = if current_speaker == SpeakerId::SpeakerA {
            "ನಮಸ್ಕಾರ"
        } else {
            "hello"
        };

        let (turn, _, _) = orchestrator
            .submit_turn(&session.id, current_speaker, text)
            .await
            .unwrap();

        assert_eq!(turn.turn_number, i as u32);
        assert_eq!(turn.speaker_id, current_speaker);

        let s = orchestrator
            .get_session(&session.id)
            .await
            .unwrap()
            .unwrap();
        current_speaker = s.active_speaker;
    }

    let turns = orchestrator.get_turns(&session.id).await.unwrap();
    assert_eq!(turns.len(), 5);
}

#[tokio::test]
async fn test_barge_in_and_interruption_behavior() {
    let storage = StorageService::open_in_memory().expect("in-memory db failed");
    let registry = Arc::new(EngineRegistry::new_with_storage(storage.clone()));
    let tts = Arc::new(AuthenticTtsEngine::new());
    let orchestrator =
        ConversationOrchestrator::new(registry.clone(), tts.clone(), Some(storage.clone()));

    let session = orchestrator
        .start_session(LanguageCode::KnIn, LanguageCode::EnUs, None, false)
        .await
        .unwrap();

    // Submit Turn 1
    let (turn1, _, _) = orchestrator
        .submit_turn(&session.id, SpeakerId::SpeakerA, "ನಮಸ್ಕಾರ")
        .await
        .unwrap();
    assert!(!turn1.is_interrupted);

    // Call interrupt
    let res = orchestrator.interrupt(&session.id).await.unwrap();
    assert!(res);

    // Microphone ducking must be reset
    assert!(!orchestrator.is_microphone_ducked());

    // Switch speaker explicitly
    let updated = orchestrator
        .switch_speaker(&session.id, Some(SpeakerId::SpeakerA))
        .await
        .unwrap();
    assert_eq!(updated.active_speaker, SpeakerId::SpeakerA);
}

#[tokio::test]
async fn test_acoustic_echo_suppression_mic_ducking() {
    let storage = StorageService::open_in_memory().expect("in-memory db failed");
    let registry = Arc::new(EngineRegistry::new_with_storage(storage.clone()));
    let tts = Arc::new(AuthenticTtsEngine::new());
    let orchestrator =
        ConversationOrchestrator::new(registry.clone(), tts.clone(), Some(storage.clone()));

    let session = orchestrator
        .start_session(LanguageCode::KnIn, LanguageCode::EnUs, None, true)
        .await
        .unwrap();

    // In idle state, ducking is false
    assert!(!orchestrator.is_microphone_ducked());

    // Feeding audio when ducked is ignored to prevent feedback loop
    let dummy_pcm = vec![0.1f32; 512];
    let res = orchestrator
        .feed_audio_chunk(&session.id, SpeakerId::SpeakerA, &dummy_pcm)
        .await
        .unwrap();
    assert!(res.is_none());
}
