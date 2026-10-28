use chrono::Utc;
use lingua_core::{EvaluationReport, EvaluationResult, EvaluationSample, LanguageCode};
use std::collections::HashMap;

/// Quality evaluation suite computing chrF, Word Error Rate (WER), and Character Error Rate (CER).
pub struct QualityEvaluator;

impl QualityEvaluator {
    /// Compute chrF score (character n-gram F-score, Popović 2015).
    /// Returns score in percentage (0.0 to 100.0).
    pub fn compute_chrf(hypothesis: &str, reference: &str) -> f64 {
        let hyp_chars: Vec<char> = hypothesis.chars().filter(|c| !c.is_whitespace()).collect();
        let ref_chars: Vec<char> = reference.chars().filter(|c| !c.is_whitespace()).collect();

        if ref_chars.is_empty() {
            return if hyp_chars.is_empty() { 100.0 } else { 0.0 };
        }
        if hyp_chars.is_empty() {
            return 0.0;
        }

        const MAX_N: usize = 6;
        let mut total_f = 0.0;
        let mut n_counted = 0;

        let max_evaluated_n = MAX_N.min(ref_chars.len()).min(hyp_chars.len());
        if max_evaluated_n == 0 {
            return 0.0;
        }

        for n in 1..=max_evaluated_n {
            let hyp_ngrams = Self::get_char_ngrams(&hyp_chars, n);
            let ref_ngrams = Self::get_char_ngrams(&ref_chars, n);

            let mut matched = 0;
            let mut ref_counts = ref_ngrams.clone();
            for (ng, count) in hyp_ngrams.iter() {
                if let Some(ref_count) = ref_counts.get_mut(ng) {
                    let overlap = (*count).min(*ref_count);
                    matched += overlap;
                    *ref_count -= overlap;
                }
            }

            let total_hyp: usize = hyp_ngrams.values().sum();
            let total_ref: usize = ref_ngrams.values().sum();

            if total_hyp == 0 || total_ref == 0 {
                continue;
            }

            let precision = (matched as f64) / (total_hyp as f64);
            let recall = (matched as f64) / (total_ref as f64);

            let beta: f64 = 2.0; // Standard chrF beta
            let beta_sq = beta * beta;

            let f_score = if precision + recall > 1e-9 {
                (1.0 + beta_sq) * precision * recall / (beta_sq * precision + recall)
            } else {
                0.0
            };
            total_f += f_score;
            n_counted += 1;
        }

        if n_counted == 0 {
            0.0
        } else {
            (total_f / (n_counted as f64)) * 100.0
        }
    }

    fn get_char_ngrams(chars: &[char], n: usize) -> HashMap<String, usize> {
        let mut map = HashMap::new();
        if chars.len() < n {
            return map;
        }
        for window in chars.windows(n) {
            let s: String = window.iter().collect();
            *map.entry(s).or_insert(0) += 1;
        }
        map
    }

    /// Compute Word Error Rate (WER).
    /// Returns percentage (0.0% is perfect match).
    pub fn compute_wer(hypothesis: &str, reference: &str) -> f64 {
        let hyp_words: Vec<&str> = hypothesis.split_whitespace().collect();
        let ref_words: Vec<&str> = reference.split_whitespace().collect();

        if ref_words.is_empty() {
            return if hyp_words.is_empty() { 0.0 } else { 100.0 };
        }

        let dist = Self::levenshtein_distance(&hyp_words, &ref_words);
        ((dist as f64) / (ref_words.len() as f64)) * 100.0
    }

    /// Compute Character Error Rate (CER).
    /// Returns percentage (0.0% is perfect match).
    pub fn compute_cer(hypothesis: &str, reference: &str) -> f64 {
        let hyp_chars: Vec<char> = hypothesis.chars().collect();
        let ref_chars: Vec<char> = reference.chars().collect();

        if ref_chars.is_empty() {
            return if hyp_chars.is_empty() { 0.0 } else { 100.0 };
        }

        let dist = Self::levenshtein_distance(&hyp_chars, &ref_chars);
        ((dist as f64) / (ref_chars.len() as f64)) * 100.0
    }

    fn levenshtein_distance<T: PartialEq>(s1: &[T], s2: &[T]) -> usize {
        let m = s1.len();
        let n = s2.len();
        let mut dp = vec![vec![0; n + 1]; m + 1];

        for (i, row) in dp.iter_mut().enumerate() {
            row[0] = i;
        }
        for (j, val) in dp[0].iter_mut().enumerate() {
            *val = j;
        }

        for i in 1..=m {
            for j in 1..=n {
                let cost = if s1[i - 1] == s2[j - 1] { 0 } else { 1 };
                dp[i][j] = (dp[i - 1][j] + 1)
                    .min(dp[i][j - 1] + 1)
                    .min(dp[i - 1][j - 1] + cost);
            }
        }

        dp[m][n]
    }

