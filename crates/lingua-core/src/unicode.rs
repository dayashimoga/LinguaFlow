use crate::error::LinguaError;
use unicode_normalization::UnicodeNormalization;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DetectedScript {
    Kannada,
    Tamil,
    Latin,
    Devanagari,
    Mixed,
    CommonOrPunctuation,
}

pub fn normalize_text_nfc(input: &str) -> String {
    input.nfc().collect::<String>()
}

pub fn sanitize_translation_input(input: &str, max_bytes: usize) -> Result<String, LinguaError> {
    if input.len() > max_bytes {
        return Err(LinguaError::Validation(format!(
            "Input text size ({} bytes) exceeds maximum allowable limit ({} bytes)",
            input.len(),
            max_bytes
        )));
    }

    let normalized = normalize_text_nfc(input);
    let trimmed = normalized.trim();
    if trimmed.is_empty() {
        return Err(LinguaError::Validation(
            "Input text cannot be empty or solely whitespace".into(),
        ));
    }

    Ok(trimmed.to_string())
}

pub fn detect_script(text: &str) -> DetectedScript {
    let mut has_kannada = false;
    let mut has_tamil = false;
    let mut has_latin = false;
    let mut has_deva = false;

    for ch in text.chars() {
        match ch as u32 {
            0x0C80..=0x0CFF => has_kannada = true,
            0x0B80..=0x0BFF => has_tamil = true,
            0x0900..=0x097F => has_deva = true,
            0x0041..=0x005A | 0x0061..=0x007A => has_latin = true,
            _ => {}
        }
    }

    let script_count =
        (has_kannada as u8) + (has_tamil as u8) + (has_latin as u8) + (has_deva as u8);
    if script_count > 1 {
        DetectedScript::Mixed
    } else if has_kannada {
        DetectedScript::Kannada
    } else if has_tamil {
        DetectedScript::Tamil
    } else if has_latin {
        DetectedScript::Latin
    } else if has_deva {
        DetectedScript::Devanagari
    } else {
        DetectedScript::CommonOrPunctuation
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_script_detection() {
        assert_eq!(detect_script("ನಮಸ್ಕಾರ"), DetectedScript::Kannada);
        assert_eq!(detect_script("வணக்கம்"), DetectedScript::Tamil);
        assert_eq!(detect_script("Hello world"), DetectedScript::Latin);
        assert_eq!(detect_script("ನಮಸ್ಕಾರ / Hello"), DetectedScript::Mixed);
        assert_eq!(
            detect_script("12345 !?"),
            DetectedScript::CommonOrPunctuation
        );
    }

    #[test]
    fn test_sanitization_and_bounding() {
        let valid = sanitize_translation_input("  ಕನ್ನಡ ನುಡಿ  ", 1000).unwrap();
        assert_eq!(valid, "ಕನ್ನಡ ನುಡಿ");

        let empty = sanitize_translation_input("   \t\n ", 1000);
        assert!(empty.is_err());

        let oversized = sanitize_translation_input("test text", 4);
        assert!(oversized.is_err());
    }
}
