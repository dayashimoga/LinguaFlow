use chrono::Utc;
use lingua_core::{CaptionSegment, LanguageCode, ModelInstallState, SpeechSessionStatus};
use lingua_engine::audio_buffer::AudioRingBuffer;
use lingua_engine::audio_capture::AudioCaptureManager;
use lingua_engine::caption_coordinator::CaptionCoordinator;
use lingua_engine::registry::EngineRegistry;
use lingua_engine::speech_session_manager::SpeechSessionCoordinator;
use lingua_engine::streaming_asr::{SherpaStreamingAsrEngine, StreamingAsrEngine};
use lingua_engine::vad::VoiceActivityDetector;
use lingua_storage::StorageService;
use std::sync::Arc;

#[tokio::test]
async fn test_audio_ring_buffer_bounds_and_soak() {
    // Capacity = 5,000 samples (~312 ms at 16 kHz)
    let mut ring_buffer = AudioRingBuffer::new(5000);

    // Push 50,000 samples continuously in 500-sample chunks (10x capacity soak)
    for i in 0..100 {
        let chunk: Vec<f32> = (0..500)
            .map(|j| ((i * 500 + j) as f32 * 0.05).sin())
            .collect();
        ring_buffer.push(&chunk);
        assert!(ring_buffer.len() <= 5000);
    }

    assert_eq!(ring_buffer.len(), 5000);
    assert_eq!(ring_buffer.total_samples_written(), 50000);
    assert_eq!(ring_buffer.dropped_samples_count(), 45000);

    let drained = ring_buffer.drain_all();
    assert_eq!(drained.len(), 5000);
    assert!(ring_buffer.is_empty());
}

#[tokio::test]
async fn test_vad_accuracy_on_speech_and_silence() {
    let mut vad = VoiceActivityDetector::new(-45.0, 3, 5);

    // 1. Initial silence: 10 frames of low noise (-60 dB)
    let silence_frame = vec![0.001; 512];
    for _ in 0..10 {
        let res = vad.process_frame(&silence_frame);
        assert_eq!(res.state, lingua_core::VadState::Silence);
        assert!(!res.is_speech_onset);
    }

    // 2. Active speech: 15 frames of sinusoidal audio (-12 dB)
    let speech_frame: Vec<f32> = (0..512).map(|i| 0.35 * (i as f32 * 0.1).sin()).collect();

    let mut detected_onset = false;
    for idx in 0..15 {
        let res = vad.process_frame(&speech_frame);
        if idx >= 2 {
            assert_eq!(res.state, lingua_core::VadState::Speech);
        }
        if res.is_speech_onset {
            detected_onset = true;
        }
    }
    assert!(detected_onset, "VAD must detect speech onset");

    // 3. Trailing silence to trigger speech offset / silence boundary
    let mut detected_offset = false;
    for _ in 0..10 {
        let res = vad.process_frame(&silence_frame);
        if res.is_speech_offset {
            detected_offset = true;
            assert_eq!(res.state, lingua_core::VadState::Silence);
        }
    }
    assert!(
        detected_offset,
        "VAD must detect speech offset on silence boundary"
    );
}

#[tokio::test]
async fn test_streaming_asr_events_monotonicity() {
    let asr_engine = SherpaStreamingAsrEngine::default();
    let mut stream = asr_engine
        .create_stream("session-monotonicity-test", LanguageCode::KnIn)
        .expect("failed to create stream");

    // Feed speech audio burst
    let speech: Vec<f32> = (0..16000).map(|i| 0.4 * (i as f32 * 0.08).sin()).collect();
    let _ = stream.feed_audio(&speech).expect("feed failed");

    // Feed silence to trigger final transcript event
    let silence = vec![0.001; 8000];
    let final_event = stream.feed_audio(&silence).expect("feed silence failed");

    assert!(final_event.is_some());
    let ev = final_event.unwrap();
    assert!(ev.is_final);
    assert_eq!(ev.seq_id, 1);
    assert_eq!(ev.language, LanguageCode::KnIn);
    assert!(!ev.text.is_empty());
}

#[tokio::test]
async fn test_bilingual_continuous_speech_captions_end_to_end() {
    let storage = StorageService::open_in_memory().expect("open DB failed");
    let registry = Arc::new(EngineRegistry::new_with_storage(storage.clone()));
    let audio_capture = Arc::new(AudioCaptureManager::new(480000));

    // Install required speech model for kn-IN ASR
    storage
        .update_model_pack_install_state(
            "asr-kn-streaming-sherpa",
            &ModelInstallState::Installed {
                installed_at: Utc::now(),
                install_path: "/test/models/asr-kn".into(),
            },
            true,
        )
        .unwrap();

    let coordinator = SpeechSessionCoordinator::new(
        Arc::clone(&registry),
        Some(storage.clone()),
        Arc::clone(&audio_capture),
    );

    let mut caption_rx = coordinator.subscribe_captions();

    // 1. Start a Kannada -> English speech caption session
    let session = coordinator
        .start_session(
            LanguageCode::KnIn,
            LanguageCode::EnUs,
            Some("Kannada Live Stream".into()),
            None,
        )
        .await
        .expect("start session failed");

    assert_eq!(session.source_lang, LanguageCode::KnIn);
    assert_eq!(session.target_lang, LanguageCode::EnUs);
    assert_eq!(session.status, SpeechSessionStatus::Active);

    // 2. Feed speech burst: 16,000 samples (1 sec)
    let speech: Vec<f32> = (0..16000).map(|i| 0.45 * (i as f32 * 0.1).sin()).collect();
    let _ = coordinator
        .feed_audio_chunk(&session.id, &speech)
        .await
        .expect("feed speech failed");

    // 3. Feed silence boundary: 8,000 samples (0.5 sec) to trigger final transcript
    let silence = vec![0.001; 8000];
    let produced = coordinator
        .feed_audio_chunk(&session.id, &silence)
        .await
        .expect("feed silence failed");

    assert!(
        !produced.is_empty(),
        "Must produce caption segment upon speech boundary"
    );
    let seg = &produced[0];
    assert_eq!(seg.session_id, session.id);
    assert!(seg.is_final);
    assert_eq!(seg.source_lang, LanguageCode::KnIn);
    assert_eq!(seg.target_lang, LanguageCode::EnUs);
    assert!(!seg.source_text.is_empty());
    assert!(!seg.translated_text.is_empty());

    // 4. Verify broadcast event received
    let broadcast_event = caption_rx.try_recv().expect("broadcast event missing");
    assert_eq!(broadcast_event.seq_id, seg.seq_id);
    assert_eq!(broadcast_event.source_text, seg.source_text);

    // 5. Query stored captions
    let stored_captions = coordinator
        .get_session_captions(&session.id)
        .await
        .expect("fetch captions failed");
    assert_eq!(stored_captions.len(), 1);
    assert_eq!(stored_captions[0].source_text, seg.source_text);
    assert_eq!(stored_captions[0].translated_text, seg.translated_text);

    // 6. Stop session
    let stopped = coordinator
        .stop_session(&session.id)
        .await
        .expect("stop session failed");
    assert_eq!(stopped.status, SpeechSessionStatus::Completed);
    assert!(stopped.ended_at.is_some());
}

