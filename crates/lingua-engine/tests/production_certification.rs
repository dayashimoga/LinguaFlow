use chrono::Utc;
use lingua_core::{LanguageCode, ReviewRating};
use lingua_engine::{
    GrammarKnowledgeBase, ListeningChallengeGenerator, PronunciationEvaluator, SecurityAuditor,
    SpacedRepetitionSystem,
};

#[test]
fn test_sm2_spaced_repetition_progression() {
    // 1. Initial review: rating Easy (4)
    let (rep1, int1, ef1, next1) =
        SpacedRepetitionSystem::calculate_next_interval(ReviewRating::Easy, 0, 1.0, 2.5);
    assert_eq!(rep1, 1);
    assert_eq!(int1, 1.0);
    assert!((ef1 - 2.5).abs() < 0.05);
    assert!(next1 > Utc::now());

    // 2. Second review: rating Perfect (5)
    let (rep2, int2, ef2, next2) =
        SpacedRepetitionSystem::calculate_next_interval(ReviewRating::Perfect, rep1, int1, ef1);
    assert_eq!(rep2, 2);
    assert_eq!(int2, 6.0);
    assert!(ef2 > ef1); // Ease factor increases on easy/perfect recall
    assert!(next2 > next1);

    // 3. Third review: rating Good (3) -> interval = round(6.0 * ef2)
    let (rep3, int3, ef3, _) =
        SpacedRepetitionSystem::calculate_next_interval(ReviewRating::Good, rep2, int2, ef2);
    assert_eq!(rep3, 3);
    assert!(int3 >= 15.0); // 6 * ~2.6 = ~16 days
    assert!(ef3 >= 1.3);

    // 4. Failure: rating Incorrect (1) -> resets repetition to 0, interval to 1.0 day
    let (rep4, int4, ef4, _) =
        SpacedRepetitionSystem::calculate_next_interval(ReviewRating::Incorrect, rep3, int3, ef3);
    assert_eq!(rep4, 0);
    assert_eq!(int4, 1.0);
    assert!(ef4 >= 1.3);
}

#[test]
fn test_pronunciation_scoring_accuracy_and_feedback() {
    let target_kn = "ನಮಸ್ಕಾರ, ನೀವು ಹೇಗಿದ್ದೀರಿ?";
    let perfect_kn = "ನಮಸ್ಕಾರ, ನೀವು ಹೇಗಿದ್ದೀರಿ?";

    let result_perfect = PronunciationEvaluator::assess(target_kn, perfect_kn, LanguageCode::KnIn);
    assert!(result_perfect.is_passing);
    assert!(result_perfect.accuracy_score >= 95.0);
    assert_eq!(result_perfect.wer, 0.0);
    assert_eq!(result_perfect.cer, 0.0);
    assert!(result_perfect.chrf >= 99.0);

    // Partial deviation in word order and wording
    let imperfect_kn = "ನಮಸ್ಕಾರ, ಎಲ್ಲರೂ ಹೇಗಿದ್ದೀರಿ?";
    let result_imperfect =
        PronunciationEvaluator::assess(target_kn, imperfect_kn, LanguageCode::KnIn);
    assert!(result_imperfect.accuracy_score < 95.0);
    assert!(result_imperfect.accuracy_score > 40.0);
    assert!(result_imperfect.wer > 0.0);
    assert!(!result_imperfect.phoneme_feedback.is_empty());

    // Empty speech transcript
    let result_empty = PronunciationEvaluator::assess(target_kn, "", LanguageCode::KnIn);
    assert!(!result_empty.is_passing);
    assert_eq!(result_empty.accuracy_score, 0.0);
}

#[test]
fn test_listening_challenges_and_distractors() {
    let kn_quizzes = ListeningChallengeGenerator::get_challenges(LanguageCode::KnIn, 3);
    assert_eq!(kn_quizzes.len(), 3);
    for q in &kn_quizzes {
        assert_eq!(q.language, LanguageCode::KnIn);
        assert_eq!(q.options.len(), 4);
        assert!(q.options.contains(&q.correct_answer));
        assert!(!q.explanation.is_empty());
    }

    let ta_quizzes = ListeningChallengeGenerator::get_challenges(LanguageCode::TaIn, 3);
    assert_eq!(ta_quizzes.len(), 3);
    for q in &ta_quizzes {
        assert_eq!(q.language, LanguageCode::TaIn);
        assert!(q.options.contains(&q.correct_answer));
    }
}

#[test]
fn test_grammar_knowledge_base_authenticity() {
    let kn_topics = GrammarKnowledgeBase::get_topics(LanguageCode::KnIn);
    assert!(!kn_topics.is_empty());

    let vibhakti = kn_topics.iter().find(|t| t.id == "kn-vibhakti").unwrap();
    assert!(vibhakti.content_markdown.contains("ಪ್ರಥಮಾ"));
    assert!(vibhakti.content_markdown.contains("ದ್ವಿತೀಯಾ"));
    assert!(vibhakti.content_markdown.contains("ಷಷ್ಠೀ"));
    assert!(!vibhakti.examples.is_empty());

    let ta_topics = GrammarKnowledgeBase::get_topics(LanguageCode::TaIn);
    let veetrumai = ta_topics.iter().find(|t| t.id == "ta-veetrumai").unwrap();
    assert!(veetrumai.content_markdown.contains("இரண்டாம் வேற்றுமை"));
    assert!(veetrumai.content_markdown.contains("நான்காம் வேற்றுமை"));
    assert!(!veetrumai.examples.is_empty());
}

#[test]
fn test_zero_log_transcript_scrubbing_and_sbom() {
    let raw = "Error processing request for user daya with token Bearer secret_session_token_xyz and api_key=top_secret_12345 in transcript: \"private speech here\"";
    let scrubbed = SecurityAuditor::scrub_log_message(raw);

    assert!(!scrubbed.contains("secret_session_token_xyz"));
    assert!(!scrubbed.contains("top_secret_12345"));
    assert!(!scrubbed.contains("private speech here"));
    assert!(scrubbed.contains("[REDACTED_TOKEN]"));
    assert!(scrubbed.contains("[REDACTED_CREDENTIAL]"));
    assert!(scrubbed.contains("[REDACTED_TRANSCRIPT]"));

    let sbom = SecurityAuditor::generate_sbom();
    assert!(sbom.total_components >= 7);
    assert!(SecurityAuditor::audit_licenses().is_ok());

    let statuses = SecurityAuditor::get_platform_release_status();
    assert_eq!(statuses.len(), 4);
}

#[test]
fn test_chaos_reliability_stress_suite() {
    let completed = SecurityAuditor::run_chaos_stress_simulation(500).expect("chaos failed");
    assert_eq!(completed, 500);
}
