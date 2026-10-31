<script lang="ts">
  import { onMount } from 'svelte';
  import {
    listDetailedModelPacks,
    downloadModelPack,
    verifyModelPack,
    deleteModelPack,
    getDiskSpace,
  } from '../lib/ipc';
  import type { ModelPackDetailedInfo, GetDiskSpaceCmdResponse } from '../lib/types';

  let models = $state<ModelPackDetailedInfo[]>([]);
  let diskSpace = $state<GetDiskSpaceCmdResponse | null>(null);
  let isLoading = $state(true);
  let activeActionId = $state<string | null>(null);
  let actionMessage = $state<{ type: 'success' | 'error' | 'info'; text: string } | null>(null);
  let filter = $state<'all' | 'installed' | 'available'>('all');

  const filteredModels = $derived(() => {
    if (filter === 'installed') {
      return models.filter((m) => m.state.status === 'Installed');
    }
    if (filter === 'available') {
      return models.filter((m) => m.state.status === 'NotInstalled');
    }
    return models;
  });

  const installedCount = $derived(() => {
    return models.filter((m) => m.state.status === 'Installed').length;
  });

  async function loadData() {
    isLoading = true;
    try {
      const [packsData, diskData] = await Promise.all([
        listDetailedModelPacks(),
        getDiskSpace().catch(() => null),
      ]);
      models = packsData;
      diskSpace = diskData;
    } catch (err: unknown) {
      actionMessage = {
        type: 'error',
        text: err instanceof Error ? err.message : 'Failed to load model manager data',
      };
    } finally {
      isLoading = false;
    }
  }

  async function handleDownload(id: string) {
    activeActionId = id;
    actionMessage = { type: 'info', text: `Downloading and verifying model pack '${id}'...` };
    try {
      await downloadModelPack(id);
      actionMessage = { type: 'success', text: `Model pack '${id}' downloaded, verified, and installed!` };
      await loadData();
    } catch (err: unknown) {
      actionMessage = {
        type: 'error',
        text: err instanceof Error ? err.message : 'Download failed',
      };
    } finally {
      activeActionId = null;
    }
  }

  async function handleVerify(id: string) {
    activeActionId = id;
    actionMessage = { type: 'info', text: `Computing SHA-256 checksum for '${id}'...` };
    try {
      const res = await verifyModelPack(id);
      if (res.is_valid) {
        actionMessage = { type: 'success', text: `Integrity check passed! ${res.details}` };
      } else {
        actionMessage = { type: 'error', text: `Integrity check FAILED: ${res.details}` };
      }
      await loadData();
    } catch (err: unknown) {
      actionMessage = {
        type: 'error',
        text: err instanceof Error ? err.message : 'Verification failed',
      };
    } finally {
      activeActionId = null;
    }
  }

  async function handleDelete(id: string) {
    if (!confirm(`Are you sure you want to delete and evict model pack '${id}' from local disk?`)) {
      return;
    }
    activeActionId = id;
    try {
      await deleteModelPack(id);
      actionMessage = { type: 'success', text: `Model pack '${id}' safely evicted from storage.` };
      await loadData();
    } catch (err: unknown) {
      actionMessage = {
        type: 'error',
        text: err instanceof Error ? err.message : 'Failed to delete model pack',
      };
    } finally {
      activeActionId = null;
    }
  }

  onMount(() => {
    loadData();
  });
</script>

