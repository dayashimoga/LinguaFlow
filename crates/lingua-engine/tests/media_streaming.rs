use lingua_core::{
    DocumentFormat, LanguageCode, MediaFormat, MediaSubtitleSegment, OverlaySettings,
    SystemAudioPermission,
};
use lingua_engine::document::{DocumentProcessor, MAX_DOCUMENT_SIZE_BYTES};
use lingua_engine::media_transcription::MediaTranscriptionPipeline;
use lingua_engine::ocr::{AuthenticOcrEngine, OcrEngine};
use lingua_engine::platform_integrations::{
    LiveOverlayController, PlatformIntegrationsCoordinator,
};

fn create_synthetic_wav(sample_rate: u32, num_samples: usize) -> Vec<u8> {
    let mut wav = Vec::new();
    wav.extend_from_slice(b"RIFF");
    let file_size = 36 + (num_samples * 2) as u32;
    wav.extend_from_slice(&file_size.to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // Subchunk1Size
    wav.extend_from_slice(&1u16.to_le_bytes()); // AudioFormat PCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // NumChannels = 1
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    let byte_rate = sample_rate * 2;
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes()); // BlockAlign
    wav.extend_from_slice(&16u16.to_le_bytes()); // BitsPerSample

    wav.extend_from_slice(b"data");
    let data_size = (num_samples * 2) as u32;
    wav.extend_from_slice(&data_size.to_le_bytes());
    for i in 0..num_samples {
        let sample = ((i as f32 * 0.1).sin() * 10000.0) as i16;
        wav.extend_from_slice(&sample.to_le_bytes());
    }
    wav
}

fn create_synthetic_bmp(width: u32, height: u32, draw_text: bool) -> Vec<u8> {
    let row_size = (width * 3 + 3) & !3; // 4-byte row alignment
    let image_size = row_size * height;
    let file_size = 54 + image_size;

    let mut bmp = Vec::with_capacity(file_size as usize);
    // BMP Header
    bmp.extend_from_slice(b"BM");
    bmp.extend_from_slice(&file_size.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes()); // reserved
    bmp.extend_from_slice(&54u32.to_le_bytes()); // pixel data offset

    // DIB Header (BITMAPINFOHEADER)
    bmp.extend_from_slice(&40u32.to_le_bytes());
    bmp.extend_from_slice(&(width as i32).to_le_bytes());
    bmp.extend_from_slice(&(height as i32).to_le_bytes());
    bmp.extend_from_slice(&1u16.to_le_bytes()); // planes
    bmp.extend_from_slice(&24u16.to_le_bytes()); // 24-bit BGR
    bmp.extend_from_slice(&0u32.to_le_bytes()); // BI_RGB (uncompressed)
    bmp.extend_from_slice(&image_size.to_le_bytes());
    bmp.extend_from_slice(&2835u32.to_le_bytes()); // 72 DPI
    bmp.extend_from_slice(&2835u32.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());
    bmp.extend_from_slice(&0u32.to_le_bytes());

    // Pixel data (bottom-up)
    for y in 0..height {
        let mut row_bytes = 0;
        for x in 0..width {
            let is_text = draw_text && ((10..=30).contains(&y) && (10..=50).contains(&x));
            if is_text {
                bmp.extend_from_slice(&[20, 20, 20]); // Dark foreground
            } else {
                bmp.extend_from_slice(&[240, 240, 240]); // Light background
            }
            row_bytes += 3;
        }
        while row_bytes % 4 != 0 {
            bmp.push(0);
            row_bytes += 1;
        }
    }

    bmp
}

#[tokio::test]
async fn test_streaming_media_chunking_and_probing() {
    // 1. Probe WAV
    let wav_data = create_synthetic_wav(16000, 32000); // 2.0 seconds
    let (fmt, duration_ms) =
        MediaTranscriptionPipeline::probe_media(&wav_data).expect("probe failed");
    assert_eq!(fmt, MediaFormat::Wav);
    assert!((1900..=2100).contains(&duration_ms));

    // 2. Chunked streaming extraction (2048 bytes per chunk)
    let chunks = MediaTranscriptionPipeline::extract_streaming_audio_chunks(&wav_data, 2048)
        .expect("extraction failed");
    assert!(!chunks.is_empty());
    assert!(chunks[0].len() <= 1024);

    // 3. Probe MP4 container header
    let mut mp4_header = Vec::new();
    mp4_header.extend_from_slice(&32u32.to_be_bytes());
    mp4_header.extend_from_slice(b"ftypisom");
    mp4_header.extend_from_slice(&[0u8; 16]);
    let (mp4_fmt, _) =
        MediaTranscriptionPipeline::probe_media(&mp4_header).expect("mp4 probe failed");
    assert_eq!(mp4_fmt, MediaFormat::Mp4);
}

