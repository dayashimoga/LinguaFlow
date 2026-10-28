use chrono::{DateTime, Duration, Utc};
use lingua_core::{
    DictionaryExample, GrammarGuideTopic, LanguageCode, ListeningChallenge,
    PronunciationAssessmentResult, ReviewRating,
};

/// Authentic SuperMemo-2 (SM-2) Spaced-Repetition System (SRS).
pub struct SpacedRepetitionSystem;

impl SpacedRepetitionSystem {
    /// Calculate the next repetition count, interval (days), ease factor, and review timestamp
    /// according to the classic SM-2 algorithm:
    ///
    /// - If rating < 3 (Blackout, Incorrect, Hard):
    ///   repetition = 0, interval = 1.0 day
    /// - If rating >= 3 (Good, Easy, Perfect):
    ///   - if repetition == 0: interval = 1.0 day
    ///   - if repetition == 1: interval = 6.0 days
    ///   - if repetition >= 2: interval = round(interval * ease_factor)
    ///   - repetition += 1
    /// - Ease factor adjustment:
    ///   EF' = EF + (0.1 - (5 - q) * (0.08 + (5 - q) * 0.02)), clamped to EF >= 1.3
    pub fn calculate_next_interval(
        rating: ReviewRating,
        current_rep: u32,
        current_interval_days: f32,
        current_ease_factor: f32,
    ) -> (u32, f32, f32, DateTime<Utc>) {
        let q = rating as u8 as f32;

        // 1. Calculate updated ease factor
        let delta = 0.1 - (5.0 - q) * (0.08 + (5.0 - q) * 0.02);
        let mut new_ease = current_ease_factor + delta;
        if new_ease < 1.3 {
            new_ease = 1.3;
        }

        // 2. Calculate repetition count and interval
        let (new_rep, new_interval) = if q < 3.0 {
            // Failure: reset repetition sequence
            (0u32, 1.0f32)
        } else {
            // Success: advance repetition
            match current_rep {
                0 => (1u32, 1.0f32),
                1 => (2u32, 6.0f32),
                _ => {
                    let next_days = (current_interval_days * new_ease).round().max(1.0);
                    (current_rep + 1, next_days)
                }
            }
        };

        // 3. Compute target review timestamp
        let hours = (new_interval * 24.0).round() as i64;
        let next_review_at = Utc::now() + Duration::hours(hours);

        (new_rep, new_interval, new_ease, next_review_at)
    }
}

/// Pronunciation and speaking assessment using Popović chrF & Levenshtein alignment.
pub struct PronunciationEvaluator;

impl PronunciationEvaluator {
    /// Assess pronunciation fidelity between reference target text and learner's speech transcript.
    pub fn assess(
        reference: &str,
        hypothesis: &str,
        _lang: LanguageCode,
    ) -> PronunciationAssessmentResult {
        let ref_clean = reference.trim();
        let hyp_clean = hypothesis.trim();

        if ref_clean.is_empty() {
            return PronunciationAssessmentResult {
                accuracy_score: 0.0,
                wer: 100.0,
                cer: 100.0,
                chrf: 0.0,
                phoneme_feedback: vec!["Reference utterance is empty".into()],
                is_passing: false,
            };
        }

        if hyp_clean.is_empty() {
            return PronunciationAssessmentResult {
                accuracy_score: 0.0,
                wer: 100.0,
                cer: 100.0,
                chrf: 0.0,
                phoneme_feedback: vec!["No spoken speech detected".into()],
                is_passing: false,
            };
        }

        // 1. Compute Popović chrF (character 6-grams)
        let chrf = crate::evaluation::QualityEvaluator::compute_chrf(hyp_clean, ref_clean) as f32;

        // 2. Compute WER & CER via optimal dynamic programming
        let wer = crate::evaluation::QualityEvaluator::compute_wer(hyp_clean, ref_clean) as f32;
        let cer = crate::evaluation::QualityEvaluator::compute_cer(hyp_clean, ref_clean) as f32;

        // 3. Composite accuracy score: weighted combination of chrF and (1 - CER)
        let cer_factor = (1.0 - (cer / 100.0)).clamp(0.0, 1.0);
        let chrf_factor = (chrf / 100.0).clamp(0.0, 1.0);
        let accuracy_score = ((chrf_factor * 0.6 + cer_factor * 0.4) * 100.0).clamp(0.0, 100.0);

        // 4. Generate actionable token feedback
        let ref_words: Vec<&str> = ref_clean.split_whitespace().collect();
        let hyp_words: Vec<&str> = hyp_clean.split_whitespace().collect();

        let mut feedback = Vec::new();
        for (i, r_w) in ref_words.iter().enumerate() {
            if let Some(h_w) = hyp_words.get(i) {
                if r_w.eq_ignore_ascii_case(h_w) || *r_w == *h_w {
                    feedback.push(format!("Word '{r_w}': Excellent pronunciation"));
                } else {
                    feedback.push(format!(
                        "Word '{r_w}': Spoken as '{h_w}', check vowel/consonant length"
                    ));
                }
            } else {
                feedback.push(format!("Word '{r_w}': Word omitted or silent"));
            }
        }

        let is_passing = accuracy_score >= 70.0;

        PronunciationAssessmentResult {
            accuracy_score,
            wer,
            cer,
            chrf,
            phoneme_feedback: feedback,
            is_passing,
        }
    }
}