<div class="models-container">
  <header class="models-header">
    <div class="title-area">
      <div class="badge-row">
        <span class="stage-badge">Local Inference Runtime</span>
        <span class="offline-badge">100% Offline On-Device Inference</span>
        <span class="crypto-badge">✓ Ed25519 Signed Manifests</span>
        <span class="ram-badge">RAM &lt; 380 MB (90s Idle Evictor)</span>
      </div>
      <h2>On-Device Translation Models</h2>
      <p class="subtitle">
        Manage local neural models for Kannada, Tamil, and English. Installed models execute
        completely offline on your CPU/NPU without sending text to cloud providers.
      </p>
    </div>

    {#if diskSpace}
      <div class="storage-card">
        <div class="storage-row">
          <span class="storage-label">Local Storage</span>
          <span class="storage-value">
            {(diskSpace.available_bytes / (1024 * 1024 * 1024)).toFixed(1)} GB free of {(diskSpace.total_bytes / (1024 * 1024 * 1024)).toFixed(1)} GB
          </span>
        </div>
        <div class="storage-bar">
          <div
            class="storage-fill"
            style="width: {Math.min(100, Math.max(5, ((diskSpace.total_bytes - diskSpace.available_bytes) / diskSpace.total_bytes) * 100))}%"
          ></div>
        </div>
        <div class="storage-path" title={diskSpace.models_path}>
          Path: <code>{diskSpace.models_path}</code>
        </div>
      </div>
    {/if}
  </header>

  {#if actionMessage}
    <div class="alert {actionMessage.type}">
      <span>{actionMessage.text}</span>
      <button type="button" class="close-alert" onclick={() => (actionMessage = null)}>×</button>
    </div>
  {/if}

  <div class="controls-bar">
    <div class="filter-chips">
      <button
        type="button"
        class="chip {filter === 'all' ? 'active' : ''}"
        onclick={() => (filter = 'all')}
      >
        All Models ({models.length})
      </button>
      <button
        type="button"
        class="chip {filter === 'installed' ? 'active' : ''}"
        onclick={() => (filter = 'installed')}
      >
        Installed ({installedCount()})
      </button>
      <button
        type="button"
        class="chip {filter === 'available' ? 'active' : ''}"
        onclick={() => (filter = 'available')}
      >
        Available ({models.length - installedCount()})
      </button>
    </div>

    <button type="button" class="refresh-btn" onclick={loadData} disabled={isLoading}>
      {#if isLoading}
        Refreshing...
      {:else}
        ↻ Refresh
      {/if}
    </button>
  </div>

  {#if isLoading && models.length === 0}
    <div class="loading-state">
      <div class="spinner"></div>
      <p>Loading model catalog and storage diagnostics...</p>
    </div>
  {:else if filteredModels().length === 0}
    <div class="empty-state">
      <p>No models match the selected filter.</p>
    </div>
  {:else}
    <div class="models-grid">
      {#each filteredModels() as pack (pack.id)}
        <div class="model-card {pack.state.status.toLowerCase()}">
          <div class="card-top">
            <div class="card-header-left">
              <span class="lang-tag">{pack.source} ↔ {pack.target}</span>
              <span class="arch-tag">{pack.architecture}</span>
            </div>
            <div class="status-indicator {pack.state.status.toLowerCase()}">
              {#if pack.state.status === 'Installed'}
                <span class="dot green"></span> Installed
              {:else if pack.state.status === 'Corrupted'}
                <span class="dot red"></span> Corrupted
              {:else}
                <span class="dot gray"></span> Not Installed
              {/if}
            </div>
          </div>

          <h3 class="model-name">{pack.name}</h3>

          <div class="model-meta">
            <div class="meta-item">
              <span class="meta-label">Model ID</span>
              <span class="meta-val"><code>{pack.id}</code></span>
            </div>
            <div class="meta-item">
              <span class="meta-label">Download Size</span>
              <span class="meta-val">{(pack.size_bytes / (1024 * 1024)).toFixed(0)} MB</span>
            </div>
            <div class="meta-item">
              <span class="meta-label">Version</span>
              <span class="meta-val">{pack.version}</span>
            </div>
            <div class="meta-item">
              <span class="meta-label">Target Stage</span>
              <span class="meta-val">Stage {pack.scheduled_stage}</span>
            </div>
          </div>

          {#if pack.state.status === 'Installed'}
            <div class="installed-info">
              <span class="crypto-valid-pill">✓ SHA-256 + Ed25519 Verified</span>
              <code class="sha-val">{pack.sha256.substring(0, 16)}...</code>
            </div>
          {:else if pack.state.status === 'Paused'}
            <div class="installed-info paused-info">
              <span class="paused-pill">⏸ Download Paused ({pack.state.detail.progress_pct.toFixed(0)}%)</span>
              <span class="paused-bytes">{(pack.state.detail.downloaded_bytes / (1024 * 1024)).toFixed(1)} / {(pack.state.detail.total_bytes / (1024 * 1024)).toFixed(1)} MB</span>
            </div>
          {/if}

          <div class="card-actions">
            {#if pack.state.status === 'NotInstalled'}
              <button
                type="button"
                class="btn-primary"
                disabled={activeActionId === pack.id}
                onclick={() => handleDownload(pack.id)}
              >
                {#if activeActionId === pack.id}
                  <span class="mini-spinner"></span> Installing...
                {:else}
                  ↓ Download & Install ({(pack.size_bytes / (1024 * 1024)).toFixed(0)} MB)
                {/if}
              </button>
            {:else if pack.state.status === 'Paused'}
              <button
                type="button"
                class="btn-primary"
                disabled={activeActionId === pack.id}
                onclick={() => handleDownload(pack.id)}
              >
                {#if activeActionId === pack.id}
                  <span class="mini-spinner"></span> Resuming...
                {:else}
                  ▶ Resume Download ({pack.state.detail.progress_pct.toFixed(0)}%)
                {/if}
              </button>
              <button
                type="button"
                class="btn-danger"
                disabled={activeActionId === pack.id}
                onclick={() => handleDelete(pack.id)}
              >
                Discard Partial
              </button>
            {:else if pack.state.status === 'Installed'}
              <button
                type="button"
                class="btn-secondary"
                disabled={activeActionId === pack.id}
                onclick={() => handleVerify(pack.id)}
              >
                {#if activeActionId === pack.id}
                  Verifying...
                {:else}
                  ✓ Verify SHA-256
                {/if}
              </button>
              <button
                type="button"
                class="btn-danger"
                disabled={activeActionId === pack.id}
                onclick={() => handleDelete(pack.id)}
              >
                Delete / Evict
              </button>
            {:else if pack.state.status === 'Corrupted'}
              <button
                type="button"
                class="btn-primary"
                disabled={activeActionId === pack.id}
                onclick={() => handleDownload(pack.id)}
              >
                Repair & Reinstall
              </button>
              <button
                type="button"
                class="btn-danger"
                disabled={activeActionId === pack.id}
                onclick={() => handleDelete(pack.id)}
              >
                Clear Corrupted
              </button>
            {/if}
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .models-container {
    padding: 1.5rem;
    max-width: 1200px;
    margin: 0 auto;
    font-family: inherit;
    color: var(--text-color, #f1f5f9);
  }

  .models-header {
    display: flex;
    justify-content: space-between;
    align-items: flex-start;
    gap: 1.5rem;
    margin-bottom: 1.5rem;
    flex-wrap: wrap;
  }

  .title-area {
    flex: 1;
    min-width: 300px;
  }

  .badge-row {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 0.5rem;
  }

  .stage-badge {
    background: #6366f1;
    color: white;
    font-size: 0.75rem;
    font-weight: 700;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    text-transform: uppercase;
  }

  .offline-badge {
    background: #10b981;
    color: white;
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
  }

  .crypto-badge {
    background: rgba(99, 102, 241, 0.15);
    color: #818cf8;
    border: 1px solid rgba(99, 102, 241, 0.3);
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
  }

  .ram-badge {
    background: rgba(16, 185, 129, 0.15);
    color: #34d399;
    border: 1px solid rgba(16, 185, 129, 0.3);
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
  }

  .crypto-valid-pill {
    background: rgba(16, 185, 129, 0.15);
    color: #10b981;
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0.15rem 0.4rem;
    border-radius: 4px;
    margin-right: 0.5rem;
  }

  .paused-pill {
    background: rgba(245, 158, 11, 0.15);
    color: #f59e0b;
    font-size: 0.75rem;
    font-weight: 600;
    padding: 0.15rem 0.4rem;
    border-radius: 4px;
  }

  .paused-bytes {
    font-size: 0.75rem;
    color: var(--text-muted, #94a3b8);
    margin-left: 0.5rem;
  }

  .title-area h2 {
    font-size: 1.8rem;
    margin: 0 0 0.5rem 0;
    font-weight: 700;
    letter-spacing: -0.02em;
  }

  .subtitle {
    font-size: 0.95rem;
    color: var(--text-muted, #94a3b8);
    margin: 0;
    line-height: 1.5;
  }

  .storage-card {
    background: var(--card-bg, #1e293b);
    border: 1px solid var(--border-color, #334155);
    border-radius: 8px;
    padding: 1rem 1.25rem;
    min-width: 280px;
  }

  .storage-row {
    display: flex;
    justify-content: space-between;
    font-size: 0.85rem;
    margin-bottom: 0.5rem;
  }

  .storage-label {
    font-weight: 600;
  }

  .storage-value {
    color: var(--text-muted, #94a3b8);
  }

  .storage-bar {
    height: 6px;
    background: #334155;
    border-radius: 3px;
    overflow: hidden;
    margin-bottom: 0.5rem;
  }

  .storage-fill {
    height: 100%;
    background: linear-gradient(90deg, #6366f1, #3b82f6);
    border-radius: 3px;
  }

  .storage-path {
    font-size: 0.75rem;
    color: var(--text-muted, #64748b);
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .alert {
    padding: 0.75rem 1rem;
    border-radius: 6px;
    margin-bottom: 1.25rem;
    display: flex;
    justify-content: space-between;
    align-items: center;
    font-size: 0.9rem;
  }

  .alert.success {
    background: rgba(16, 185, 129, 0.15);
    border: 1px solid #10b981;
    color: #34d399;
  }

  .alert.error {
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid #ef4444;
    color: #f87171;
  }

  .alert.info {
    background: rgba(59, 130, 246, 0.15);
    border: 1px solid #3b82f6;
    color: #60a5fa;
  }

  .close-alert {
    background: none;
    border: none;
    color: inherit;
    font-size: 1.2rem;
    cursor: pointer;
  }

  .controls-bar {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 1.5rem;
    gap: 1rem;
    flex-wrap: wrap;
  }

  .filter-chips {
    display: flex;
    gap: 0.5rem;
  }

  .chip {
    background: var(--card-bg, #1e293b);
    border: 1px solid var(--border-color, #334155);
    color: var(--text-color, #f1f5f9);
    padding: 0.4rem 0.85rem;
    border-radius: 20px;
    font-size: 0.85rem;
    cursor: pointer;
    transition: all 0.15s ease;
  }

  .chip.active {
    background: #3b82f6;
    border-color: #3b82f6;
    color: white;
    font-weight: 600;
  }

  .refresh-btn {
    background: transparent;
    border: 1px solid var(--border-color, #334155);
    color: var(--text-color, #94a3b8);
    padding: 0.4rem 0.75rem;
    border-radius: 6px;
    font-size: 0.85rem;
    cursor: pointer;
  }

  .refresh-btn:hover {
    color: white;
    border-color: #64748b;
  }

  .models-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(340px, 1fr));
    gap: 1.25rem;
  }

  .model-card {
    background: var(--card-bg, #1e293b);
    border: 1px solid var(--border-color, #334155);
    border-radius: 10px;
    padding: 1.25rem;
    display: flex;
    flex-direction: column;
    justify-content: space-between;
    box-shadow: 0 4px 6px -1px rgba(0, 0, 0, 0.1);
    transition: border-color 0.2s ease;
  }

  .model-card:hover {
    border-color: #64748b;
  }

  .model-card.installed {
    border-left: 4px solid #10b981;
  }

  .model-card.corrupted {
    border-left: 4px solid #ef4444;
  }

  .card-top {
    display: flex;
    justify-content: space-between;
    align-items: center;
    margin-bottom: 0.75rem;
  }

  .card-header-left {
    display: flex;
    gap: 0.4rem;
    align-items: center;
  }

  .lang-tag {
    background: #334155;
    color: #e2e8f0;
    font-size: 0.75rem;
    padding: 0.2rem 0.45rem;
    border-radius: 4px;
    font-weight: 600;
  }

  .arch-tag {
    background: rgba(99, 102, 241, 0.15);
    color: #818cf8;
    font-size: 0.7rem;
    padding: 0.2rem 0.4rem;
    border-radius: 4px;
  }

  .status-indicator {
    display: flex;
    align-items: center;
    gap: 0.35rem;
    font-size: 0.8rem;
    font-weight: 600;
  }

  .status-indicator.installed {
    color: #34d399;
  }

  .status-indicator.corrupted {
    color: #f87171;
  }

  .status-indicator.notinstalled {
    color: #94a3b8;
  }

  .dot {
    width: 8px;
    height: 8px;
    border-radius: 50%;
  }

  .dot.green {
    background: #10b981;
  }

  .dot.red {
    background: #ef4444;
  }

  .dot.gray {
    background: #64748b;
  }

  .model-name {
    font-size: 1.1rem;
    margin: 0 0 1rem 0;
    font-weight: 600;
    line-height: 1.4;
  }

  .model-meta {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 0.6rem;
    margin-bottom: 1rem;
    background: rgba(15, 23, 42, 0.4);
    padding: 0.75rem;
    border-radius: 6px;
  }

  .meta-item {
    display: flex;
    flex-direction: column;
    gap: 0.15rem;
  }

  .meta-label {
    font-size: 0.7rem;
    color: var(--text-muted, #94a3b8);
    text-transform: uppercase;
  }

  .meta-val {
    font-size: 0.85rem;
    font-weight: 500;
  }

  .meta-val code {
    font-size: 0.75rem;
  }

  .installed-info {
    font-size: 0.75rem;
    color: #64748b;
    margin-bottom: 1rem;
    display: flex;
    align-items: center;
    gap: 0.4rem;
  }

  .sha-val {
    color: #94a3b8;
  }

  .card-actions {
    display: flex;
    gap: 0.5rem;
    margin-top: auto;
  }

  .btn-primary {
    flex: 1;
    background: #3b82f6;
    color: white;
    border: none;
    border-radius: 6px;
    padding: 0.6rem 0.75rem;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
    transition: background 0.15s ease;
    display: flex;
    justify-content: center;
    align-items: center;
    gap: 0.4rem;
  }

  .btn-primary:hover:not(:disabled) {
    background: #2563eb;
  }

  .btn-primary:disabled {
    opacity: 0.6;
    cursor: not-allowed;
  }

  .btn-secondary {
    flex: 1;
    background: #334155;
    color: #f1f5f9;
    border: none;
    border-radius: 6px;
    padding: 0.6rem 0.75rem;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-secondary:hover:not(:disabled) {
    background: #475569;
  }

  .btn-danger {
    background: rgba(239, 68, 68, 0.15);
    color: #f87171;
    border: 1px solid rgba(239, 68, 68, 0.3);
    border-radius: 6px;
    padding: 0.6rem 0.75rem;
    font-size: 0.85rem;
    font-weight: 600;
    cursor: pointer;
  }

  .btn-danger:hover:not(:disabled) {
    background: #ef4444;
    color: white;
  }

  .loading-state,
  .empty-state {
    text-align: center;
    padding: 3rem 1rem;
    color: var(--text-muted, #94a3b8);
  }

  .spinner {
    width: 32px;
    height: 32px;
    border: 3px solid rgba(99, 102, 241, 0.2);
    border-top-color: #6366f1;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    margin: 0 auto 1rem;
  }

  .mini-spinner {
    width: 14px;
    height: 14px;
    border: 2px solid rgba(255, 255, 255, 0.3);
    border-top-color: white;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
    display: inline-block;
  }

  @keyframes spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
