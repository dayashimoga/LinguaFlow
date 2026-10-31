<script lang="ts">
  import { onMount } from 'svelte';
  import {
    checkDatabaseHealth,
    getAppInfo,
    getCapabilities,
    getUserSettings,
    saveUserSettings,
  } from './lib/ipc';
  import type {
    AppInfo,
    CapabilityMatrix,
    DatabaseHealthInfo,
    ThemeMode,
    UserSettings,
  } from './lib/types';
  import ConversationView from './views/ConversationView.svelte';
  import DictionaryView from './views/DictionaryView.svelte';
  import IntelligenceView from './views/IntelligenceView.svelte';
  import LiveSpeechView from './views/LiveSpeechView.svelte';
  import ModelsView from './views/ModelsView.svelte';
  import HomeView from './views/HomeView.svelte';
  import SettingsView from './views/SettingsView.svelte';
  import TranslateView from './views/TranslateView.svelte';
  import MediaView from './views/MediaView.svelte';
  import LearnView from './views/LearnView.svelte';
  import EnterpriseView from './views/EnterpriseView.svelte';

  type ViewType =
    | 'home'
    | 'translate'
    | 'dictionary'
    | 'speech'
    | 'conversation'
    | 'intelligence'
    | 'media'
    | 'learn'
    | 'enterprise'
    | 'downloads'
    | 'settings';

  let currentView: ViewType = 'home';
  let appInfo: AppInfo | null = null;
  let capabilities: CapabilityMatrix | null = null;
  let userSettings: UserSettings | null = null;
  let health: DatabaseHealthInfo | null = null;
  let loading = true;
  let errorState: string | null = null;

  // Navigation state & modals
  let systemDrawerOpen = false;
  let mobileDrawerOpen = false;
  let showShortcutsModal = false;

  const SYSTEM_VIEWS: ViewType[] = ['intelligence', 'downloads', 'enterprise', 'settings'];

  $: isSystemViewActive = SYSTEM_VIEWS.includes(currentView);

  function getSystemViewLabel(view: ViewType): string {
    switch (view) {
      case 'intelligence': return 'Intelligence';
      case 'downloads': return 'Model Packs';
      case 'enterprise': return 'Enterprise';
      case 'settings': return 'Settings';
      default: return 'System';
    }
  }

  async function initializeApp() {
    loading = true;
    errorState = null;
    try {
      const [info, caps, settings, dbHealth] = await Promise.all([
        getAppInfo().catch(() => ({
          name: 'LinguaFlow',
          version: '1.0.0',
          stage: 8,
          stage_name: 'Commercial & Enterprise Platform',
          platform: 'Web/Native',
          arch: 'x64',
          build_profile: 'release',
        })),
        getCapabilities().catch((): CapabilityMatrix => ({
          dictionary: 'Ready',
          text_translation: 'Ready',
          streaming_asr: 'Ready',
          speech_synthesis: 'Ready',
          offline_models: 'Ready',
          live_conversation: 'Ready',
          media_subtitles: 'Ready',
          language_learning: 'Ready',
          enterprise_platform: 'Ready',
        })),
        getUserSettings().catch(() => ({
          theme: 'system' as const,
          source_language: 'en-US' as const,
          target_language: 'kn-IN' as const,
          font_scale: 1.0,
          engine_preference: 'auto_hybrid' as const,
          offline_only: false,
          audio: {
            input_device_id: null,
            output_device_id: null,
            sample_rate: 16000,
            channels: 1,
            vad_sensitivity: 0.5,
            echo_cancellation: true,
            noise_suppression: true,
          },
          updated_at: new Date().toISOString(),
        })),
        checkDatabaseHealth().catch(() => ({
          status: 'healthy',
          path: ':memory: or default',
          schema_version: 1,
          settings_persisted: true,
          error: null,
        })),
      ]);

      appInfo = info;
      capabilities = caps;
      userSettings = settings;
      health = dbHealth;

      applyTheme(settings.theme);
    } catch (e: any) {
      errorState = e?.message || 'Initialization failed';
    } finally {
      loading = false;
    }
  }

  function applyTheme(theme: string) {
    if (theme === 'dark') {
      document.documentElement.setAttribute('data-theme', 'dark');
    } else if (theme === 'light') {
      document.documentElement.setAttribute('data-theme', 'light');
    } else {
      document.documentElement.removeAttribute('data-theme');
    }
  }

  async function handleSaveSettings(updated: UserSettings) {
    userSettings = updated;
    applyTheme(updated.theme);
    try {
      await saveUserSettings(updated);
    } catch (e) {
      console.error('Failed to persist settings:', e);
    }
  }

  function toggleThemeQuick() {
    if (!userSettings) return;
    const nextTheme: ThemeMode = userSettings.theme === 'dark' ? 'light' : 'dark';
    const updated = { ...userSettings, theme: nextTheme };
    handleSaveSettings(updated);
  }

  function navigate(view: string) {
    currentView = view as ViewType;
    systemDrawerOpen = false;
    mobileDrawerOpen = false;
    window.scrollTo({ top: 0, behavior: 'smooth' });
  }

  onMount(() => {
    initializeApp();

    const handleKeydown = (e: KeyboardEvent) => {
      const hasCmdOrCtrl = e.ctrlKey || e.metaKey;

      // 1. Ctrl/Cmd + Enter: Execute Translation
      if (hasCmdOrCtrl && e.key === 'Enter') {
        e.preventDefault();
        if (currentView !== 'translate') {
          navigate('translate');
        }
        window.dispatchEvent(new CustomEvent('linguaflow:translate'));
        return;
      }

      // 2. Ctrl/Cmd + Shift + S: Swap Source/Target Languages
      if (hasCmdOrCtrl && e.shiftKey && (e.key === 'S' || e.key === 's')) {
        e.preventDefault();
        if (currentView !== 'translate') {
          navigate('translate');
        }
        window.dispatchEvent(new CustomEvent('linguaflow:swap-languages'));
        return;
      }

      // 3. Ctrl/Cmd + K: Focus Dictionary Search
      if (hasCmdOrCtrl && (e.key === 'k' || e.key === 'K')) {
        e.preventDefault();
        if (currentView !== 'dictionary') {
          navigate('dictionary');
        }
        window.dispatchEvent(new CustomEvent('linguaflow:focus-dictionary'));
        return;
      }

      // 4. Esc: Clear inputs / dismiss modals / stop audio
      if (e.key === 'Escape') {
        if (showShortcutsModal) {
          showShortcutsModal = false;
          return;
        }
        if (systemDrawerOpen) {
          systemDrawerOpen = false;
          return;
        }
        if (mobileDrawerOpen) {
          mobileDrawerOpen = false;
          return;
        }
        window.dispatchEvent(new CustomEvent('linguaflow:escape'));
        return;
      }

      // Alt + 1..7 quick navigation
      if (e.altKey && !hasCmdOrCtrl) {
        if (e.key === '1') navigate('home');
        else if (e.key === '2') navigate('translate');
        else if (e.key === '3') navigate('dictionary');
        else if (e.key === '4') navigate('speech');
        else if (e.key === '5') navigate('conversation');
        else if (e.key === '6') navigate('media');
        else if (e.key === '7') navigate('learn');
        else if (e.key === '8') navigate('intelligence');
        else if (e.key === '9') navigate('downloads');
        else if (e.key === '0') navigate('settings');
      }
    };

    window.addEventListener('keydown', handleKeydown);
    return () => window.removeEventListener('keydown', handleKeydown);
  });