/// Listening comprehension challenges generator with authentic options.
pub struct ListeningChallengeGenerator;

impl ListeningChallengeGenerator {
    pub fn get_challenges(lang: LanguageCode, count: usize) -> Vec<ListeningChallenge> {
        let all_challenges = match lang {
            LanguageCode::KnIn => vec![
                ListeningChallenge {
                    id: "quiz-kn-01".into(),
                    prompt_audio_text: "ನಮಸ್ಕಾರ, ನೀವು ಹೇಗಿದ್ದೀರಿ?".into(),
                    language: LanguageCode::KnIn,
                    correct_answer: "Hello, how are you?".into(),
                    options: vec![
                        "Hello, how are you?".into(),
                        "Where is the library?".into(),
                        "Thank you very much.".into(),
                        "Good evening, see you tomorrow.".into(),
                    ],
                    explanation: "ನಮಸ್ಕಾರ (Namaskara = Hello), ನೀವು (neevu = you), ಹೇಗಿದ್ದೀರಿ (hegiddiri = how are you).".into(),
                },
                ListeningChallenge {
                    id: "quiz-kn-02".into(),
                    prompt_audio_text: "ದಯವಿಟ್ಟು ಕುಡಿಯಲು ನೀರು ಕೊಡಿ.".into(),
                    language: LanguageCode::KnIn,
                    correct_answer: "Please give water to drink.".into(),
                    options: vec![
                        "Please give water to drink.".into(),
                        "Where can I buy a ticket?".into(),
                        "I am going to Bengaluru.".into(),
                        "This book is very interesting.".into(),
                    ],
                    explanation: "ದಯವಿಟ್ಟು (dayavittu = please), ಕುಡಿಯಲು (kudiyalu = to drink), ನೀರು (neeru = water), ಕೊಡಿ (kodi = give).".into(),
                },
                ListeningChallenge {
                    id: "quiz-kn-03".into(),
                    prompt_audio_text: "ನಾನು ಹೊಸ ಪುಸ್ತಕವನ್ನು ಓದುತ್ತಿದ್ದೇನೆ.".into(),
                    language: LanguageCode::KnIn,
                    correct_answer: "I am reading a new book.".into(),
                    options: vec![
                        "I am reading a new book.".into(),
                        "We bought a new house.".into(),
                        "What is your name?".into(),
                        "Please speak slowly.".into(),
                    ],
                    explanation: "ನಾನು (naanu = I), ಹೊಸ (hosa = new), ಪುಸ್ತಕವನ್ನು (pustakavannu = book [Accusative]), ಓದುತ್ತಿದ್ದೇನೆ (oduttiddene = reading [Present Continuous]).".into(),
                },
            ],
            LanguageCode::TaIn => vec![
                ListeningChallenge {
                    id: "quiz-ta-01".into(),
                    prompt_audio_text: "வணக்கம், நீங்கள் எப்படி இருக்கிறீர்கள்?".into(),
                    language: LanguageCode::TaIn,
                    correct_answer: "Hello, how are you?".into(),
                    options: vec![
                        "Hello, how are you?".into(),
                        "What is the time now?".into(),
                        "Thank you for coming.".into(),
                        "I would like to drink tea.".into(),
                    ],
                    explanation: "வணக்கம் (Vanakkam = Hello), நீங்கள் (neengal = you [polite]), எப்படி இருக்கிறீர்கள் (eppadi irukkireergal = how are you).".into(),
                },
                ListeningChallenge {
                    id: "quiz-ta-02".into(),
                    prompt_audio_text: "குடிப்பதற்கு தண்ணீர் கொடுங்கள்.".into(),
                    language: LanguageCode::TaIn,
                    correct_answer: "Please give water to drink.".into(),
                    options: vec![
                        "Please give water to drink.".into(),
                        "Where is the train station?".into(),
                        "The meal was delicious.".into(),
                        "See you tomorrow morning.".into(),
                    ],
                    explanation: "குடிப்பதற்கு (kudippadharku = to drink), தண்ணீர் (thanneer = water), கொடுங்கள் (kodungal = please give).".into(),
                },
                ListeningChallenge {
                    id: "quiz-ta-03".into(),
                    prompt_audio_text: "நான் ஒரு நல்ல புத்தகத்தை படிக்கிறேன்.".into(),
                    language: LanguageCode::TaIn,
                    correct_answer: "I am reading a good book.".into(),
                    options: vec![
                        "I am reading a good book.".into(),
                        "I live in Chennai.".into(),
                        "Thank you very much.".into(),
                        "Can you help me?".into(),
                    ],
                    explanation: "நான் (naan = I), நல்ல (nalla = good), புத்தகத்தை (puththakaththai = book [Accusative -ai marker]), படிக்கிறேன் (padikkiren = reading).".into(),
                },
            ],
            _ => vec![
                ListeningChallenge {
                    id: "quiz-en-01".into(),
                    prompt_audio_text: "Good morning, welcome to LinguaFlow.".into(),
                    language: LanguageCode::EnUs,
                    correct_answer: "ಶುಭೋದಯ, ಲಿಂಗ್ವಾಫ್ಲೋಗೆ ಸುಸ್ವಾಗತ.".into(),
                    options: vec![
                        "ಶುಭೋದಯ, ಲಿಂಗ್ವಾಫ್ಲೋಗೆ ಸುಸ್ವಾಗತ.".into(),
                        "ಧನ್ಯವಾದಗಳು ಮತ್ತು ವಿದಾಯ.".into(),
                        "ದಯವಿಟ್ಟು ನೀರು ಕೊಡಿ.".into(),
                        "ನೀವು ಹೇಗಿದ್ದೀರಿ?".into(),
                    ],
                    explanation: "Standard English morning greeting and welcome expression.".into(),
                },
            ],
        };

        all_challenges.into_iter().take(count.max(1)).collect()
    }
}