#[tokio::test]
async fn test_subtitle_export_formats() {
    let segments = vec![
        MediaSubtitleSegment {
            id: "seg-1".to_string(),
            session_id: "media-test-1".to_string(),
            seq_id: 1,
            start_ms: 1000,
            end_ms: 3500,
            source_text: "ನಮಸ್ಕಾರ, ಹೇಗಿದ್ದೀರಿ?".to_string(),
            translated_text: "Hello, how are you?".to_string(),
        },
        MediaSubtitleSegment {
            id: "seg-2".to_string(),
            session_id: "media-test-1".to_string(),
            seq_id: 2,
            start_ms: 3600,
            end_ms: 6000,
            source_text: "ನಾನು ಚೆನ್ನಾಗಿದ್ದೇನೆ, ಧನ್ಯವಾದಗಳು.".to_string(),
            translated_text: "I am doing well, thank you.".to_string(),
        },
    ];

    // SRT Export
    let srt = MediaTranscriptionPipeline::format_srt(&segments, true);
    assert!(srt.contains("1\n00:00:01,000 --> 00:00:03,500\nHello, how are you?"));
    assert!(srt.contains("2\n00:00:03,600 --> 00:00:06,000\nI am doing well, thank you."));

    // VTT Export
    let vtt = MediaTranscriptionPipeline::format_vtt(&segments, true);
    assert!(vtt.starts_with("WEBVTT\n\n"));
    assert!(vtt.contains("00:00:01.000 --> 00:00:03.500"));

    // TXT Export
    let txt = MediaTranscriptionPipeline::format_txt(&segments, true);
    assert!(txt.contains("Hello, how are you?"));
    assert!(txt.contains("I am doing well, thank you."));

    // JSON Export
    let json_str = MediaTranscriptionPipeline::format_json(&segments).expect("json export failed");
    let json_val: serde_json::Value = serde_json::from_str(&json_str).expect("invalid json");
    assert_eq!(json_val.as_array().unwrap().len(), 2);
    assert_eq!(json_val[0]["translated_text"], "Hello, how are you?");
}

#[tokio::test]
async fn test_authentic_ocr_pipeline_and_in_place_translation() {
    let ocr = AuthenticOcrEngine::new();
    let bmp = create_synthetic_bmp(100, 60, true);

    let result = ocr
        .extract_text_and_regions(&bmp, "image/bmp", Some(LanguageCode::KnIn))
        .expect("ocr failed");

    assert!(!result.blocks.is_empty(), "Should extract text blocks");
    let block = &result.blocks[0];
    assert!(block.confidence > 0.0);
    assert!(!block.text.is_empty());
    assert!(block.translated_text.is_some());
    assert!(block.bounding_box.width > 0.0);
    assert!(block.bounding_box.height > 0.0);
    assert_eq!(result.detected_language, LanguageCode::KnIn);
}