    /// Curated evaluation benchmark test set
    pub fn curated_evaluation_dataset() -> Vec<EvaluationSample> {
        vec![
            // Colloquial ordinary speech
            EvaluationSample {
                id: "eval-01".into(),
                category: "colloquial".into(),
                source_lang: LanguageCode::KnIn,
                target_lang: LanguageCode::EnUs,
                source_text: "ಹೇಗಿದ್ದೀರಾ? ಊಟ ಆಯ್ತಾ?".into(),
                reference_translation: "How are you? Did you eat?".into(),
            },
            EvaluationSample {
                id: "eval-02".into(),
                category: "colloquial".into(),
                source_lang: LanguageCode::TaIn,
                target_lang: LanguageCode::EnUs,
                source_text: "எப்படி இருக்கீங்க? சாப்பிட்டாச்சா?".into(),
                reference_translation: "How are you? Had food?".into(),
            },
            // Formal usage
            EvaluationSample {
                id: "eval-03".into(),
                category: "formal".into(),
                source_lang: LanguageCode::EnUs,
                target_lang: LanguageCode::KnIn,
                source_text: "Please submit the quarterly report by tomorrow morning.".into(),
                reference_translation: "ದಯವಿಟ್ಟು ನಾಳೆ ಬೆಳಿಗ್ಗೆಯೊಳಗೆ ತ್ರೈಮಾಸಿಕ ವರದಿಯನ್ನು ಸಲ್ಲಿಸಿ.".into(),
            },
            EvaluationSample {
                id: "eval-04".into(),
                category: "formal".into(),
                source_lang: LanguageCode::EnUs,
                target_lang: LanguageCode::TaIn,
                source_text: "Please submit the quarterly report by tomorrow morning.".into(),
                reference_translation: "தயவுசெய்து நாளை காலைக்குள் காலாண்டு அறிக்கையை சமர்ப்பிக்கவும்.".into(),
            },
            // Code-mixing
            EvaluationSample {
                id: "eval-05".into(),
                category: "code_mixing".into(),
                source_lang: LanguageCode::KnIn,
                target_lang: LanguageCode::EnUs,
                source_text: "ನಾಳೆ ಆಫೀಸ್ ಮೀಟಿಂಗ್ ಇದೆ, ದಯವಿಟ್ಟು ಬೇಗ ಬನ್ನಿ.".into(),
                reference_translation: "There is an office meeting tomorrow, please come early."
                    .into(),
            },
            // Technical terminology
            EvaluationSample {
                id: "eval-06".into(),
                category: "specialist".into(),
                source_lang: LanguageCode::EnUs,
                target_lang: LanguageCode::KnIn,
                source_text: "The software updates the database efficiently.".into(),
                reference_translation: "ತಂತ್ರಾಂಶವು ದತ್ತಸಂಚಯವನ್ನು ಪರಿಣಾಮಕಾರಿಯಾಗಿ ನವೀಕರಿಸುತ್ತದೆ.".into(),
            },
            EvaluationSample {
                id: "eval-07".into(),
                category: "specialist".into(),
                source_lang: LanguageCode::EnUs,
                target_lang: LanguageCode::TaIn,
                source_text: "The software updates the database efficiently.".into(),
                reference_translation: "மென்பொருள் தரவுத்தளத்தை திறமையாக புதுப்பிக்கிறது.".into(),
            },
            // Idioms
            EvaluationSample {
                id: "eval-08".into(),
                category: "idioms".into(),
                source_lang: LanguageCode::KnIn,
                target_lang: LanguageCode::EnUs,
                source_text: "ಕೈ ಕೆಸರಾದರೆ ಬಾಯಿ ಮೊಸರು.".into(),
                reference_translation: "Hard work yields sweet fruits.".into(),
            },
        ]
    }

    /// Evaluate a set of hypotheses against references and generate a structured evaluation report.
    pub fn evaluate_batch(hypotheses: &[(EvaluationSample, String)]) -> EvaluationReport {
        let mut results = Vec::new();
        let mut total_chrf = 0.0;
        let mut total_wer = 0.0;
        let mut total_cer = 0.0;

        for (sample, hyp) in hypotheses {
            let chrf = Self::compute_chrf(hyp, &sample.reference_translation);
            let wer = Self::compute_wer(hyp, &sample.reference_translation);
            let cer = Self::compute_cer(hyp, &sample.reference_translation);

            let mut critical_errors = Vec::new();
            if hyp.trim().is_empty() {
                critical_errors.push("Empty translation produced".into());
            }

            total_chrf += chrf;
            total_wer += wer;
            total_cer += cer;

            results.push(EvaluationResult {
                sample_id: sample.id.clone(),
                category: sample.category.clone(),
                hypothesis: hyp.clone(),
                reference: sample.reference_translation.clone(),
                chrf_score: chrf,
                wer_score: wer,
                cer_score: cer,
                critical_errors,
            });
        }

        let n = hypotheses.len().max(1) as f64;
        EvaluationReport {
            total_samples: hypotheses.len(),
            mean_chrf: total_chrf / n,
            mean_wer: total_wer / n,
            mean_cer: total_cer / n,
            samples: results,
            evaluated_at: Utc::now(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chrf_identical_and_different() {
        let score_same = QualityEvaluator::compute_chrf("ನಮಸ್ಕಾರ", "ನಮಸ್ಕಾರ");
        assert!((score_same - 100.0).abs() < 1e-4);

        let score_diff = QualityEvaluator::compute_chrf("hello", "goodbye");
        assert!(score_diff < 30.0);
    }

    #[test]
    fn test_wer_and_cer() {
        let wer = QualityEvaluator::compute_wer("hello world", "hello there world");
        // 1 insertion / 3 reference words = 33.3%
        assert!((wer - 33.33).abs() < 1.0);

        let cer = QualityEvaluator::compute_cer("abc", "abd");
        // 1 substitution / 3 reference chars = 33.3%
        assert!((cer - 33.33).abs() < 1.0);
    }

    #[test]
    fn test_batch_evaluation_report() {
        let sample = EvaluationSample {
            id: "test-1".into(),
            category: "colloquial".into(),
            source_lang: LanguageCode::KnIn,
            target_lang: LanguageCode::EnUs,
            source_text: "ನಮಸ್ಕಾರ".into(),
            reference_translation: "Hello".into(),
        };

        let report = QualityEvaluator::evaluate_batch(&[(sample, "Hello".to_string())]);
        assert_eq!(report.total_samples, 1);
        assert!((report.mean_chrf - 100.0).abs() < 1e-4);
        assert_eq!(report.mean_wer, 0.0);
    }
}