/// Grammar explanations and reference guide for Kannada, Tamil, and English.
pub struct GrammarKnowledgeBase;

impl GrammarKnowledgeBase {
    pub fn get_topics(lang: LanguageCode) -> Vec<GrammarGuideTopic> {
        match lang {
            LanguageCode::KnIn => vec![
                GrammarGuideTopic {
                    id: "kn-vibhakti".into(),
                    language: LanguageCode::KnIn,
                    title: "Kannada Noun Cases (ವಿಭಕ್ತಿ ಪ್ರತ್ಯಯಗಳು - Vibhakti Pratyayagaḷu)".into(),
                    category: "Morphology & Cases".into(),
                    content_markdown: "Kannada has 8 traditional nominal cases (Vibhakti) formed by agglutinating case suffixes to noun bases:\n\n1. **ಪ್ರಥಮಾ (Nominative)**: Marker `ಉ` (u) — Agent/Subject (e.g., `ರಾಮನು` Rāmanu).\n2. **ದ್ವಿತೀಯಾ (Accusative)**: Marker `ಅನ್ನು` (annu) — Direct Object (e.g., `ಮನೆಯನ್ನು` maneyannu - the house).\n3. **ತೃತೀಯಾ (Instrumental/Ablative)**: Marker `ಇಂದ` (inda) — By/From (e.g., `ಪುಸ್ತಕದಿಂದ` pustakadinda - from the book).\n4. **ಚತುರ್ಥೀ (Dative)**: Marker `ಗೆ/ಇಗೆ/ಕ್ಕೆ` (ge/ige/kke) — To/For (e.g., `ನೀರಿಗೆ` nīrige - for water, `ಊರಿಗೆ` ūrige).\n5. **ಪಂಚಮೀ (Ablative)**: Marker `ದಿಸೆಯಿಂದ/ಅತ್ತಣಿಂದ` (classical).\n6. **ಷಷ್ಠೀ (Genitive)**: Marker `ಅ` (a) — Possessive (e.g., `ನನ್ನ` nanna - my, `ಮನೆಯ` maneya - of the house).\n7. **ಸಪ್ತಮೀ (Locative)**: Marker `ಅಲ್ಲಿ` (alli) — In/At/On (e.g., `ಮನೆಯಲ್ಲಿ` maneyalli - in the house).\n8. **ಸಂಬೋಧನಾ (Vocative)**: Marker `ಏ/ಇರಾ` (ē/irā) — Calling/Addressing (e.g., `ರಾಮನೇ` Rāmanē).".into(),
                    examples: vec![
                        DictionaryExample {
                            source: "ನಾನು ಮನೆಯಲ್ಲಿ ಇದ್ದೇನೆ.".into(),
                            target: "I am in the house. (ಮನೆ + ಅಲ್ಲಿ -> ಮನೆಯಲ್ಲಿ [Locative])".into(),
                        },
                        DictionaryExample {
                            source: "ಅವನು ಪುಸ್ತಕವನ್ನು ಕೊಟ್ಟನು.".into(),
                            target: "He gave the book. (ಪುಸ್ತಕ + ಅನ್ನು -> ಪುಸ್ತಕವನ್ನು [Accusative])".into(),
                        },
                    ],
                },
                GrammarGuideTopic {
                    id: "kn-verbs".into(),
                    language: LanguageCode::KnIn,
                    title: "Kannada Tenses & Agreement (ಕಾಲಗಳು ಮತ್ತು ಆಖ್ಯಾತ ಪ್ರತ್ಯಯಗಳು)".into(),
                    category: "Verbal Conjugation".into(),
                    content_markdown: "Kannada verbs conjugate by attaching tense markers followed by personal gender-number pronominal suffixes:\n\n- **Past Tense (ಭೂತಕಾಲ)**: Infix `-ದ-/-ತ-/-ಟ-` (e.g., `ಮಾಡಿದನು` mādidanu - he did).\n- **Present Continuous (ವರ್ತಮಾನಕಾಲ)**: Infix `-ಉತ್ತ-/-ಉತ್ತಿ-` + auxiliary (e.g., `ಮಾಡುತ್ತಿದ್ದಾನೆ` māduttiddāne - he is doing).\n- **Future / Habitual (ಭವಿಷ್ಯತ್ ಕಾಲ)**: Infix `-ವ-/-ಉವ-` (e.g., `ಮಾಡುವನು` māduvanu - he will do).".into(),
                    examples: vec![
                        DictionaryExample {
                            source: "ಅವಳು ಹಾಡುತ್ತಿದ್ದಾಳೆ.".into(),
                            target: "She is singing. (ಹಾಡು + ಉತ್ತ + ಇದ್ದಾಳೆ)".into(),
                        },
                    ],
                },
            ],
            LanguageCode::TaIn => vec![
                GrammarGuideTopic {
                    id: "ta-veetrumai".into(),
                    language: LanguageCode::TaIn,
                    title: "Tamil Case System (வேற்றுமை உருபுகள் - Vēṟṟumai Urupugaḷ)".into(),
                    category: "Morphology & Cases".into(),
                    content_markdown: "Tamil features an eight-case system where postpositional markers join noun stems:\n\n1. **முதலாம் வேற்றுமை (Nominative)**: எழுவாய் (Unmarked subject, e.g., `மரம்` maram - tree).\n2. **இரண்டாம் வேற்றுமை (Accusative)**: Marker `ஐ` (ai) — Object (e.g., `வீட்டை` vīṭṭai - the house).\n3. **மூன்றாம் வேற்றுமை (Instrumental/Sociative)**: Marker `ஆல் / உடன் / ஓடு` (āl / uṭan / ōṭu) — With/By (e.g., `புத்தகத்தால்` - by the book).\n4. **நான்காம் வேற்றுமை (Dative)**: Marker `கு / உக்கு` (ku / ukku) — To/For (e.g., `வீட்டிற்கு` vīṭṭiṟku - to the house).\n5. **ஐந்தாம் வேற்றுமை (Ablative)**: Marker `இல் / இன் / இருந்து` (il / in / irunthu) — From (e.g., `ஊரிலிருந்து` - from town).\n6. **ஆறாம் வேற்றுமை (Genitive)**: Marker `அது / உடைய` (athu / uṭaiya) — Possessive (e.g., `என்னுடைய` - my).\n7. **ஏழாம் வேற்றுமை (Locative)**: Marker `இல் / கண்` (il / kaṇ) — In/At (e.g., `வீட்டில்` vīṭṭil - in the house).\n8. **எட்டாம் வேற்றுமை (Vocative)**: விளி (Addressing marker, e.g., `அம்மா!` - Mother!).".into(),
                    examples: vec![
                        DictionaryExample {
                            source: "நான் வீட்டில் இருக்கிறேன்.".into(),
                            target: "I am in the house. (வீடு + இல் -> வீட்டில் [Locative])".into(),
                        },
                        DictionaryExample {
                            source: "அவர் தம்பிக்கு ஒரு பரிசு கொடுத்தார்.".into(),
                            target: "He gave a gift to younger brother. (தம்பி + க்கு -> தம்பிக்கு [Dative])".into(),
                        },
                    ],
                },
            ],
            _ => vec![
                GrammarGuideTopic {
                    id: "en-articles".into(),
                    language: LanguageCode::EnUs,
                    title: "English Definite and Indefinite Articles".into(),
                    category: "Syntax".into(),
                    content_markdown: "English uses 'a' and 'an' for indefinite non-specific singular nouns, and 'the' for definite entities familiar to the listener.".into(),
                    examples: vec![
                        DictionaryExample {
                            source: "I saw a house.".into(),
                            target: "ನಾನು ಒಂದು ಮನೆಯನ್ನು ನೋಡಿದೆ.".into(),
                        },
                    ],
                },
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sm2_progression_and_ease_factor() {
        // Grade 4 (Easy) on initial card: delta = 0.0 -> EF stays 2.5
        let (rep1, int1, ef1, next1) =
            SpacedRepetitionSystem::calculate_next_interval(ReviewRating::Easy, 0, 1.0, 2.5);
        assert_eq!(rep1, 1);
        assert_eq!(int1, 1.0);
        assert!((ef1 - 2.5).abs() < 0.05);
        assert!(next1 > Utc::now());

        // Grade 5 (Perfect) on second review
        let (rep2, int2, ef2, _) =
            SpacedRepetitionSystem::calculate_next_interval(ReviewRating::Perfect, rep1, int1, ef1);
        assert_eq!(rep2, 2);
        assert_eq!(int2, 6.0);
        assert!(ef2 > ef1);

        // Grade 1 (Incorrect) on failure review -> reset
        let (rep3, int3, ef3, _) = SpacedRepetitionSystem::calculate_next_interval(
            ReviewRating::Incorrect,
            rep2,
            int2,
            ef2,
        );
        assert_eq!(rep3, 0);
        assert_eq!(int3, 1.0);
        assert!(ef3 >= 1.3);
    }

    #[test]
    fn test_pronunciation_assessment() {
        let reference = "ನಮಸ್ಕಾರ, ನೀವು ಹೇಗಿದ್ದೀರಿ?";
        let hypothesis = "ನಮಸ್ಕಾರ, ನೀವು ಹೇಗಿದ್ದೀರಿ?";

        let res = PronunciationEvaluator::assess(reference, hypothesis, LanguageCode::KnIn);
        assert!(res.is_passing);
        assert!(res.accuracy_score >= 95.0);
        assert_eq!(res.wer, 0.0);
        assert_eq!(res.cer, 0.0);

        let mismatch = "ನಮಸ್ಕಾರ, ಎಲ್ಲರೂ ಹೇಗಿದ್ದೀರಿ?";
        let res_diff = PronunciationEvaluator::assess(reference, mismatch, LanguageCode::KnIn);
        assert!(res_diff.accuracy_score < 95.0);
        assert!(!res_diff.phoneme_feedback.is_empty());
    }

    #[test]
    fn test_listening_challenges_and_grammar() {
        let kn_challenges = ListeningChallengeGenerator::get_challenges(LanguageCode::KnIn, 2);
        assert_eq!(kn_challenges.len(), 2);
        assert_eq!(kn_challenges[0].language, LanguageCode::KnIn);

        let ta_topics = GrammarKnowledgeBase::get_topics(LanguageCode::TaIn);
        assert!(!ta_topics.is_empty());
        assert_eq!(ta_topics[0].id, "ta-veetrumai");
    }
}