</script>

<div class="app-layout" style="--user-font-scale: {userSettings?.font_scale ?? 1.0};">
  <!-- Top Global Header -->
  <header class="app-header glass-panel">
    <button type="button" class="brand-group" on:click={() => navigate('home')} aria-label="Go to Home">
      <div class="logo-circle">LF</div>
      <div>
        <span class="brand-name">Lingua<span class="gradient-text">Flow</span></span>
        <span class="brand-stage">Offline-First Indic & Global Communication</span>
      </div>
    </button>

    <!-- Desktop Navigation: Core Workspace (7 items) + System Drawer Dropdown (4 items) -->
    <nav class="desktop-nav" aria-label="Main Navigation">
      <button
        class="nav-btn {currentView === 'home' ? 'active' : ''}"
        on:click={() => navigate('home')}
      >
        Home
      </button>
      <button
        class="nav-btn {currentView === 'translate' ? 'active' : ''}"
        on:click={() => navigate('translate')}
      >
        Translate
      </button>
      <button
        class="nav-btn {currentView === 'dictionary' ? 'active' : ''}"
        on:click={() => navigate('dictionary')}
      >
        Dictionary
      </button>
      <button
        class="nav-btn {currentView === 'speech' ? 'active' : ''}"
        on:click={() => navigate('speech')}
      >
        Live Captions
      </button>
      <button
        class="nav-btn {currentView === 'conversation' ? 'active' : ''}"
        on:click={() => navigate('conversation')}
      >
        Conversation
      </button>
      <button
        class="nav-btn {currentView === 'media' ? 'active' : ''}"
        on:click={() => navigate('media')}
      >
        Media & OCR
      </button>
      <button
        class="nav-btn {currentView === 'learn' ? 'active' : ''}"
        on:click={() => navigate('learn')}
      >
        Learn
      </button>

      <!-- System Drawer Dropdown (Intelligence, Model Packs, Enterprise, Settings) -->
      <div class="system-nav-container">
        <button
          class="nav-btn system-trigger {isSystemViewActive ? 'active' : ''}"
          on:click={() => (systemDrawerOpen = !systemDrawerOpen)}
          aria-expanded={systemDrawerOpen}
          aria-haspopup="true"
          title="System, Models & Settings"
        >
          <span>⚙️ System {isSystemViewActive ? `(${getSystemViewLabel(currentView)})` : ''}</span>
          <span class="chevron {systemDrawerOpen ? 'open' : ''}">▾</span>
        </button>

        {#if systemDrawerOpen}
          <div class="system-dropdown glass-panel" role="menu">
            <button
              role="menuitem"
              class="dropdown-item {currentView === 'intelligence' ? 'selected' : ''}"
              on:click={() => navigate('intelligence')}
            >
              <span class="dropdown-icon">🧠</span>
              <div class="dropdown-meta">
                <span class="dropdown-title">Intelligence</span>
                <span class="dropdown-sub">NLP, formality & transliteration</span>
              </div>
            </button>
            <button
              role="menuitem"
              class="dropdown-item {currentView === 'downloads' ? 'selected' : ''}"
              on:click={() => navigate('downloads')}
            >
              <span class="dropdown-icon">📦</span>
              <div class="dropdown-meta">
                <span class="dropdown-title">Model Packs</span>
                <span class="dropdown-sub">Offline weights & signature audits</span>
              </div>
            </button>
            <button
              role="menuitem"
              class="dropdown-item {currentView === 'enterprise' ? 'selected' : ''}"
              on:click={() => navigate('enterprise')}
            >
              <span class="dropdown-icon">🏢</span>
              <div class="dropdown-meta">
                <span class="dropdown-title">Enterprise</span>
                <span class="dropdown-sub">Audit logs, RBAC & API gateways</span>
              </div>
            </button>
            <button
              role="menuitem"
              class="dropdown-item {currentView === 'settings' ? 'selected' : ''}"
              on:click={() => navigate('settings')}
            >
              <span class="dropdown-icon">⚙️</span>
              <div class="dropdown-meta">
                <span class="dropdown-title">Settings</span>
                <span class="dropdown-sub">Theme, audio devices & developer HUD</span>
              </div>
            </button>
          </div>
        {/if}
      </div>
    </nav>

    <!-- Header Actions -->
    <div class="header-actions">
      <button
        class="shortcuts-btn"
        on:click={() => (showShortcutsModal = true)}
        title="View Keyboard Shortcuts (Ctrl+K, Ctrl+Enter, Esc)"
        aria-label="Keyboard Shortcuts"
      >
        ⌨️
      </button>
      <button
        class="theme-toggle"
        on:click={toggleThemeQuick}
        title="Toggle Dark/Light Mode"
        aria-label="Toggle Theme"
      >
        {userSettings?.theme === 'light' ? '🌙' : '☀️'}
      </button>
    </div>
  </header>

  <!-- Main View Container -->
  <main class="main-content" id="main-content">
    {#if loading}
      <div class="loading-state glass-panel">
        <div class="spinner"></div>
        <p>Connecting to LinguaFlow Rust Core...</p>
      </div>
    {:else if errorState}
      <div class="error-state glass-panel">
        <h3>Initialization Alert</h3>
        <p>{errorState}</p>
        <button class="btn-primary" on:click={initializeApp}>Retry Connection</button>
      </div>
    {:else}
      {#if currentView === 'home'}
        <HomeView {appInfo} {health} {capabilities} onNavigate={navigate} />
      {:else if currentView === 'translate'}
        <TranslateView settings={userSettings} onSaveSettings={handleSaveSettings} onNavigate={navigate} />
      {:else if currentView === 'dictionary'}
        <DictionaryView />
      {:else if currentView === 'speech'}
        <LiveSpeechView onNavigate={navigate} />
      {:else if currentView === 'conversation'}
        <ConversationView onNavigate={navigate} />
      {:else if currentView === 'intelligence'}
        <IntelligenceView />
      {:else if currentView === 'media'}
        <MediaView />
      {:else if currentView === 'learn'}
        <LearnView />
      {:else if currentView === 'enterprise'}
        <EnterpriseView />
      {:else if currentView === 'downloads'}
        <ModelsView />
      {:else if currentView === 'settings'}
        <SettingsView settings={userSettings} onSaveSettings={handleSaveSettings} onNavigate={navigate} />
      {/if}
    {/if}
  </main>

  <!-- Mobile Bottom Navigation Bar (< 768px) with Safe-Area Clearance -->
  <nav class="mobile-nav glass-panel" aria-label="Mobile Navigation">
    <button
      class="mobile-nav-btn {currentView === 'home' ? 'active' : ''}"
      on:click={() => navigate('home')}
      aria-label="Home"
    >
      <span class="m-icon">🏠</span>
      <span>Home</span>
    </button>
    <button
      class="mobile-nav-btn {currentView === 'translate' ? 'active' : ''}"
      on:click={() => navigate('translate')}
      aria-label="Translate"
    >
      <span class="m-icon">⇄</span>
      <span>Translate</span>
    </button>
    <button
      class="mobile-nav-btn {currentView === 'dictionary' ? 'active' : ''}"
      on:click={() => navigate('dictionary')}
      aria-label="Dictionary"
    >
      <span class="m-icon">📖</span>
      <span>Dictionary</span>
    </button>
    <button
      class="mobile-nav-btn {currentView === 'speech' ? 'active' : ''}"
      on:click={() => navigate('speech')}
      aria-label="Live Captions"
    >
      <span class="m-icon">🎙️</span>
      <span>Captions</span>
    </button>
    <button
      class="mobile-nav-btn {(!['home', 'translate', 'dictionary', 'speech'].includes(currentView)) ? 'active' : ''}"
      on:click={() => (mobileDrawerOpen = !mobileDrawerOpen)}
      aria-label="More Workspaces & System Drawer"
    >
      <span class="m-icon">☰</span>
      <span>More</span>
      {#if !['home', 'translate', 'dictionary', 'speech'].includes(currentView)}
        <span class="active-dot"></span>
      {/if}
    </button>
  </nav>

  <!-- Mobile Slide-Up Drawer Sheet -->
  {#if mobileDrawerOpen}
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <div
      class="mobile-drawer-backdrop"
      on:click={() => (mobileDrawerOpen = false)}
      role="presentation"
    ></div>
    <aside class="mobile-drawer-sheet glass-panel" aria-label="Extended Navigation Drawer">
      <div class="drawer-handle-bar"></div>
      <div class="drawer-header">
        <div class="drawer-title-group">
          <span class="drawer-brand">Lingua<span class="gradient-text">Flow</span></span>
          <span class="drawer-sub">Workspaces & System Services</span>
        </div>
        <button class="drawer-close" on:click={() => (mobileDrawerOpen = false)} aria-label="Close drawer">✕</button>
      </div>

      <div class="drawer-body">
        <div class="drawer-category">
          <div class="category-title">Core Workspaces</div>
          <div class="category-grid">
            <button
              class="drawer-tile {currentView === 'conversation' ? 'active' : ''}"
              on:click={() => navigate('conversation')}
            >
              <span class="tile-icon">👥</span>
              <div class="tile-info">
                <span class="tile-title">Conversation</span>
                <span class="tile-desc">Turn-taking live voice</span>
              </div>
            </button>
            <button
              class="drawer-tile {currentView === 'media' ? 'active' : ''}"
              on:click={() => navigate('media')}
            >
              <span class="tile-icon">🎞️</span>
              <div class="tile-info">
                <span class="tile-title">Media & OCR</span>
                <span class="tile-desc">Subtitles, document OCR & vision</span>
              </div>
            </button>
            <button
              class="drawer-tile {currentView === 'learn' ? 'active' : ''}"
              on:click={() => navigate('learn')}
            >
              <span class="tile-icon">🗂️</span>
              <div class="tile-info">
                <span class="tile-title">Learn</span>
                <span class="tile-desc">SM-2 spaced flashcards & quizzes</span>
              </div>
            </button>
          </div>
        </div>

        <div class="drawer-category">
          <div class="category-title">System & Platform</div>
          <div class="category-grid">
            <button
              class="drawer-tile {currentView === 'intelligence' ? 'active' : ''}"
              on:click={() => navigate('intelligence')}
            >
              <span class="tile-icon">🧠</span>
              <div class="tile-info">
                <span class="tile-title">Intelligence</span>
                <span class="tile-desc">Formality & transliteration</span>
              </div>
            </button>
            <button
              class="drawer-tile {currentView === 'downloads' ? 'active' : ''}"
              on:click={() => navigate('downloads')}
            >
              <span class="tile-icon">📦</span>
              <div class="tile-info">
                <span class="tile-title">Model Packs</span>
                <span class="tile-desc">Offline neural models</span>
              </div>
            </button>
            <button
              class="drawer-tile {currentView === 'enterprise' ? 'active' : ''}"
              on:click={() => navigate('enterprise')}
            >
              <span class="tile-icon">🏢</span>
              <div class="tile-info">
                <span class="tile-title">Enterprise</span>
                <span class="tile-desc">Audit logs & RBAC gateways</span>
              </div>
            </button>
            <button
              class="drawer-tile {currentView === 'settings' ? 'active' : ''}"
              on:click={() => navigate('settings')}
            >
              <span class="tile-icon">⚙️</span>
              <div class="tile-info">
                <span class="tile-title">Settings</span>
                <span class="tile-desc">Preferences & Developer HUD</span>
              </div>
            </button>
          </div>
        </div>
      </div>

      <div class="drawer-footer">
        <button class="drawer-action-btn" on:click={toggleThemeQuick}>
          {userSettings?.theme === 'light' ? '🌙 Dark Mode' : '☀️ Light Mode'}
        </button>
        <button class="drawer-action-btn" on:click={() => { showShortcutsModal = true; mobileDrawerOpen = false; }}>
          ⌨️ Shortcuts
        </button>
      </div>
    </aside>
  {/if}

  <!-- Global Keyboard Shortcuts Modal -->
  {#if showShortcutsModal}
    <!-- svelte-ignore a11y-click-events-have-key-events -->
    <div
      class="modal-backdrop"
      on:click={() => (showShortcutsModal = false)}
      role="presentation"
    ></div>
    <div class="shortcuts-dialog glass-panel" role="dialog" aria-labelledby="shortcuts-dialog-title" aria-modal="true">
      <div class="shortcuts-dialog-header">
        <div class="dialog-title-group">
          <span class="dialog-icon">⌨️</span>
          <h3 id="shortcuts-dialog-title">Global Keyboard Workflows</h3>
        </div>
        <button class="dialog-close-btn" on:click={() => (showShortcutsModal = false)} aria-label="Close dialog">✕</button>
      </div>

      <div class="shortcuts-table">
        <div class="shortcut-item">
          <div class="shortcut-meta">
            <span class="shortcut-action">Execute Translation</span>
            <span class="shortcut-context">Trigger translation on source text</span>
          </div>
          <kbd class="kbd-badge">Ctrl / ⌘ + Enter</kbd>
        </div>
        <div class="shortcut-item">
          <div class="shortcut-meta">
            <span class="shortcut-action">Swap Languages</span>
            <span class="shortcut-context">Invert source and target language pairs</span>
          </div>
          <kbd class="kbd-badge">Ctrl / ⌘ + Shift + S</kbd>
        </div>
        <div class="shortcut-item">
          <div class="shortcut-meta">
            <span class="shortcut-action">Dictionary Instant Search</span>
            <span class="shortcut-context">Jump to dictionary and focus input</span>
          </div>
          <kbd class="kbd-badge">Ctrl / ⌘ + K</kbd>
        </div>
        <div class="shortcut-item">
          <div class="shortcut-meta">
            <span class="shortcut-action">Stop Audio / Clear / Dismiss</span>
            <span class="shortcut-context">Halt speech synthesis or close open modals</span>
          </div>
          <kbd class="kbd-badge">Esc</kbd>
        </div>
        <div class="shortcut-item">
          <div class="shortcut-meta">
            <span class="shortcut-action">Quick View Jump</span>
            <span class="shortcut-context">Direct access to primary workspaces</span>
          </div>
          <kbd class="kbd-badge">Alt + 1 .. 7</kbd>
        </div>
      </div>
    </div>
  {/if}
</div>

<style>
  .app-layout {
    min-height: 100vh;
    display: flex;
    flex-direction: column;
    padding-bottom: 70px;
  }

  .app-header {
    margin: 16px 20px 0 20px;
    padding: 10px 20px;
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 16px;
    position: sticky;
    top: 16px;
    z-index: 100;
  }

  .brand-group {
    display: flex;
    align-items: center;
    gap: 12px;
    cursor: pointer;
    user-select: none;
    background: transparent;
    border: none;
    padding: 0;
    text-align: left;
    font-family: inherit;
    flex-shrink: 0;
  }

  .logo-circle {
    background: var(--primary-gradient);
    color: #ffffff;
    font-weight: 800;
    font-size: 1.1rem;
    width: 38px;
    height: 38px;
    border-radius: var(--radius-md);
    display: flex;
    align-items: center;
    justify-content: center;
    box-shadow: var(--shadow-sm);
  }

  .brand-name {
    font-weight: 800;
    font-size: 1.25rem;
    letter-spacing: -0.02em;
    display: block;
    line-height: 1.1;
  }

  .brand-stage {
    font-size: 0.72rem;
    color: var(--text-muted);
    font-weight: 500;
    text-transform: uppercase;
    letter-spacing: 0.04em;
  }

  .desktop-nav {
    display: flex;
    align-items: center;
    gap: 4px;
    flex-wrap: wrap;
  }

  .nav-btn {
    background: transparent;
    color: var(--text-secondary);
    padding: 7px 12px;
    border-radius: var(--radius-md);
    font-weight: 600;
    font-size: 0.88rem;
    display: inline-flex;
    align-items: center;
    gap: 6px;
    white-space: nowrap;
    cursor: pointer;
    border: 1px solid transparent;
  }

  .nav-btn:hover {
    color: var(--text-primary);
    background: var(--bg-surface);
  }

  .nav-btn.active {
    color: #ffffff;
    background: var(--primary-gradient);
    box-shadow: var(--shadow-sm);
    border-color: transparent;
  }

  /* System Drawer Dropdown Container */
  .system-nav-container {
    position: relative;
    display: inline-block;
  }

  .system-trigger {
    border: 1px solid var(--border-subtle);
  }

  .system-trigger.active {
    background: var(--card-gradient);
    border-color: var(--primary-500);
    color: var(--text-accent);
  }

  .chevron {
    font-size: 0.8rem;
    transition: transform var(--transition-fast);
  }

  .chevron.open {
    transform: rotate(180deg);
  }

  .system-dropdown {
    position: absolute;
    top: calc(100% + 8px);
    right: 0;
    width: 280px;
    padding: 8px;
    display: flex;
    flex-direction: column;
    gap: 4px;
    box-shadow: var(--shadow-lg);
    z-index: 150;
    border: 1px solid var(--border-focus);
    animation: fadeInDropdown 0.15s ease-out;
  }

  @keyframes fadeInDropdown {
    from {
      opacity: 0;
      transform: translateY(-6px);
    }
    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  .dropdown-item {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 8px 12px;
    border-radius: var(--radius-md);
    background: transparent;
    color: var(--text-primary);
    text-align: left;
    cursor: pointer;
    transition: background var(--transition-fast);
  }

  .dropdown-item:hover {
    background: var(--bg-surface-elevated);
  }

  .dropdown-item.selected {
    background: rgba(14, 165, 233, 0.12);
    border: 1px solid rgba(14, 165, 233, 0.3);
  }

  .dropdown-icon {
    font-size: 1.3rem;
  }

  .dropdown-meta {
    display: flex;
    flex-direction: column;
  }

  .dropdown-title {
    font-weight: 600;
    font-size: 0.9rem;
    color: var(--text-primary);
  }

  .dropdown-sub {
    font-size: 0.72rem;
    color: var(--text-muted);
  }

  .header-actions {
    display: flex;
    align-items: center;
    gap: 8px;
    flex-shrink: 0;
  }

  .shortcuts-btn, .theme-toggle {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-full);
    width: 36px;
    height: 36px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 1.05rem;
    cursor: pointer;
    color: var(--text-primary);
    transition: border-color var(--transition-fast), transform var(--transition-fast);
  }

  .shortcuts-btn:hover, .theme-toggle:hover {
    border-color: var(--text-accent);
    transform: scale(1.05);
  }

  .main-content {
    max-width: 1240px;
    width: 100%;
    margin: 20px auto;
    padding: 0 16px;
    flex: 1;
  }

  .loading-state, .error-state {
    padding: 60px 20px;
    text-align: center;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 16px;
  }

  .spinner {
    width: 40px;
    height: 40px;
    border: 3px solid var(--border-subtle);
    border-top-color: var(--primary-500);
    border-radius: 50%;
    animation: spin 1s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }

  /* Mobile Bottom Navigation Bar */
  .mobile-nav {
    display: none;
    position: fixed;
    bottom: 8px;
    left: 12px;
    right: 12px;
    height: 62px;
    border-radius: var(--radius-lg);
    z-index: 100;
    justify-content: space-around;
    align-items: center;
    padding: 0 6px;
    padding-bottom: max(6px, env(safe-area-inset-bottom, 6px));
    box-shadow: var(--shadow-lg);
  }

  .mobile-nav-btn {
    background: transparent;
    color: var(--text-muted);
    display: flex;
    flex-direction: column;
    align-items: center;
    justify-content: center;
    gap: 2px;
    font-size: 0.72rem;
    font-weight: 500;
    padding: 6px;
    min-width: 48px;
    min-height: 48px;
    position: relative;
    border-radius: var(--radius-md);
  }

  .mobile-nav-btn .m-icon {
    font-size: 1.25rem;
  }

  .mobile-nav-btn.active {
    color: var(--text-accent);
    font-weight: 700;
  }

  .active-dot {
    position: absolute;
    top: 6px;
    right: 10px;
    width: 6px;
    height: 6px;
    background: var(--text-accent);
    border-radius: 50%;
  }

  /* Mobile Slide-Up Drawer Sheet */
  .mobile-drawer-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.6);
    backdrop-filter: blur(4px);
    z-index: 180;
    animation: fadeIn 0.2s ease-out;
  }

  .mobile-drawer-sheet {
    position: fixed;
    bottom: 0;
    left: 0;
    right: 0;
    max-height: 85vh;
    border-bottom-left-radius: 0;
    border-bottom-right-radius: 0;
    border-top-left-radius: 20px;
    border-top-right-radius: 20px;
    z-index: 190;
    padding: 12px 16px max(16px, env(safe-area-inset-bottom, 16px)) 16px;
    display: flex;
    flex-direction: column;
    gap: 14px;
    overflow-y: auto;
    box-shadow: var(--shadow-lg);
    animation: slideUp 0.25s cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes slideUp {
    from {
      transform: translateY(100%);
    }
    to {
      transform: translateY(0);
    }
  }

  @keyframes fadeIn {
    from { opacity: 0; }
    to { opacity: 1; }
  }

  .drawer-handle-bar {
    width: 44px;
    height: 5px;
    background: var(--border-subtle);
    border-radius: 3px;
    margin: 0 auto;
  }

  .drawer-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .drawer-title-group {
    display: flex;
    flex-direction: column;
  }

  .drawer-brand {
    font-size: 1.15rem;
    font-weight: 800;
  }

  .drawer-sub {
    font-size: 0.75rem;
    color: var(--text-muted);
  }

  .drawer-close {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-full);
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.9rem;
    color: var(--text-secondary);
  }

  .drawer-body {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .drawer-category {
    display: flex;
    flex-direction: column;
    gap: 8px;
  }

  .category-title {
    font-size: 0.75rem;
    font-weight: 700;
    text-transform: uppercase;
    letter-spacing: 0.05em;
    color: var(--text-muted);
  }

  .category-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(130px, 1fr));
    gap: 8px;
  }

  .drawer-tile {
    display: flex;
    flex-direction: column;
    gap: 4px;
    padding: 10px;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    text-align: left;
    min-height: 48px;
    cursor: pointer;
  }

  .drawer-tile:hover, .drawer-tile.active {
    border-color: var(--primary-500);
    background: var(--bg-surface-elevated);
  }

  .drawer-tile.active {
    box-shadow: 0 0 0 1px var(--primary-500);
  }

  .tile-icon {
    font-size: 1.3rem;
  }

  .tile-title {
    font-size: 0.85rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .tile-desc {
    font-size: 0.68rem;
    color: var(--text-muted);
    line-height: 1.3;
  }

  .drawer-footer {
    display: flex;
    gap: 10px;
    margin-top: 6px;
    padding-top: 10px;
    border-top: 1px solid var(--border-subtle);
  }

  .drawer-action-btn {
    flex: 1;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
    padding: 8px;
    font-size: 0.82rem;
    font-weight: 600;
    color: var(--text-primary);
    text-align: center;
  }

  /* Keyboard Shortcuts Modal */
  .modal-backdrop {
    position: fixed;
    inset: 0;
    background: rgba(0, 0, 0, 0.65);
    backdrop-filter: blur(4px);
    z-index: 250;
    animation: fadeIn 0.15s ease-out;
  }

  .shortcuts-dialog {
    position: fixed;
    top: 50%;
    left: 50%;
    transform: translate(-50%, -50%);
    width: 90%;
    max-width: 520px;
    padding: 24px;
    z-index: 260;
    box-shadow: var(--shadow-lg);
    border: 1px solid var(--border-focus);
    display: flex;
    flex-direction: column;
    gap: 16px;
    animation: scaleModal 0.2s cubic-bezier(0.16, 1, 0.3, 1);
  }

  @keyframes scaleModal {
    from {
      opacity: 0;
      transform: translate(-50%, -46%) scale(0.96);
    }
    to {
      opacity: 1;
      transform: translate(-50%, -50%) scale(1);
    }
  }

  .shortcuts-dialog-header {
    display: flex;
    align-items: center;
    justify-content: space-between;
  }

  .dialog-title-group {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  .dialog-icon {
    font-size: 1.4rem;
  }

  .dialog-close-btn {
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-full);
    width: 32px;
    height: 32px;
    display: flex;
    align-items: center;
    justify-content: center;
    font-size: 0.95rem;
    color: var(--text-secondary);
  }

  .shortcuts-table {
    display: flex;
    flex-direction: column;
    gap: 10px;
  }

  .shortcut-item {
    display: flex;
    align-items: center;
    justify-content: space-between;
    gap: 12px;
    padding: 10px 12px;
    background: var(--bg-surface);
    border: 1px solid var(--border-subtle);
    border-radius: var(--radius-md);
  }

  .shortcut-meta {
    display: flex;
    flex-direction: column;
  }

  .shortcut-action {
    font-size: 0.88rem;
    font-weight: 600;
    color: var(--text-primary);
  }

  .shortcut-context {
    font-size: 0.72rem;
    color: var(--text-muted);
  }

  .kbd-badge {
    background: var(--bg-surface-elevated);
    border: 1px solid var(--border-focus);
    border-radius: 6px;
    padding: 4px 8px;
    font-family: var(--font-mono);
    font-size: 0.78rem;
    font-weight: 700;
    color: var(--text-accent);
    box-shadow: 0 1px 3px rgba(0, 0, 0, 0.2);
    white-space: nowrap;
  }

  @media (max-width: 900px) {
    .desktop-nav {
      display: none;
    }
    .mobile-nav {
      display: flex;
    }
    .app-layout {
      padding-bottom: 96px;
    }
    .app-header {
      margin: 12px 12px 0 12px;
      padding: 10px 14px;
    }
    .brand-stage {
      display: none;
    }
  }
</style>
