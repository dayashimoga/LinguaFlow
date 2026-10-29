use chrono::Utc;
use lingua_core::{LanguageCode, ModelInstallState, TtsSynthesizeRequest};
use lingua_engine::audio_buffer::AudioRingBuffer;
use lingua_engine::audio_capture::AudioCaptureManager;
use lingua_engine::streaming_asr::{SherpaStreamingAsrEngine, StreamingAsrEngine};
use lingua_engine::traits::TextToSpeechEngine;
use lingua_engine::tts_engine::AuthenticTtsEngine;
use lingua_engine::vad::{
    compute_pitch_periodicity, AcousticClassification, VoiceActivityDetector,
};
use lingua_storage::StorageService;

#[test]
fn test_audio_device_enumeration() {
    // 1. Enumerate input microphones
    let input_devices = AudioCaptureManager::list_input_devices().expect("list inputs failed");
    assert!(
        !input_devices.is_empty(),
        "Must report at least one input audio device"
    );

    // 2. Enumerate output speakers
    let output_devices = AudioCaptureManager::list_output_devices().expect("list outputs failed");
    assert!(
        !output_devices.is_empty(),
        "Must report at least one output audio device"
    );

    for dev in &output_devices {
        assert!(!dev.id.is_empty());
        assert!(!dev.name.is_empty());
    }
}

#[test]
fn test_silero_hybrid_vad_discrimination() {
    let vad = VoiceActivityDetector::default();

    // 1. Pure silence: energy < -55 dB -> Silence
    let silence = vec![0.0001f32; 512];
    let (prob_silence, class_silence) = vad.classify_frame(&silence);
    assert_eq!(class_silence, AcousticClassification::Silence);
    assert!(prob_silence < 0.20);

    // 2. Human Voiced Speech: 160 Hz fundamental tone -> VoicedSpeech
    let voiced_speech: Vec<f32> = (0..512)
        .map(|i| 0.45 * (2.0 * std::f32::consts::PI * 160.0 * (i as f32 / 16000.0)).sin())
        .collect();
    let (prob_speech, class_speech) = vad.classify_frame(&voiced_speech);
    assert_eq!(class_speech, AcousticClassification::VoicedSpeech);
    assert!(prob_speech >= 0.70);

    let (periodicity, _) = compute_pitch_periodicity(&voiced_speech);
    assert!(periodicity > 0.60);

    // 3. Mechanical Click: impulse noise burst with high ZCR (> 0.40) and no pitch periodicity
    let mut click = vec![0.0f32; 512];
    let mut seed: u64 = 123456789;
    for i in 0..512 {
        seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
        let val = ((seed >> 33) as f32 / (1u64 << 31) as f32) - 0.5;
        click[i] = val * 0.7;
    }
    let (prob_click, class_click) = vad.classify_frame(&click);
    assert_eq!(class_click, AcousticClassification::MechanicalNoise);
    assert!(prob_click < 0.35);
}

#[tokio::test]
async fn test_asr_capability_gating_and_streaming() {
    let storage = StorageService::open_in_memory().expect("in-memory db failed");
    let asr_engine = SherpaStreamingAsrEngine::new_with_storage(Some(storage.clone()));

    // 1. Honest capability gating: uninstalled model MUST return OfflineModelMissing
    let gate_res = asr_engine.create_stream("test-uninstalled", LanguageCode::KnIn);
    assert!(gate_res.is_err());
    match gate_res.err().unwrap() {
        lingua_core::LinguaError::OfflineModelMissing(msg) => {
            assert!(msg.contains("asr-kn-streaming-sherpa"));
        }
        other => panic!("Expected OfflineModelMissing, got {:?}", other),
    }

    // 2. Install speech pack in storage catalog
    storage
        .update_model_pack_install_state(
            "asr-kn-streaming-sherpa",
            &ModelInstallState::Installed {
                installed_at: Utc::now(),
                install_path: "/data/models/asr-kn".into(),
            },
            true,
        )
        .unwrap();

    // 3. Stream creation succeeds once installed
    let mut stream = asr_engine
        .create_stream("test-installed", LanguageCode::KnIn)
        .expect("stream creation failed after model install");

    // Feed speech samples (16,000 samples = 1 second)
    let speech: Vec<f32> = (0..16000)
        .map(|i| 0.40 * (2.0 * std::f32::consts::PI * 150.0 * (i as f32 / 16000.0)).sin())
        .collect();
    let _ = stream.feed_audio(&speech).expect("feed failed");

    // Feed silence hangover to trigger silence boundary / finalization (8,000 samples = 500 ms)
    let silence = vec![0.0001f32; 8000];
    let final_event = stream.feed_audio(&silence).expect("feed failed");
    assert!(final_event.is_some());
    let event = final_event.unwrap();
    assert!(event.is_final);
    assert_eq!(event.language, LanguageCode::KnIn);
    assert!(!event.text.is_empty());
    assert!(event.confidence >= 0.90);
}

#[tokio::test]
async fn test_neural_tts_synthesis_multilingual() {
    let tts = AuthenticTtsEngine::new();

    // 1. Kannada Speech Synthesis
    let kn_resp = tts
        .synthesize(TtsSynthesizeRequest {
            text: "ನಮಸ್ಕಾರ, ನೀವು ಹೇಗಿದ್ದೀರಿ?".into(),
            language: LanguageCode::KnIn,
            voice_id: None,
            speed: 1.0,
            pitch: 1.0,
        })
        .await
        .expect("Kannada synthesis failed");
    assert_eq!(kn_resp.language, LanguageCode::KnIn);
    assert!(kn_resp.duration_ms > 200);
    assert!(!kn_resp.audio_samples.is_empty());

    // 2. Tamil Speech Synthesis
    let ta_resp = tts
        .synthesize(TtsSynthesizeRequest {
            text: "வணக்கம், நீங்கள் எப்படி இருக்கிறீர்கள்?".into(),
            language: LanguageCode::TaIn,
            voice_id: None,
            speed: 1.0,
            pitch: 1.0,
        })
        .await
        .expect("Tamil synthesis failed");
    assert_eq!(ta_resp.language, LanguageCode::TaIn);
    assert!(ta_resp.duration_ms > 200);
    assert!(!ta_resp.audio_samples.is_empty());

    // 3. English Speech Synthesis with 1.25x speed
    let en_resp = tts
        .synthesize(TtsSynthesizeRequest {
            text: "Welcome to LinguaFlow live speech translation.".into(),
            language: LanguageCode::EnUs,
            voice_id: None,
            speed: 1.25,
            pitch: 1.0,
        })
        .await
        .expect("English synthesis failed");
    assert_eq!(en_resp.language, LanguageCode::EnUs);
    assert!(en_resp.duration_ms > 200);

    // Audio samples must be bounded in normalized range [-1.0, 1.0]
    for s in &en_resp.audio_samples {
        assert!(*s >= -1.0 && *s <= 1.0);
    }
}

#[test]
fn test_bounded_audio_ring_buffer_strictly_under_2mb() {
    // 480,000 samples of f32 is 1.92 MB (strictly < 2 MB)
    let max_samples = 480_000;
    let mut buffer = AudioRingBuffer::new(max_samples);

    // Soak with 1,500,000 samples
    let chunk = vec![0.35f32; 1000];
    for _ in 0..1500 {
        buffer.push(&chunk);
        assert!(buffer.len() <= max_samples);
    }

    let memory_bytes = buffer.len() * std::mem::size_of::<f32>();
    assert!(
        memory_bytes < 2_000_000,
        "Audio buffer memory must remain strictly bounded under 2 MB"
    );
}
