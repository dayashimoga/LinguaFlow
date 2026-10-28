<script lang="ts">
  import { onMount } from 'svelte';
  import {
    assessPronunciation,
    getDeckCards,
    getGrammarTopics,
    getLearnerProgress,
    getListeningChallenges,
    listLearningDecks,
    reviewCard,
    synthesizeSpeech,
  } from '../lib/ipc';
  import type {
    GrammarGuideTopic,
    LanguageCode,
    LearnerProgress,
    LearningCard,
    LearningDeck,
    ListeningChallenge,
    PronunciationAssessmentResult,
    ReviewRating,
    ReviewResult,
  } from '../lib/types';

  type LearnTab = 'flashcards' | 'pronunciation' | 'listening' | 'grammar';
  let activeTab: LearnTab = 'flashcards';

  // ----------------------------------------------------
  // Shared / State: Learner Progress
  // ----------------------------------------------------
  let progress: LearnerProgress = {
    total_cards_studied: 0,
    cards_due_today: 0,
    cards_mastered: 0,
    current_streak_days: 0,
    retention_rate: 0,
    study_time_minutes: 0,
  };
  let isLoadingProgress = false;

  // ----------------------------------------------------
  // Tab 1: Spaced Repetition Flashcards
  // ----------------------------------------------------
  let decks: LearningDeck[] = [];
  let selectedDeckId = '';
  let cards: LearningCard[] = [];
  let currentCardIndex = 0;
  let isCardFlipped = false;
  let isLoadingDecks = false;
  let isLoadingCards = false;
  let reviewFeedback: ReviewResult | null = null;
  let isSpeaking = false;

  $: currentCard = cards.length > 0 && currentCardIndex < cards.length ? cards[currentCardIndex] : null;

  async function loadDecks() {
    isLoadingDecks = true;
    try {
      decks = await listLearningDecks();
      if (decks.length > 0 && !selectedDeckId) {
        selectedDeckId = decks[0].id;
        await loadCards(selectedDeckId);
      }
    } catch (e) {
      console.error('Failed to load learning decks:', e);
    } finally {
      isLoadingDecks = false;
    }
  }

  async function loadCards(deckId: string) {
    if (!deckId) return;
    isLoadingCards = true;
    isCardFlipped = false;
    currentCardIndex = 0;
    reviewFeedback = null;
    try {
      cards = await getDeckCards(deckId);
    } catch (e) {
      console.error('Failed to load flashcards:', e);
    } finally {
      isLoadingCards = false;
    }
  }

  function handleDeckChange(e: Event) {
    const target = e.target as HTMLSelectElement;
    selectedDeckId = target.value;
    loadCards(selectedDeckId);
  }

  function flipCard() {
    isCardFlipped = !isCardFlipped;
  }

  async function handleReview(rating: ReviewRating) {
    if (!currentCard) return;
    try {
      reviewFeedback = await reviewCard(currentCard.id, rating);
      await refreshProgress();
      // Move to next card after brief visual cue
      setTimeout(() => {
        isCardFlipped = false;
        reviewFeedback = null;
        if (currentCardIndex + 1 < cards.length) {
          currentCardIndex += 1;
        } else {
          // Wrap around or reload cards
          currentCardIndex = 0;
        }
      }, 700);
    } catch (e) {
      console.error('Failed to submit review:', e);
    }
  }

  async function playAudio(text: string, lang: LanguageCode) {
    if (isSpeaking || !text) return;
    isSpeaking = true;
    try {
      const resp = await synthesizeSpeech({
        text,
        language: lang,
        speed: 1.0,
        pitch: 1.0,
      });
      if (resp && resp.audio_samples.length > 0) {
        // Play synthesized PCM via Web Audio API
        const ctx = new (window.AudioContext || (window as any).webkitAudioContext)({
          sampleRate: resp.sample_rate || 22050,
        });
        const buffer = ctx.createBuffer(1, resp.audio_samples.length, resp.sample_rate || 22050);
        const channelData = buffer.getChannelData(0);
        for (let i = 0; i < resp.audio_samples.length; i++) {
          channelData[i] = resp.audio_samples[i] / 32768.0;
        }
        const src = ctx.createBufferSource();
        src.buffer = buffer;
        src.connect(ctx.destination);
        src.onended = () => {
          isSpeaking = false;
          ctx.close();
        };
        src.start();
      } else {
        isSpeaking = false;
      }
    } catch (e) {
      console.error('Audio playback error:', e);
      isSpeaking = false;
    }
  }

  // ----------------------------------------------------
  // Tab 2: Native Pronunciation Assessment
  // ----------------------------------------------------
  let pronReferenceText = 'ನಮಸ್ಕಾರ, ನೀವು ಹೇಗಿದ್ದೀರಿ?';
  let pronSpokenText = 'ನಮಸ್ಕಾರ, ನೀವು ಹೇಗಿದ್ದೀರಿ?';
  let pronLang: LanguageCode = 'kn-IN';
  let isAssessingPron = false;
  let pronResult: PronunciationAssessmentResult | null = null;

  const PRON_PRESETS: { label: string; text: string; lang: LanguageCode }[] = [
    { label: 'Kannada: Greeting', text: 'ನಮಸ್ಕಾರ, ನೀವು ಹೇಗಿದ್ದೀರಿ?', lang: 'kn-IN' },
    { label: 'Kannada: Water Request', text: 'ದಯವಿಟ್ಟು ನನಗೆ ನೀರು ಕೊಡಿ.', lang: 'kn-IN' },
    { label: 'Kannada: Gratitude', text: 'ನಿಮಗೆ ತುಂಬಾ ಧನ್ಯವಾದಗಳು.', lang: 'kn-IN' },
    { label: 'Tamil: Greeting', text: 'வணக்கம், நீங்கள் எப்படி இருக்கிறீர்கள்?', lang: 'ta-IN' },
    { label: 'Tamil: Water Request', text: 'தயவுசெய்து தண்ணீர் கொடுங்கள்.', lang: 'ta-IN' },
    { label: 'Tamil: Gratitude', text: 'மிக்க நன்றி, மீண்டும் சந்திப்போம்.', lang: 'ta-IN' },
  ];

  function selectPreset(preset: typeof PRON_PRESETS[0]) {
    pronReferenceText = preset.text;
    pronSpokenText = preset.text;
    pronLang = preset.lang;
    pronResult = null;
  }

  async function handleAssessPronunciation() {
    if (!pronReferenceText || !pronSpokenText) return;
    isAssessingPron = true;
    try {
      pronResult = await assessPronunciation({
        reference_text: pronReferenceText,
        spoken_transcript: pronSpokenText,
        language: pronLang,
      });
    } catch (e) {
      console.error('Pronunciation assessment failed:', e);
    } finally {
      isAssessingPron = false;
    }
  }

  // ----------------------------------------------------
  // Tab 3: Listening Comprehension Challenges
  // ----------------------------------------------------
  let quizLang: LanguageCode = 'kn-IN';
  let quizCount = 3;
  let challenges: ListeningChallenge[] = [];
  let currentQuizIndex = 0;
  let selectedOption: string | null = null;
  let isQuizSubmitted = false;
  let isLoadingQuiz = false;

  $: activeChallenge = challenges.length > 0 && currentQuizIndex < challenges.length ? challenges[currentQuizIndex] : null;

  async function loadQuizChallenges() {
    isLoadingQuiz = true;
    currentQuizIndex = 0;
    selectedOption = null;
    isQuizSubmitted = false;
    try {
      challenges = await getListeningChallenges(quizLang, quizCount);
    } catch (e) {
      console.error('Failed to load listening challenges:', e);
    } finally {
      isLoadingQuiz = false;
    }
  }

  function handleSelectOption(opt: string) {
    if (isQuizSubmitted) return;
    selectedOption = opt;
    isQuizSubmitted = true;
  }

  function nextQuizChallenge() {
    if (currentQuizIndex + 1 < challenges.length) {
      currentQuizIndex += 1;
      selectedOption = null;
      isQuizSubmitted = false;
    } else {
      // Reload challenges
      loadQuizChallenges();
    }
  }

  // ----------------------------------------------------
  // Tab 4: Dravidian Grammar Guide
  // ----------------------------------------------------
  let grammarLang: LanguageCode = 'kn-IN';
  let grammarTopics: GrammarGuideTopic[] = [];
  let selectedTopicId = '';
  let isLoadingGrammar = false;
  let grammarSearchQuery = '';

  $: activeTopic = grammarTopics.find((t) => t.id === selectedTopicId) || (grammarTopics.length > 0 ? grammarTopics[0] : null);

  async function loadGrammarTopics() {
    isLoadingGrammar = true;
    try {
      grammarTopics = await getGrammarTopics(grammarLang);
      if (grammarTopics.length > 0) {
        selectedTopicId = grammarTopics[0].id;
      }
    } catch (e) {
      console.error('Failed to load grammar topics:', e);
    } finally {
      isLoadingGrammar = false;
    }
  }

  // ----------------------------------------------------
  // Refresh Progress metrics
  // ----------------------------------------------------
  async function refreshProgress() {
    isLoadingProgress = true;
    try {
      progress = await getLearnerProgress();
    } catch (e) {
      console.error('Failed to refresh learner progress:', e);
    } finally {
      isLoadingProgress = false;
    }
  }

  onMount(async () => {
    await Promise.all([loadDecks(), refreshProgress(), loadQuizChallenges(), loadGrammarTopics()]);
  });