#[tokio::test]
async fn test_caption_export_formats_vtt_srt_json() {
    let seg1 = CaptionSegment {
        id: "cap-1".into(),
        session_id: "sess-export-1".into(),
        seq_id: 1,
        source_text: "ನಮಸ್ಕಾರ, ನೀವು ಹೇಗಿದ್ದೀರಿ?".into(),
        translated_text: "Hello, how are you?".into(),
        source_lang: LanguageCode::KnIn,
        target_lang: LanguageCode::EnUs,
        is_final: true,
        confidence: 0.96,
        start_time_ms: 0,
        end_time_ms: 1800,
        created_at: chrono::Utc::now(),
    };

    let seg2 = CaptionSegment {
        id: "cap-2".into(),
        session_id: "sess-export-1".into(),
        seq_id: 2,
        source_text: "ಧನ್ಯವಾದಗಳು".into(),
        translated_text: "Thank you".into(),
        source_lang: LanguageCode::KnIn,
        target_lang: LanguageCode::EnUs,
        is_final: true,
        confidence: 0.98,
        start_time_ms: 2200,
        end_time_ms: 3100,
        created_at: chrono::Utc::now(),
    };

    let segments = vec![seg1, seg2];

    // 1. WebVTT Export
    let vtt = CaptionCoordinator::export_vtt(&segments);
    assert!(vtt.starts_with("WEBVTT"));
    assert!(vtt.contains("00:00:00.000 --> 00:00:01.800"));
    assert!(vtt.contains("ನಮಸ್ಕಾರ, ನೀವು ಹೇಗಿದ್ದೀರಿ?"));
    assert!(vtt.contains("Hello, how are you?"));
    assert!(vtt.contains("00:00:02.200 --> 00:00:03.100"));

    // 2. SRT Export
    let srt = CaptionCoordinator::export_srt(&segments);
    assert!(srt.starts_with("1\n00:00:00,000 --> 00:00:01,800\n"));
    assert!(srt.contains("2\n00:00:02,200 --> 00:00:03,100\n"));

    // 3. JSON Export
    let json = CaptionCoordinator::export_json(&segments).expect("json export failed");
    assert!(json.contains("sess-export-1"));
    assert!(json.contains("Hello, how are you?"));
}

#[tokio::test]
async fn test_session_lifecycle_pause_resume_stop() {
    let storage = StorageService::open_in_memory().expect("open DB failed");
    let registry = Arc::new(EngineRegistry::new_with_storage(storage.clone()));
    let audio_capture = Arc::new(AudioCaptureManager::new(480000));

    // Install required speech model for ta-IN ASR
    storage
        .update_model_pack_install_state(
            "asr-ta-streaming-sherpa",
            &ModelInstallState::Installed {
                installed_at: Utc::now(),
                install_path: "/test/models/asr-ta".into(),
            },
            true,
        )
        .unwrap();

    let coordinator = SpeechSessionCoordinator::new(
        Arc::clone(&registry),
        Some(storage.clone()),
        Arc::clone(&audio_capture),
    );

    let session = coordinator
        .start_session(
            LanguageCode::TaIn,
            LanguageCode::EnUs,
            Some("Tamil Lecture".into()),
            None,
        )
        .await
        .expect("start failed");
    assert_eq!(session.status, SpeechSessionStatus::Active);

    // Pause session
    let paused = coordinator
        .pause_session(&session.id)
        .await
        .expect("pause failed");
    assert_eq!(paused.status, SpeechSessionStatus::Paused);

    // Feeding audio while paused must return empty and not crash
    let samples = vec![0.2; 1600];
    let res = coordinator
        .feed_audio_chunk(&session.id, &samples)
        .await
        .expect("feed while paused failed");
    assert!(res.is_empty());

    // Resume session
    let resumed = coordinator
        .resume_session(&session.id)
        .await
        .expect("resume failed");
    assert_eq!(resumed.status, SpeechSessionStatus::Active);

    // Stop session
    let stopped = coordinator
        .stop_session(&session.id)
        .await
        .expect("stop failed");
    assert_eq!(stopped.status, SpeechSessionStatus::Completed);
    assert!(stopped.ended_at.is_some());
}
