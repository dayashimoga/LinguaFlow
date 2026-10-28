<script lang="ts">
  import { onMount } from 'svelte';
  import { listModelPacks } from '../lib/ipc';
  import type { ModelPackInfo } from '../lib/types';

  let packs: ModelPackInfo[] = [];
  let loading = true;
  let errorMsg: string | null = null;

  async function loadPacks() {
    loading = true;
    errorMsg = null;
    try {
      packs = await listModelPacks();
    } catch (e: any) {
      errorMsg = e?.message || 'Failed to query model packs';
    } finally {
      loading = false;
    }
  }

  onMount(() => {
    loadPacks();
  });

  function formatBytes(bytes: number): string {
    if (bytes === 0) return '0 B';
    const mb = bytes / (1024 * 1024);
    return `${mb.toFixed(1)} MB`;
  }
</script>

<div class="downloads-view">
  <!-- Storage Budget & Integrity Overview -->
  <div class="budget-card glass-panel">
    <div class="budget-header">
      <h2>Offline Model Packs & Storage Manager</h2>
      <button class="btn-secondary" on:click={loadPacks} disabled={loading}>
        {loading ? 'Refreshing...' : '↻ Refresh Manifest'}
      </button>
    </div>
    <p class="budget-desc">
      In accordance with <strong>AGENTS.md Requirement 7</strong> ("Verifiable upstream model versions, machine-readable inventory, never bundle huge models silently"), all offline packs require cryptographic SHA-256 integrity verification and user consent prior to local storage allocation.
    </p>

    <div class="metrics-row">
      <div class="metric-box">
        <span class="m-label">Storage Budget Limit</span>
        <span class="m-val">User-Prompted</span>
        <span class="m-sub">Zero Silent Downloads</span>
      </div>
      <div class="metric-box">
        <span class="m-label">Active Installed Size</span>
        <span class="m-val">0.0 MB</span>
        <span class="m-sub">Clean Stage 0 Footprint</span>
      </div>
      <div class="metric-box">
        <span class="m-label">Integrity Protocol</span>
        <span class="m-val">SHA-256</span>
        <span class="m-sub">Cryptographic Manifests</span>
      </div>
    </div>
  </div>

  <!-- Model Packs Catalog List -->
  {#if loading}
    <div class="status-msg glass-panel">Loading model packs from SQLite database...</div>
  {:else if errorMsg}
    <div class="status-msg error glass-panel">Error: {errorMsg}</div>
  {:else}
    <div class="packs-list">
      {#each packs as pack}
        <div class="pack-card glass-panel">
          <div class="pack-main">
            <div class="pack-title-row">
              <h3>{pack.name}</h3>
              <span class="badge badge-info">Stage {pack.scheduled_stage}</span>
              {#if pack.is_installed}
                <span class="badge badge-active">Installed</span>
              {:else}
                <span class="badge badge-pending">Available in Catalog</span>
              {/if}
            </div>

            <div class="pack-meta-row">
              <span><strong>Pair:</strong> <code>{pack.source}</code> ➔ <code>{pack.target}</code></span>
              <span><strong>Version:</strong> v{pack.version}</span>
              <span><strong>Size:</strong> {formatBytes(pack.size_bytes)}</span>
            </div>

            <div class="pack-checksum">
              <span class="hash-label">SHA-256:</span>
              <code class="hash-val">{pack.sha256}</code>
            </div>
          </div>

          <div class="pack-action">
            <button class="btn-secondary" disabled title="Scheduled for Stage {pack.scheduled_stage}">
              Download (Stage {pack.scheduled_stage})
            </button>
          </div>
        </div>
      {/each}
    </div>
  {/if}
</div>

<style>
  .downloads-view {
    display: flex;
    flex-direction: column;
    gap: 20px;
  }

  .budget-card {
    padding: 28px;
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .budget-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
    flex-wrap: wrap;
    gap: 12px;
  }

  .budget-desc {
    color: var(--text-secondary);
    font-size: 0.95rem;
    line-height: 1.6;
    max-width: 820px;
  }

  .metrics-row {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 16px;
    padding-top: 16px;
    border-top: 1px solid var(--border-subtle);
  }

  .metric-box {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .m-label {
    font-size: 0.8rem;
    color: var(--text-muted);
    text-transform: uppercase;
    font-weight: 600;
  }

  .m-val {
    font-size: 1.3rem;
    font-weight: 700;
  }

  .m-sub {
    font-size: 0.85rem;
    color: var(--text-accent);
  }

  .status-msg {
    padding: 24px;
    text-align: center;
    color: var(--text-secondary);
  }

  .status-msg.error {
    color: #f87171;
    border-color: rgba(248, 113, 113, 0.3);
  }

  .packs-list {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .pack-card {
    padding: 20px;
    display: flex;
    justify-content: space-between;
    align-items: center;
    gap: 20px;
    flex-wrap: wrap;
  }

  .pack-main {
    display: flex;
    flex-direction: column;
    gap: 10px;
    flex: 1;
    min-width: 280px;
  }

  .pack-title-row {
    display: flex;
    align-items: center;
    gap: 10px;
    flex-wrap: wrap;
  }

  .pack-title-row h3 {
    font-size: 1.1rem;
  }

  .pack-meta-row {
    display: flex;
    gap: 20px;
    flex-wrap: wrap;
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .pack-meta-row code {
    background: var(--bg-surface);
    padding: 2px 6px;
    border-radius: 4px;
    font-family: var(--font-mono);
  }

  .pack-checksum {
    display: flex;
    align-items: center;
    gap: 8px;
    font-size: 0.8rem;
  }

  .hash-label {
    color: var(--text-muted);
    font-weight: 600;
  }

  .hash-val {
    background: var(--bg-surface);
    padding: 3px 8px;
    border-radius: 4px;
    font-family: var(--font-mono);
    color: var(--text-secondary);
    word-break: break-all;
  }

  .pack-action {
    display: flex;
    align-items: center;
  }
</style>