</script>

<div class="learn-container">
  <!-- Header & Progress Banner -->
  <header class="learn-header">
    <div class="title-area">
      <h2>Interactive Language Learning</h2>
      <p class="subtitle">
        SuperMemo-2 Spaced Repetition, Native Pronunciation Scoring & Dravidian Grammar Mastery
      </p>
    </div>

    <!-- Live Mastery Metrics Ribbon -->
    <div class="metrics-ribbon">
      <div class="metric-card">
        <span class="metric-val">{progress.total_cards_studied}</span>
        <span class="metric-lbl">Studied</span>
      </div>
      <div class="metric-card">
        <span class="metric-val highlight">{progress.cards_due_today}</span>
        <span class="metric-lbl">Due Today</span>
      </div>
      <div class="metric-card">
        <span class="metric-val success">{progress.cards_mastered}</span>
        <span class="metric-lbl">Mastered</span>
      </div>
      <div class="metric-card">
        <span class="metric-val streak">🔥 {progress.current_streak_days}d</span>
        <span class="metric-lbl">Streak</span>
      </div>
      <div class="metric-card">
        <span class="metric-val">{Math.round(progress.retention_rate * 100)}%</span>
        <span class="metric-lbl">Retention</span>
      </div>
    </div>
  </header>

  <!-- Navigation Tabs -->
  <nav class="learn-tabs" aria-label="Learning Modes">
    <button
      class="tab-btn"
      class:active={activeTab === 'flashcards'}
      on:click={() => (activeTab = 'flashcards')}
    >
      🗂️ Flashcards (SM-2)
    </button>
    <button
      class="tab-btn"
      class:active={activeTab === 'pronunciation'}
      on:click={() => (activeTab = 'pronunciation')}
    >
      🎙️ Pronunciation Scoring
    </button>
    <button
      class="tab-btn"
      class:active={activeTab === 'listening'}
      on:click={() => (activeTab = 'listening')}
    >
      🎧 Listening Comprehension
    </button>
    <button
      class="tab-btn"
      class:active={activeTab === 'grammar'}
      on:click={() => (activeTab = 'grammar')}
    >
      📖 Dravidian Grammar
    </button>
  </nav>

  <!-- Tab 1: Spaced Repetition Flashcards -->
  {#if activeTab === 'flashcards'}
    <div class="tab-pane flashcards-view">
      <div class="deck-toolbar">
        <label for="deck-select">Active Deck:</label>
        <select id="deck-select" value={selectedDeckId} on:change={handleDeckChange}>
          {#each decks as d}
            <option value={d.id}>{d.title} ({d.source_lang} → {d.target_lang}) [{d.total_cards} cards]</option>
          {/each}
        </select>
        <span class="deck-counter">
          Card {cards.length > 0 ? currentCardIndex + 1 : 0} of {cards.length}
        </span>
      </div>

      {#if isLoadingCards}
        <div class="loading-state">Loading flashcard deck...</div>
      {:else if currentCard}
        <!-- 3D Flip Card -->
        <div class="flip-card-container">
          <div
            class="flip-card"
            class:flipped={isCardFlipped}
            on:click={flipCard}
            role="button"
            tabindex="0"
            on:keydown={(e) => e.key === 'Enter' || e.key === ' ' ? flipCard() : null}
            aria-label="Flashcard. Click or press space to flip."
          >
            <!-- Card Front -->
            <div class="card-face card-front">
              <div class="face-badge">
                <span class="badge-lang">{currentCard.source_lang}</span>
                {#if currentCard.part_of_speech}
                  <span class="badge-pos">{currentCard.part_of_speech}</span>
                {/if}
              </div>

              <div class="term-display">
                <h3 class="unicode-term">{currentCard.source_term}</h3>
                {#if currentCard.romanization}
                  <p class="romanization">{currentCard.romanization}</p>
                {/if}
                {#if currentCard.ipa}
                  <p class="ipa">[{currentCard.ipa}]</p>
                {/if}
              </div>

              <div class="card-footer">
                <button
                  class="audio-btn"
                  on:click|stopPropagation={() => playAudio(currentCard.source_term, currentCard.source_lang)}
                  title="Listen to authentic pronunciation"
                  disabled={isSpeaking}
                >
                  🔊 {isSpeaking ? 'Playing...' : 'Pronounce'}
                </button>
                <span class="tap-hint">Click card to reveal translation & grammar</span>
              </div>
            </div>

            <!-- Card Back -->
            <div class="card-face card-back">
              <div class="face-badge">
                <span class="badge-lang">{currentCard.target_lang}</span>
                <span class="badge-interval">Interval: {currentCard.interval_days}d (EF: {currentCard.ease_factor.toFixed(2)})</span>
              </div>

              <div class="term-display">
                <h3 class="target-term">{currentCard.target_term}</h3>
                {#if currentCard.example_source}
                  <div class="example-box">
                    <p class="ex-src">“{currentCard.example_source}”</p>
                    <p class="ex-tgt">“{currentCard.example_target}”</p>
                  </div>
                {/if}
                {#if currentCard.grammar_notes}
                  <div class="grammar-note">
                    <strong>Grammar:</strong> {currentCard.grammar_notes}
                  </div>
                {/if}
              </div>

              <div class="card-footer">
                <button
                  class="audio-btn"
                  on:click|stopPropagation={() => playAudio(currentCard.target_term, currentCard.target_lang)}
                  title="Listen to target phrase"
                  disabled={isSpeaking}
                >
                  🔊 Pronounce
                </button>
                <span class="tap-hint">Rate recall below to advance interval</span>
              </div>
            </div>
          </div>
        </div>

        <!-- SM-2 Rating Controls -->
        <div class="rating-bar">
          <p class="rating-prompt">How well did you recall this term?</p>
          <div class="rating-buttons">
            <button class="rate-btn r0" on:click={() => handleReview(0)}>
              <span class="r-num">0</span>
              <span class="r-label">Blackout</span>
            </button>
            <button class="rate-btn r1" on:click={() => handleReview(1)}>
              <span class="r-num">1</span>
              <span class="r-label">Wrong</span>
            </button>
            <button class="rate-btn r2" on:click={() => handleReview(2)}>
              <span class="r-num">2</span>
              <span class="r-label">Hard</span>
            </button>
            <button class="rate-btn r3" on:click={() => handleReview(3)}>
              <span class="r-num">3</span>
              <span class="r-label">Good</span>
            </button>
            <button class="rate-btn r4" on:click={() => handleReview(4)}>
              <span class="r-num">4</span>
              <span class="r-label">Easy</span>
            </button>
            <button class="rate-btn r5" on:click={() => handleReview(5)}>
              <span class="r-num">5</span>
              <span class="r-label">Perfect</span>
            </button>
          </div>
        </div>

        {#if reviewFeedback}
          <div class="review-feedback-toast">
            [OK] Interval updated: <strong>{reviewFeedback.new_interval_days} day(s)</strong> | EF: <strong>{reviewFeedback.new_ease_factor.toFixed(2)}</strong>
          </div>
        {/if}
      {:else}
        <div class="empty-state">No flashcards found in this deck.</div>
      {/if}
    </div>
  {/if}

  <!-- Tab 2: Pronunciation Scoring -->
  {#if activeTab === 'pronunciation'}
    <div class="tab-pane pronunciation-view">
      <div class="panel-card">
        <h3>Native Pronunciation Evaluation</h3>
        <p class="panel-desc">
          Evaluates accent fidelity, word alignment (WER), and character precision (CER & chrF)
          against verified native Dravidian phonetics.
        </p>

        <!-- Presets Ribbon -->
        <div class="presets-row">
          <span class="presets-lbl">Quick Presets:</span>
          {#each PRON_PRESETS as p}
            <button class="preset-chip" on:click={() => selectPreset(p)}>
              {p.label}
            </button>
          {/each}
        </div>

        <div class="pron-inputs-grid">
          <div class="input-col">
            <label for="pron-ref">Reference Target Phrase ({pronLang}):</label>
            <textarea id="pron-ref" bind:value={pronReferenceText} rows="3"></textarea>
            <button
              class="audio-preview-btn"
              on:click={() => playAudio(pronReferenceText, pronLang)}
              disabled={isSpeaking}
            >
              🔊 Play Reference Audio
            </button>
          </div>

          <div class="input-col">
            <label for="pron-spoken">Spoken Input / Recognized Speech:</label>
            <textarea id="pron-spoken" bind:value={pronSpokenText} rows="3"></textarea>
            <button
              class="action-btn primary"
              on:click={handleAssessPronunciation}
              disabled={isAssessingPron}
            >
              {isAssessingPron ? 'Analyzing Phonetics...' : '🎙️ Evaluate Pronunciation'}
            </button>
          </div>
        </div>

        {#if pronResult}
          <div class="pron-results-dashboard">
            <div class="score-card" class:pass={pronResult.is_passing} class:fail={!pronResult.is_passing}>
              <div class="score-circle">
                <span class="score-num">{Math.round(pronResult.accuracy_score)}%</span>
                <span class="score-sub">Accuracy</span>
              </div>
              <div class="score-meta">
                <h4>{pronResult.is_passing ? '✓ Native-Level Pass' : '⚠ Needs Phonetic Practice'}</h4>
                <div class="metrics-grid">
                  <div class="metric-pill">chrF: <strong>{pronResult.chrf.toFixed(2)}</strong></div>
                  <div class="metric-pill">WER: <strong>{(pronResult.wer * 100).toFixed(1)}%</strong></div>
                  <div class="metric-pill">CER: <strong>{(pronResult.cer * 100).toFixed(1)}%</strong></div>
                </div>
              </div>
            </div>

            {#if pronResult.phoneme_feedback.length > 0}
              <div class="feedback-section">
                <h4>Phoneme & Syllable Recommendations:</h4>
                <div class="feedback-list">
                  {#each pronResult.phoneme_feedback as fb}
                    <div class="feedback-item">
                      <span class="fb-icon">💡</span>
                      <span class="fb-text">{fb}</span>
                    </div>
                  {/each}
                </div>
              </div>
            {/if}
          </div>
        {/if}
      </div>
    </div>
  {/if}

  <!-- Tab 3: Listening Comprehension Challenges -->
  {#if activeTab === 'listening'}
    <div class="tab-pane listening-view">
      <div class="quiz-toolbar">
        <label for="quiz-lang-select">Language:</label>
        <select id="quiz-lang-select" bind:value={quizLang} on:change={loadQuizChallenges}>
          <option value="kn-IN">Kannada (ಕನ್ನಡ)</option>
          <option value="ta-IN">Tamil (தமிழ்)</option>
          <option value="en-US">English (ಆಂಗ್ಲ)</option>
        </select>
        <span class="quiz-counter">
          Challenge {challenges.length > 0 ? currentQuizIndex + 1 : 0} of {challenges.length}
        </span>
      </div>

      {#if isLoadingQuiz}
        <div class="loading-state">Generating authentic listening challenges...</div>
      {:else if activeChallenge}
        <div class="quiz-card">
          <div class="quiz-header">
            <h4>Listen carefully to the audio and select the accurate meaning:</h4>
            <div class="audio-player-area">
              <button
                class="big-audio-btn"
                on:click={() => playAudio(activeChallenge.prompt_audio_text, activeChallenge.language)}
                disabled={isSpeaking}
              >
                🔊 {isSpeaking ? 'Playing Audio...' : 'Play Spoken Phrase'}
              </button>
              <span class="phrase-preview">“{activeChallenge.prompt_audio_text}”</span>
            </div>
          </div>

          <div class="options-grid">
            {#each activeChallenge.options as opt, idx}
              <button
                class="option-btn"
                class:selected={selectedOption === opt}
                class:correct={isQuizSubmitted && opt === activeChallenge.correct_answer}
                class:wrong={isQuizSubmitted && selectedOption === opt && opt !== activeChallenge.correct_answer}
                on:click={() => handleSelectOption(opt)}
                disabled={isQuizSubmitted}
              >
                <span class="opt-index">{String.fromCharCode(65 + idx)}</span>
                <span class="opt-text">{opt}</span>
              </button>
            {/each}
          </div>

          {#if isQuizSubmitted}
            <div class="explanation-box" class:success={selectedOption === activeChallenge.correct_answer}>
              <div class="exp-title">
                {selectedOption === activeChallenge.correct_answer ? '🎉 Excellent! Correct Answer.' : '❌ Not quite right.'}
              </div>
              <p class="exp-desc">{activeChallenge.explanation}</p>
              <button class="action-btn next-btn" on:click={nextQuizChallenge}>
                {currentQuizIndex + 1 < challenges.length ? 'Next Challenge →' : 'Complete Quiz'}
              </button>
            </div>
          {/if}
        </div>
      {/if}
    </div>
  {/if}

  <!-- Tab 4: Dravidian Grammar Guide -->
  {#if activeTab === 'grammar'}
    <div class="tab-pane grammar-view">
      <div class="grammar-toolbar">
        <label for="grammar-lang">Language Guide:</label>
        <select id="grammar-lang" bind:value={grammarLang} on:change={loadGrammarTopics}>
          <option value="kn-IN">Kannada (ವಿಭಕ್ತಿ ಪ್ರತ್ಯಯಗಳು)</option>
          <option value="ta-IN">Tamil (வேற்றுமை உருபுகள்)</option>
        </select>
        <input
          type="text"
          class="search-input"
          placeholder="Search case markers, vibhakti, or suffixes..."
          bind:value={grammarSearchQuery}
        />
      </div>

      {#if isLoadingGrammar}
        <div class="loading-state">Loading grammatical guides...</div>
      {:else}
        <div class="grammar-content-layout">
          <!-- Sidebar: Topics List -->
          <div class="topics-sidebar">
            {#each grammarTopics.filter((t) => !grammarSearchQuery || t.title.toLowerCase().includes(grammarSearchQuery.toLowerCase()) || t.content_markdown.toLowerCase().includes(grammarSearchQuery.toLowerCase())) as topic}
              <button
                class="topic-item"
                class:active={topic.id === selectedTopicId}
                on:click={() => (selectedTopicId = topic.id)}
              >
                <div class="topic-title">{topic.title}</div>
                <div class="topic-cat">{topic.category}</div>
              </button>
            {/each}
          </div>

          <!-- Active Topic Content -->
          {#if activeTopic}
            <div class="topic-body">
              <div class="topic-header">
                <h3>{activeTopic.title}</h3>
                <span class="cat-pill">{activeTopic.category}</span>
              </div>

              <div class="markdown-prose">
                <pre class="prose-pre">{activeTopic.content_markdown}</pre>
              </div>

              {#if activeTopic.examples.length > 0}
                <div class="examples-card">
                  <h4>Contextual Sentence Examples:</h4>
                  <ul class="ex-list">
                    {#each activeTopic.examples as ex}
                      <li class="ex-item">
                        <span class="ex-bullet">🔹</span>
                        <span class="ex-body">{ex}</span>
                      </li>
                    {/each}
                  </ul>
                </div>
              {/if}
            </div>
          {/if}
        </div>
      {/if}
    </div>
  {/if}
</div>

<style>
  .learn-container {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    padding: 1.5rem;
    max-width: 1200px;
    margin: 0 auto;
    font-family: inherit;
    color: var(--text-color, #f1f5f9);
  }

  .learn-header {
    display: flex;
    flex-wrap: wrap;
    justify-content: space-between;
    align-items: center;
    gap: 1rem;
    padding-bottom: 1rem;
    border-bottom: 1px solid rgba(255, 255, 255, 0.1);
  }

  .title-area h2 {
    margin: 0 0 0.25rem 0;
    font-size: 1.75rem;
    font-weight: 700;
    letter-spacing: -0.02em;
    background: linear-gradient(135deg, #38bdf8, #818cf8);
    -webkit-background-clip: text;
    background-clip: text;
    -webkit-text-fill-color: transparent;
  }

  .subtitle {
    margin: 0;
    font-size: 0.9rem;
    color: var(--text-muted, #94a3b8);
  }

  .metrics-ribbon {
    display: flex;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .metric-card {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 0.5rem 0.85rem;
    background: rgba(30, 41, 59, 0.7);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 10px;
    min-width: 70px;
  }

  .metric-val {
    font-size: 1.15rem;
    font-weight: 700;
    color: #f8fafc;
  }

  .metric-val.highlight {
    color: #38bdf8;
  }

  .metric-val.success {
    color: #4ade80;
  }

  .metric-val.streak {
    color: #fb923c;
  }

  .metric-lbl {
    font-size: 0.7rem;
    text-transform: uppercase;
    color: #94a3b8;
    margin-top: 2px;
  }

  .learn-tabs {
    display: flex;
    gap: 0.5rem;
    overflow-x: auto;
    padding-bottom: 0.5rem;
  }

  .tab-btn {
    padding: 0.65rem 1.15rem;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(30, 41, 59, 0.6);
    color: #cbd5e1;
    font-weight: 600;
    font-size: 0.9rem;
    cursor: pointer;
    transition: all 0.2s ease;
    white-space: nowrap;
  }

  .tab-btn:hover {
    background: rgba(51, 65, 85, 0.8);
    color: #ffffff;
  }

  .tab-btn.active {
    background: linear-gradient(135deg, #2563eb, #4f46e5);
    color: #ffffff;
    border-color: #6366f1;
    box-shadow: 0 4px 12px rgba(79, 70, 229, 0.3);
  }

  /* Flashcards Tab */
  .flashcards-view {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1.5rem;
  }

  .deck-toolbar {
    display: flex;
    align-items: center;
    gap: 1rem;
    width: 100%;
    max-width: 600px;
  }

  .deck-toolbar select {
    flex: 1;
    padding: 0.5rem 0.75rem;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.15);
    background: #1e293b;
    color: #f8fafc;
    font-size: 0.9rem;
  }

  .deck-counter {
    font-size: 0.85rem;
    color: #94a3b8;
    white-space: nowrap;
  }

  /* 3D Flip Card */
  .flip-card-container {
    width: 100%;
    max-width: 600px;
    height: 340px;
    perspective: 1200px;
    cursor: pointer;
  }

  .flip-card {
    position: relative;
    width: 100%;
    height: 100%;
    transform-style: preserve-3d;
    transition: transform 0.6s cubic-bezier(0.4, 0.2, 0.2, 1);
    border-radius: 16px;
  }

  .flip-card.flipped {
    transform: rotateY(180deg);
  }

  .card-face {
    position: absolute;
    width: 100%;
    height: 100%;
    backface-visibility: hidden;
    border-radius: 16px;
    padding: 1.5rem;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    box-sizing: border-box;
    box-shadow: 0 8px 32px rgba(0, 0, 0, 0.35);
  }

  .card-front {
    background: linear-gradient(145deg, #1e293b, #0f172a);
    border: 1px solid rgba(56, 189, 248, 0.25);
  }

  .card-back {
    background: linear-gradient(145deg, #1e1b4b, #0f172a);
    border: 1px solid rgba(129, 140, 248, 0.3);
    transform: rotateY(180deg);
  }

  .face-badge {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .badge-lang {
    font-size: 0.75rem;
    font-weight: 700;
    text-transform: uppercase;
    padding: 0.2rem 0.5rem;
    border-radius: 6px;
    background: rgba(56, 189, 248, 0.15);
    color: #38bdf8;
  }

  .badge-pos {
    font-size: 0.75rem;
    padding: 0.2rem 0.5rem;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.08);
    color: #cbd5e1;
  }

  .badge-interval {
    font-size: 0.75rem;
    color: #a5b4fc;
  }

  .term-display {
    text-align: center;
    margin: auto 0;
  }

  .unicode-term {
    font-size: 2.2rem;
    font-weight: 700;
    margin: 0 0 0.5rem 0;
    color: #ffffff;
    line-height: 1.3;
  }

  .target-term {
    font-size: 2rem;
    font-weight: 700;
    margin: 0 0 0.75rem 0;
    color: #818cf8;
  }

  .romanization {
    font-size: 1.15rem;
    color: #94a3b8;
    margin: 0 0 0.25rem 0;
  }

  .ipa {
    font-size: 0.95rem;
    color: #64748b;
    font-family: monospace;
    margin: 0;
  }

  .example-box {
    background: rgba(0, 0, 0, 0.25);
    padding: 0.75rem;
    border-radius: 8px;
    margin-top: 0.5rem;
    text-align: left;
  }

  .ex-src {
    font-size: 0.9rem;
    color: #e2e8f0;
    margin: 0 0 0.25rem 0;
  }

  .ex-tgt {
    font-size: 0.85rem;
    color: #94a3b8;
    margin: 0;
  }

  .grammar-note {
    font-size: 0.85rem;
    color: #a5b4fc;
    margin-top: 0.5rem;
  }

  .card-footer {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .audio-btn {
    padding: 0.4rem 0.75rem;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.15);
    background: rgba(255, 255, 255, 0.05);
    color: #f1f5f9;
    font-size: 0.85rem;
    cursor: pointer;
    transition: all 0.2s;
  }

  .audio-btn:hover:not(:disabled) {
    background: rgba(56, 189, 248, 0.2);
    border-color: #38bdf8;
  }

  .tap-hint {
    font-size: 0.75rem;
    color: #64748b;
  }

  /* Rating Bar */
  .rating-bar {
    width: 100%;
    max-width: 600px;
    text-align: center;
  }

  .rating-prompt {
    font-size: 0.85rem;
    color: #94a3b8;
    margin-bottom: 0.75rem;
  }

  .rating-buttons {
    display: grid;
    grid-template-columns: repeat(6, 1fr);
    gap: 0.5rem;
  }

  .rate-btn {
    display: flex;
    flex-direction: column;
    align-items: center;
    padding: 0.65rem 0.25rem;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(30, 41, 59, 0.8);
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .rate-btn .r-num {
    font-size: 1.1rem;
    font-weight: 700;
  }

  .rate-btn .r-label {
    font-size: 0.65rem;
    text-transform: uppercase;
    margin-top: 2px;
  }

  .rate-btn.r0:hover { background: #dc2626; color: #fff; border-color: #ef4444; }
  .rate-btn.r1:hover { background: #ea580c; color: #fff; border-color: #f97316; }
  .rate-btn.r2:hover { background: #d97706; color: #fff; border-color: #f59e0b; }
  .rate-btn.r3:hover { background: #2563eb; color: #fff; border-color: #3b82f6; }
  .rate-btn.r4:hover { background: #059669; color: #fff; border-color: #10b981; }
  .rate-btn.r5:hover { background: #16a34a; color: #fff; border-color: #22c55e; }

  .review-feedback-toast {
    padding: 0.6rem 1rem;
    border-radius: 8px;
    background: rgba(16, 185, 129, 0.2);
    border: 1px solid #10b981;
    color: #ecfdf5;
    font-size: 0.85rem;
  }

  /* Pronunciation Scoring Tab */
  .panel-card {
    background: rgba(30, 41, 59, 0.6);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 14px;
    padding: 1.5rem;
  }

  .panel-card h3 {
    margin: 0 0 0.25rem 0;
    font-size: 1.35rem;
  }

  .panel-desc {
    color: #94a3b8;
    font-size: 0.85rem;
    margin: 0 0 1rem 0;
  }

  .presets-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    flex-wrap: wrap;
    margin-bottom: 1.25rem;
  }

  .presets-lbl {
    font-size: 0.8rem;
    color: #94a3b8;
  }

  .preset-chip {
    padding: 0.35rem 0.65rem;
    border-radius: 6px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(15, 23, 42, 0.6);
    color: #cbd5e1;
    font-size: 0.75rem;
    cursor: pointer;
    transition: all 0.15s;
  }

  .preset-chip:hover {
    background: rgba(56, 189, 248, 0.15);
    color: #38bdf8;
    border-color: #38bdf8;
  }

  .pron-inputs-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.25rem;
  }

  @media (max-width: 768px) {
    .pron-inputs-grid {
      grid-template-columns: 1fr;
    }
  }

  .input-col {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .input-col label {
    font-size: 0.85rem;
    font-weight: 600;
    color: #cbd5e1;
  }

  .input-col textarea {
    padding: 0.75rem;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.12);
    background: rgba(15, 23, 42, 0.8);
    color: #f8fafc;
    font-size: 1rem;
    resize: vertical;
    font-family: inherit;
  }

  .action-btn {
    padding: 0.65rem 1rem;
    border-radius: 8px;
    font-weight: 600;
    cursor: pointer;
    border: none;
    transition: all 0.2s;
  }

  .action-btn.primary {
    background: linear-gradient(135deg, #0284c7, #2563eb);
    color: #ffffff;
  }

  .action-btn.primary:hover:not(:disabled) {
    box-shadow: 0 4px 12px rgba(37, 99, 235, 0.4);
  }

  .audio-preview-btn {
    padding: 0.55rem 0.85rem;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.15);
    background: rgba(255, 255, 255, 0.05);
    color: #e2e8f0;
    font-size: 0.85rem;
    cursor: pointer;
  }

  .pron-results-dashboard {
    margin-top: 1.5rem;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .score-card {
    display: flex;
    align-items: center;
    gap: 1.5rem;
    padding: 1.25rem;
    border-radius: 12px;
    background: rgba(15, 23, 42, 0.8);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .score-card.pass {
    border-color: rgba(34, 197, 94, 0.4);
    background: linear-gradient(135deg, rgba(34, 197, 94, 0.1), rgba(15, 23, 42, 0.8));
  }

  .score-card.fail {
    border-color: rgba(239, 68, 68, 0.4);
    background: linear-gradient(135deg, rgba(239, 68, 68, 0.1), rgba(15, 23, 42, 0.8));
  }

  .score-circle {
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    width: 80px;
    height: 80px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.05);
    border: 3px solid currentColor;
  }

  .score-card.pass .score-circle { color: #22c55e; }
  .score-card.fail .score-circle { color: #ef4444; }

  .score-num {
    font-size: 1.4rem;
    font-weight: 800;
  }

  .score-sub {
    font-size: 0.6rem;
    text-transform: uppercase;
  }

  .score-meta h4 {
    margin: 0 0 0.5rem 0;
    font-size: 1.1rem;
  }

  .metrics-grid {
    display: flex;
    gap: 0.75rem;
  }

  .metric-pill {
    padding: 0.25rem 0.6rem;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.08);
    font-size: 0.8rem;
  }

  .feedback-section {
    background: rgba(15, 23, 42, 0.6);
    border-radius: 10px;
    padding: 1rem;
  }

  .feedback-section h4 {
    margin: 0 0 0.75rem 0;
    font-size: 0.95rem;
    color: #e2e8f0;
  }

  .feedback-list {
    display: flex;
    flex-direction: column;
    gap: 0.4rem;
  }

  .feedback-item {
    display: flex;
    align-items: center;
    gap: 0.5rem;
    font-size: 0.85rem;
    color: #cbd5e1;
  }

  /* Listening Challenge Quiz Tab */
  .quiz-toolbar {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .quiz-toolbar select {
    padding: 0.5rem 0.75rem;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.15);
    background: #1e293b;
    color: #f8fafc;
    font-size: 0.9rem;
  }

  .quiz-counter {
    color: #94a3b8;
    font-size: 0.85rem;
  }

  .quiz-card {
    background: rgba(30, 41, 59, 0.6);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 14px;
    padding: 1.5rem;
  }

  .quiz-header h4 {
    margin: 0 0 1rem 0;
    font-size: 1.1rem;
    color: #cbd5e1;
  }

  .audio-player-area {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1.5rem;
    padding: 0.75rem 1rem;
    background: rgba(15, 23, 42, 0.6);
    border-radius: 10px;
  }

  .big-audio-btn {
    padding: 0.6rem 1.15rem;
    border-radius: 8px;
    border: none;
    background: linear-gradient(135deg, #4f46e5, #7c3aed);
    color: #fff;
    font-weight: 600;
    cursor: pointer;
  }

  .phrase-preview {
    font-size: 1.15rem;
    color: #38bdf8;
    font-weight: 600;
  }

  .options-grid {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.75rem;
  }

  @media (max-width: 640px) {
    .options-grid {
      grid-template-columns: 1fr;
    }
  }

  .option-btn {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem;
    border-radius: 10px;
    border: 1px solid rgba(255, 255, 255, 0.1);
    background: rgba(15, 23, 42, 0.7);
    color: #f1f5f9;
    text-align: left;
    cursor: pointer;
    transition: all 0.2s;
  }

  .option-btn:hover:not(:disabled) {
    border-color: #6366f1;
    background: rgba(30, 41, 59, 0.9);
  }

  .opt-index {
    font-weight: 700;
    font-size: 0.9rem;
    width: 24px;
    height: 24px;
    border-radius: 50%;
    background: rgba(255, 255, 255, 0.1);
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .opt-text {
    flex: 1;
    font-size: 0.95rem;
  }

  .option-btn.correct {
    background: rgba(34, 197, 94, 0.2);
    border-color: #22c55e;
    color: #4ade80;
  }

  .option-btn.wrong {
    background: rgba(239, 68, 68, 0.2);
    border-color: #ef4444;
    color: #f87171;
  }

  .explanation-box {
    margin-top: 1.25rem;
    padding: 1rem;
    border-radius: 10px;
    background: rgba(15, 23, 42, 0.8);
    border: 1px solid rgba(255, 255, 255, 0.1);
  }

  .explanation-box.success {
    border-color: rgba(34, 197, 94, 0.3);
  }

  .exp-title {
    font-weight: 700;
    font-size: 1rem;
    margin-bottom: 0.4rem;
  }

  .exp-desc {
    font-size: 0.9rem;
    color: #cbd5e1;
    margin: 0 0 1rem 0;
  }

  .next-btn {
    background: #4f46e5;
    color: #fff;
  }

  /* Grammar Guide Tab */
  .grammar-toolbar {
    display: flex;
    align-items: center;
    gap: 1rem;
    margin-bottom: 1rem;
  }

  .grammar-toolbar select, .grammar-toolbar input {
    padding: 0.5rem 0.75rem;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.15);
    background: #1e293b;
    color: #f8fafc;
    font-size: 0.9rem;
  }

  .search-input {
    flex: 1;
  }

  .grammar-content-layout {
    display: grid;
    grid-template-columns: 280px 1fr;
    gap: 1.25rem;
  }

  @media (max-width: 768px) {
    .grammar-content-layout {
      grid-template-columns: 1fr;
    }
  }

  .topics-sidebar {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .topic-item {
    text-align: left;
    padding: 0.75rem;
    border-radius: 8px;
    border: 1px solid rgba(255, 255, 255, 0.08);
    background: rgba(30, 41, 59, 0.6);
    color: #cbd5e1;
    cursor: pointer;
    transition: all 0.15s;
  }

  .topic-item:hover {
    background: rgba(51, 65, 85, 0.8);
  }

  .topic-item.active {
    background: rgba(79, 70, 229, 0.2);
    border-color: #6366f1;
    color: #ffffff;
  }

  .topic-title {
    font-weight: 600;
    font-size: 0.85rem;
  }

  .topic-cat {
    font-size: 0.7rem;
    color: #94a3b8;
    margin-top: 2px;
  }

  .topic-body {
    background: rgba(30, 41, 59, 0.6);
    border: 1px solid rgba(255, 255, 255, 0.08);
    border-radius: 12px;
    padding: 1.5rem;
  }

  .topic-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    border-bottom: 1px solid rgba(255, 255, 255, 0.08);
    padding-bottom: 0.75rem;
    margin-bottom: 1rem;
  }

  .topic-header h3 {
    margin: 0;
    font-size: 1.2rem;
  }

  .cat-pill {
    padding: 0.2rem 0.6rem;
    border-radius: 6px;
    background: rgba(255, 255, 255, 0.08);
    font-size: 0.75rem;
    color: #38bdf8;
  }

  .prose-pre {
    white-space: pre-wrap;
    font-family: inherit;
    font-size: 0.9rem;
    line-height: 1.6;
    color: #cbd5e1;
    margin: 0;
  }

  .examples-card {
    margin-top: 1.5rem;
    padding: 1rem;
    border-radius: 8px;
    background: rgba(15, 23, 42, 0.6);
  }

  .examples-card h4 {
    margin: 0 0 0.5rem 0;
    font-size: 0.95rem;
    color: #f1f5f9;
  }

  .ex-list {
    list-style: none;
    padding: 0;
    margin: 0;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .ex-item {
    display: flex;
    gap: 0.5rem;
    font-size: 0.85rem;
    color: #cbd5e1;
  }

  .loading-state, .empty-state {
    padding: 3rem;
    text-align: center;
    color: #94a3b8;
    font-size: 0.95rem;
  }
</style>