#[tokio::test]
async fn test_document_processor_markdown_and_pdf_handling() {
    // 1. File size limit
    assert!(DocumentProcessor::validate_file_size(1000).is_ok());
    assert!(DocumentProcessor::validate_file_size((MAX_DOCUMENT_SIZE_BYTES + 1) as usize).is_err());

    // 2. Markdown with code blocks preservation
    let md_input = "# ನಮಸ್ಕಾರ\n\nಇದು ಪರೀಕ್ಷಾ ದಾಖಲೆ.\n\n```rust\nfn main() {\n    println!(\"hello\");\n}\n```\n\nಧನ್ಯವಾದಗಳು.";
    let blocks = DocumentProcessor::extract_blocks(md_input.as_bytes(), DocumentFormat::Markdown)
        .expect("md parse failed");

    assert_eq!(blocks.len(), 4);
    assert_eq!(blocks[0].source_text, "# ನಮಸ್ಕಾರ");
    assert_eq!(blocks[1].source_text, "ಇದು ಪರೀಕ್ಷಾ ದಾಖಲೆ.");
    // Code block preserved
    assert_eq!(
        blocks[2].source_text,
        "```rust\nfn main() {\n    println!(\"hello\");\n}\n```"
    );
    assert_eq!(blocks[3].source_text, "ಧನ್ಯವಾದಗಳು.");

    // Simulate translated blocks
    let mut translated_blocks = blocks.clone();
    translated_blocks[0].translated_text = "# Hello".to_string();
    translated_blocks[1].translated_text = "This is a test document.".to_string();
    // Code block translated_text remains untouched
    translated_blocks[2].translated_text = translated_blocks[2].source_text.clone();
    translated_blocks[3].translated_text = "Thank you.".to_string();

    let exported_md =
        DocumentProcessor::export_document(DocumentFormat::Markdown, &translated_blocks)
            .expect("export md failed");
    assert!(exported_md.contains("# Hello"));
    assert!(exported_md.contains("```rust"));

    // 3. PDF Stream parsing
    let pdf_stream = b"%PDF-1.4\nstream\nBT\n/F1 12 Tf\n(Hello World) Tj\nET\nendstream";
    let pdf_blocks = DocumentProcessor::extract_blocks(pdf_stream, DocumentFormat::Pdf)
        .expect("pdf parse failed");
    assert!(!pdf_blocks.is_empty());
    assert_eq!(pdf_blocks[0].source_text, "Hello World");

    // 4. CSV parsing and export
    let csv_input = "id,name,role\n1,Alice,Engineer\n2,Bob,Architect\n";
    let csv_blocks = DocumentProcessor::extract_blocks(csv_input.as_bytes(), DocumentFormat::Csv)
        .expect("csv parse failed");
    assert_eq!(csv_blocks.len(), 3);
    let mut translated_csv_blocks = csv_blocks.clone();
    translated_csv_blocks[0].translated_text = "ಕ್ರಮಾಂಕ,ಹೆಸರು,ಪಾತ್ರ".to_string();
    let exported_csv =
        DocumentProcessor::export_document(DocumentFormat::Csv, &translated_csv_blocks)
            .expect("export csv failed");
    assert!(exported_csv.contains("ಕ್ರಮಾಂಕ,ಹೆಸರು,ಪಾತ್ರ"));
}

#[tokio::test]
async fn test_platform_integrations_and_overlay_policy() {
    // 1. Overlay settings clamping
    let mut settings = OverlaySettings {
        enabled: true,
        always_on_top: true,
        x: 5000,
        y: 5000,
        width: 10000,
        height: 10000,
        font_size: 150,
        font_family: "Inter".into(),
        text_color: "#ffffff".into(),
        background_color: "#000000".into(),
        opacity: 2.0,
        source_lang: LanguageCode::EnUs,
        target_lang: LanguageCode::KnIn,
        show_source: true,
        show_translated: true,
    };
    LiveOverlayController::validate_and_clamp_settings(&mut settings);
    assert_eq!(settings.font_size, 48);
    assert_eq!(settings.opacity, 1.0);
    assert!(settings.width <= 1920);
    assert!(settings.height <= 600);

    // 2. Audio capture permission & DRM documentation
    let perm = LiveOverlayController::check_system_audio_permission();
    #[cfg(target_os = "windows")]
    assert_eq!(perm, SystemAudioPermission::Granted);
    let policy = LiveOverlayController::get_protected_content_policy();
    assert!(policy.contains("DRM") && policy.contains("HDCP"));

    // 3. Coordinator shortcuts validation
    assert!(
        PlatformIntegrationsCoordinator::validate_shortcut_accelerator("CommandOrControl+Shift+L")
            .is_ok()
    );
    assert!(PlatformIntegrationsCoordinator::validate_shortcut_accelerator("Alt+Shift+C").is_ok());
    assert!(
        PlatformIntegrationsCoordinator::validate_shortcut_accelerator("InvalidShortcut").is_err()
    );

    // 4. Non-invasive clipboard translation gating
    let clip_res = PlatformIntegrationsCoordinator::handle_user_clipboard_translation("  ನಮಸ್ಕಾರ  ")
        .expect("clipboard translate failed");
    assert_eq!(clip_res, "ನಮಸ್ಕಾರ");
    assert!(PlatformIntegrationsCoordinator::handle_user_clipboard_translation("   ").is_err());

    // 5. Native OS share sheet formatting
    let share =
        PlatformIntegrationsCoordinator::format_share_payload("Greeting", "ನಮಸ್ಕಾರ", "Hello");
    assert!(share.contains("Greeting"));
    assert!(share.contains("Original:\nನಮಸ್ಕಾರ"));
    assert!(share.contains("Translation:\nHello"));
}
