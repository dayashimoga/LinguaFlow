use crate::error::LinguaError;
use serde::{Deserialize, Serialize};
use std::fmt;
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LanguageCode {
    #[serde(rename = "kn-IN")]
    KnIn,
    #[serde(rename = "ta-IN")]
    TaIn,
    #[serde(rename = "en-US")]
    EnUs,
    #[serde(rename = "hi-IN")]
    HiIn,
    #[serde(rename = "te-IN")]
    TeIn,
    #[serde(rename = "ml-IN")]
    MlIn,
}

impl LanguageCode {
    pub fn as_bcp47(&self) -> &'static str {
        match self {
            LanguageCode::KnIn => "kn-IN",
            LanguageCode::TaIn => "ta-IN",
            LanguageCode::EnUs => "en-US",
            LanguageCode::HiIn => "hi-IN",
            LanguageCode::TeIn => "te-IN",
            LanguageCode::MlIn => "ml-IN",
        }
    }

    pub fn short_code(&self) -> &'static str {
        match self {
            LanguageCode::KnIn => "kn",
            LanguageCode::TaIn => "ta",
            LanguageCode::EnUs => "en",
            LanguageCode::HiIn => "hi",
            LanguageCode::TeIn => "te",
            LanguageCode::MlIn => "ml",
        }
    }

    pub fn english_name(&self) -> &'static str {
        match self {
            LanguageCode::KnIn => "Kannada",
            LanguageCode::TaIn => "Tamil",
            LanguageCode::EnUs => "English",
            LanguageCode::HiIn => "Hindi",
            LanguageCode::TeIn => "Telugu",
            LanguageCode::MlIn => "Malayalam",
        }
    }

    pub fn native_name(&self) -> &'static str {
        match self {
            LanguageCode::KnIn => "ಕನ್ನಡ",
            LanguageCode::TaIn => "தமிழ்",
            LanguageCode::EnUs => "English",
            LanguageCode::HiIn => "हिन्दी",
            LanguageCode::TeIn => "తెలుగు",
            LanguageCode::MlIn => "മലയാളം",
        }
    }

    pub fn script_name(&self) -> &'static str {
        match self {
            LanguageCode::KnIn => "Kannada (Knda)",
            LanguageCode::TaIn => "Tamil (Taml)",
            LanguageCode::EnUs => "Latin (Latn)",
            LanguageCode::HiIn => "Devanagari (Deva)",
            LanguageCode::TeIn => "Telugu (Telu)",
            LanguageCode::MlIn => "Malayalam (Mlym)",
        }
    }

    pub fn supported_core_languages() -> &'static [LanguageCode] {
        &[LanguageCode::KnIn, LanguageCode::TaIn, LanguageCode::EnUs]
    }
}

impl fmt::Display for LanguageCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_bcp47())
    }
}

impl FromStr for LanguageCode {
    type Err = LinguaError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let normalized = s.trim().to_lowercase();
        match normalized.as_str() {
            "kn-in" | "kn" | "kannada" => Ok(LanguageCode::KnIn),
            "ta-in" | "ta" | "tamil" => Ok(LanguageCode::TaIn),
            "en-us" | "en-gb" | "en" | "english" => Ok(LanguageCode::EnUs),
            "hi-in" | "hi" | "hindi" => Ok(LanguageCode::HiIn),
            "te-in" | "te" | "telugu" => Ok(LanguageCode::TeIn),
            "ml-in" | "ml" | "malayalam" => Ok(LanguageCode::MlIn),
            other => Err(LinguaError::UnsupportedLanguage(
                other.to_string(),
                "LanguageCode parser".to_string(),
            )),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct LanguagePair {
    pub source: LanguageCode,
    pub target: LanguageCode,
}

impl LanguagePair {
    pub fn new(source: LanguageCode, target: LanguageCode) -> Result<Self, LinguaError> {
        if source == target {
            return Err(LinguaError::IdenticalLanguagePair(
                source.as_bcp47().to_string(),
                target.as_bcp47().to_string(),
            ));
        }
        Ok(Self { source, target })
    }

    pub fn swapped(&self) -> Self {
        Self {
            source: self.target,
            target: self.source,
        }
    }

    pub fn pair_key(&self) -> String {
        format!("{}_{}", self.source.as_bcp47(), self.target.as_bcp47())
    }

    pub fn validate(&self) -> Result<(), LinguaError> {
        if self.source == self.target {
            return Err(LinguaError::IdenticalLanguagePair(
                self.source.as_bcp47().to_string(),
                self.target.as_bcp47().to_string(),
            ));
        }
        Ok(())
    }
}

impl fmt::Display for LanguagePair {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} -> {}",
            self.source.as_bcp47(),
            self.target.as_bcp47()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_language_parsing_and_bcp47() {
        assert_eq!(LanguageCode::from_str("kn").unwrap(), LanguageCode::KnIn);
        assert_eq!(LanguageCode::from_str("KN-IN").unwrap(), LanguageCode::KnIn);
        assert_eq!(LanguageCode::from_str("Tamil").unwrap(), LanguageCode::TaIn);
        assert_eq!(LanguageCode::from_str("en-US").unwrap(), LanguageCode::EnUs);
        assert!(LanguageCode::from_str("invalid-lang").is_err());
    }

    #[test]
    fn test_language_metadata() {
        let kn = LanguageCode::KnIn;
        assert_eq!(kn.as_bcp47(), "kn-IN");
        assert_eq!(kn.english_name(), "Kannada");
        assert_eq!(kn.native_name(), "ಕನ್ನಡ");

        let ta = LanguageCode::TaIn;
        assert_eq!(ta.as_bcp47(), "ta-IN");
        assert_eq!(ta.native_name(), "தமிழ்");
    }

    #[test]
    fn test_language_pair_validation() {
        let pair = LanguagePair::new(LanguageCode::KnIn, LanguageCode::EnUs);
        assert!(pair.is_ok());
        let valid_pair = pair.unwrap();
        assert_eq!(valid_pair.pair_key(), "kn-IN_en-US");

        let swapped = valid_pair.swapped();
        assert_eq!(swapped.source, LanguageCode::EnUs);
        assert_eq!(swapped.target, LanguageCode::KnIn);

        // Identical pair must be rejected
        let invalid = LanguagePair::new(LanguageCode::KnIn, LanguageCode::KnIn);
        assert!(invalid.is_err());
        match invalid {
            Err(LinguaError::IdenticalLanguagePair(s, t)) => {
                assert_eq!(s, "kn-IN");
                assert_eq!(t, "kn-IN");
            }
            _ => panic!("Expected IdenticalLanguagePair error"),
        }
    }
}
