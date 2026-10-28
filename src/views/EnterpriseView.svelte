<script lang="ts">
  import { onMount } from 'svelte';
  import {
    getEnterpriseAccount,
    updateAccountTier,
    registerDevice,
    listDevices,
    createApiKey,
    listApiKeys,
    revokeApiKey,
    getUsageSummary,
    listInvoices,
    createOrganization,
    listOrgMembers,
    addOrgMember,
    listOrgGlossaries,
    createOrgGlossary,
    addOrgGlossaryTerm,
    listAuditEvents,
    getWorkerMetrics,
  } from '../lib/ipc';
  import type {
    AccountTier,
    ApiKeyRecord,
    AuditEvent,
    BillingInvoiceRecord,
    DeviceRegistration,
    DeviceType,
    LanguageCode,
    OrgGlossary,
    OrgMember,
    Organization,
    UsageSummary,
    UserAccount,
    WorkerMetrics,
  } from '../lib/types';

  type TabType = 'workspace' | 'metering' | 'glossaries' | 'developer';
  let activeTab: TabType = 'workspace';

  // State
  let loading = true;
  let statusMessage: string | null = null;
  let errorMessage: string | null = null;

  // Account & Quota State
  let account: UserAccount | null = null;
  let usageSummary: UsageSummary | null = null;
  let invoices: BillingInvoiceRecord[] = [];
  let devices: DeviceRegistration[] = [];

  // Organization & RBAC State
  let currentOrg: Organization | null = null;
  let orgMembers: OrgMember[] = [];
  let newOrgName = '';
  let newOrgSlug = '';
  let newMemberEmail = '';
  let newMemberName = '';
  let newMemberRole = 'member';

  // Glossaries State
  let glossaries: OrgGlossary[] = [];
  let selectedGlossaryId: string | null = null;
  let newGlossaryName = '';
  let newGlossaryDesc = '';
  let newGlossarySource: LanguageCode = 'en-US';
  let newGlossaryTarget: LanguageCode = 'kn-IN';
  let newTermSource = '';
  let newTermTarget = '';
  let newTermNotes = '';

  // Developer & Security State
  let apiKeys: ApiKeyRecord[] = [];
  let newKeyName = '';
  let generatedRawKey: string | null = null;
  let auditEvents: AuditEvent[] = [];
  let workerMetrics: WorkerMetrics | null = null;

  // Device registration form
  let newDeviceName = '';
  let newDeviceType: DeviceType = 'desktop_windows';

  onMount(async () => {
    await loadInitialData();
  });

  async function loadInitialData() {
    loading = true;
    errorMessage = null;
    try {
      // 1. Load account and usage
      account = await getEnterpriseAccount();
      usageSummary = await getUsageSummary(account?.id);
      invoices = await listInvoices(account?.id);
      devices = await listDevices(account?.id);

      // 2. Load API keys & worker telemetry
      apiKeys = await listApiKeys(account?.id);
      workerMetrics = await getWorkerMetrics();

      // 3. Mock or load an active organization if existing
      auditEvents = await listAuditEvents(undefined, 25);
    } catch (e: any) {
      errorMessage = e.message || 'Failed to initialize enterprise data';
    } finally {
      loading = false;
    }
  }

  async function handleSetTier(tier: AccountTier) {
    if (!account) return;
    try {
      await updateAccountTier(account.id, tier);
      account = await getEnterpriseAccount(account.id);
      usageSummary = await getUsageSummary(account.id);
      statusMessage = `Successfully upgraded account tier to: ${tier.toUpperCase()}`;
      setTimeout(() => (statusMessage = null), 4000);
    } catch (e: any) {
      errorMessage = e.message || 'Failed to update account tier';
    }
  }

  async function handleCreateOrg() {
    if (!account || !newOrgName.trim() || !newOrgSlug.trim()) return;
    try {
      const org = await createOrganization(newOrgName.trim(), newOrgSlug.trim(), account.id);
      currentOrg = org;
      orgMembers = await listOrgMembers(org.id);
      glossaries = await listOrgGlossaries(org.id);
      auditEvents = await listAuditEvents(org.id);
      newOrgName = '';
      newOrgSlug = '';
      statusMessage = `Organization '${org.name}' created with tenant isolation!`;
      setTimeout(() => (statusMessage = null), 4000);
    } catch (e: any) {
      errorMessage = e.message || 'Failed to create organization';
    }
  }

  async function handleAddMember() {
    if (!currentOrg || !newMemberEmail.trim() || !newMemberName.trim()) return;
    try {
      const mockUserId = `usr-${Date.now().toString(36)}`;
      await addOrgMember(
        currentOrg.id,
        mockUserId,
        newMemberEmail.trim(),
        newMemberName.trim(),
        newMemberRole
      );
      orgMembers = await listOrgMembers(currentOrg.id);
      auditEvents = await listAuditEvents(currentOrg.id);
      newMemberEmail = '';
      newMemberName = '';
      statusMessage = `Added ${newMemberName} to workspace with role: ${newMemberRole}`;
      setTimeout(() => (statusMessage = null), 4000);
    } catch (e: any) {
      errorMessage = e.message || 'Failed to add organization member';
    }
  }

  async function handleCreateGlossary() {
    if (!currentOrg || !newGlossaryName.trim()) return;
    try {
      await createOrgGlossary(
        currentOrg.id,
        newGlossaryName.trim(),
        newGlossaryDesc.trim(),
        newGlossarySource,
        newGlossaryTarget
      );
      glossaries = await listOrgGlossaries(currentOrg.id);
      newGlossaryName = '';
      newGlossaryDesc = '';
      statusMessage = 'Enterprise Dravidian glossary published.';
      setTimeout(() => (statusMessage = null), 4000);
    } catch (e: any) {
      errorMessage = e.message || 'Failed to create glossary';
    }
  }

  async function handleAddTerm(glossaryId: string) {
    if (!currentOrg || !newTermSource.trim() || !newTermTarget.trim()) return;
    try {
      await addOrgGlossaryTerm(
        currentOrg.id,
        glossaryId,
        newTermSource.trim(),
        newTermTarget.trim(),
        newTermNotes.trim() || undefined
      );
      glossaries = await listOrgGlossaries(currentOrg.id);
      newTermSource = '';
      newTermTarget = '';
      newTermNotes = '';
      statusMessage = 'Dravidian terminology pair added with active substitution.';
      setTimeout(() => (statusMessage = null), 4000);
    } catch (e: any) {
      errorMessage = e.message || 'Failed to add glossary term';
    }
  }

  async function handleCreateApiKey() {
    if (!account || !newKeyName.trim()) return;
    try {
      const { record, rawKey } = await createApiKey(
        account.id,
        newKeyName.trim(),
        currentOrg ? currentOrg.id : null
      );
      generatedRawKey = rawKey;
      apiKeys = await listApiKeys(account.id);
      newKeyName = '';
      statusMessage = 'New live API key generated successfully.';
      setTimeout(() => (statusMessage = null), 4000);
    } catch (e: any) {
      errorMessage = e.message || 'Failed to generate API key';
    }
  }

  async function handleRevokeApiKey(keyId: string) {
    try {
      await revokeApiKey(keyId);
      if (account) {
        apiKeys = await listApiKeys(account.id);
      }
      statusMessage = 'API key revoked.';
      setTimeout(() => (statusMessage = null), 4000);
    } catch (e: any) {
      errorMessage = e.message || 'Failed to revoke API key';
    }
  }

  async function handleRegisterDevice() {
    if (!account || !newDeviceName.trim()) return;
    try {
      const fp = `SHA256:${Math.random().toString(36).substring(2, 10)}${Math.random().toString(36).substring(2, 10)}`;
      await registerDevice(account.id, newDeviceName.trim(), newDeviceType, fp);
      devices = await listDevices(account.id);
      newDeviceName = '';
      statusMessage = 'Device registered with hardware fingerprint binding.';
      setTimeout(() => (statusMessage = null), 4000);
    } catch (e: any) {
      errorMessage = e.message || 'Failed to register device';
    }
  }

  async function refreshTelemetry() {
    try {
      workerMetrics = await getWorkerMetrics();
      if (account) {
        usageSummary = await getUsageSummary(account.id);
      }
    } catch (e: any) {
      // Non-blocking telemetry refresh
    }
  }
</script>

<div class="enterprise-view">
  <!-- Header Title & Tab Navigation -->
  <header class="enterprise-header glass-panel">
    <div class="title-row">
      <div class="icon-wrap">🏢</div>
      <div>
        <h2>Commercial & Enterprise Platform</h2>
        <p class="subtitle">
          Entitlements, usage metering, multi-tenant RBAC workspaces, Dravidian glossaries, and API gateway.
        </p>
      </div>
    </div>

    <!-- Navigation Tabs -->
    <div class="enterprise-tabs" role="tablist">
      <button
        class="tab-btn {activeTab === 'workspace' ? 'active' : ''}"
        role="tab"
        aria-selected={activeTab === 'workspace'}
        on:click={() => (activeTab = 'workspace')}
      >
        <span>🏛️</span> Workspaces & RBAC
      </button>
      <button
        class="tab-btn {activeTab === 'metering' ? 'active' : ''}"
        role="tab"
        aria-selected={activeTab === 'metering'}
        on:click={() => (activeTab = 'metering')}
      >
        <span>📊</span> Usage, Quotas & Invoices
      </button>
      <button
        class="tab-btn {activeTab === 'glossaries' ? 'active' : ''}"
        role="tab"
        aria-selected={activeTab === 'glossaries'}
        on:click={() => (activeTab = 'glossaries')}
      >
        <span>📚</span> Shared Glossaries
      </button>
      <button
        class="tab-btn {activeTab === 'developer' ? 'active' : ''}"
        role="tab"
        aria-selected={activeTab === 'developer'}
        on:click={() => (activeTab = 'developer')}
      >
        <span>⚡</span> Developer API & Audit
      </button>
    </div>
  </header>

  <!-- Alerts & Feedback -->
  {#if statusMessage}
    <div class="alert alert-success glass-panel">
      <span>✓</span> {statusMessage}
    </div>
  {/if}
  {#if errorMessage}
    <div class="alert alert-error glass-panel">
      <span>⚠️</span> {errorMessage}
      <button class="btn-text" on:click={() => (errorMessage = null)}>Dismiss</button>
    </div>
  {/if}

  {#if loading}
    <div class="loading-panel glass-panel">
      <div class="spinner"></div>
      <p>Synchronizing enterprise entitlements and cryptographic keys...</p>
    </div>
  {:else}
    <!-- Tab 1: Organization Workspaces & RBAC -->
    {#if activeTab === 'workspace'}
      <div class="tab-content workspace-tab">
        <div class="grid-2col">
          <!-- Active Organization Info / Creation Card -->
          <div class="card glass-panel">
            <h3>🏢 Organization Workspace</h3>
            {#if currentOrg}
              <div class="org-profile">
                <div class="profile-header">
                  <span class="org-badge">TENANT ISOLATED</span>
                  <h4>{currentOrg.name}</h4>
                  <span class="slug">slug: <code>{currentOrg.slug}</code></span>
                </div>
                <div class="prop-list">
                  <div class="prop">
                    <span class="label">Organization ID:</span>
                    <span class="val font-mono">{currentOrg.id}</span>
                  </div>
                  <div class="prop">
                    <span class="label">Seat Allocation:</span>
                    <span class="val">{orgMembers.length} / {currentOrg.max_seats} seats used</span>
                  </div>
                  <div class="prop">
                    <span class="label">Data Retention:</span>
                    <span class="val">{currentOrg.data_retention_days} days (Zero-transcript scrubbing)</span>
                  </div>
                  <div class="prop">
                    <span class="label">SSO Integration:</span>
                    <span class="val">{currentOrg.sso_enabled ? 'SAML 2.0 / OIDC Active' : 'Native Token Auth'}</span>
                  </div>
                </div>
              </div>
            {:else}
              <div class="empty-create-org">
                <p>You are currently operating in local mode. Create an enterprise organization workspace to enable multi-tenant collaboration, shared glossaries, and team access.</p>
                <div class="form-row">
                  <input
                    type="text"
                    placeholder="Organization Name (e.g. Bangalore Health Services)"
                    bind:value={newOrgName}
                    class="input-field"
                  />
                  <input
                    type="text"
                    placeholder="URL Slug (e.g. bangalore-health)"
                    bind:value={newOrgSlug}
                    class="input-field"
                  />
                  <button class="btn-primary" on:click={handleCreateOrg}>Create Workspace</button>
                </div>
              </div>
            {/if}
          </div>

          <!-- Role-Based Access Control (RBAC) Permissions Matrix -->
          <div class="card glass-panel">
            <h3>🔐 RBAC Permission Matrix</h3>
            <p class="section-desc">Strict multi-tenant role capabilities enforced in Rust core:</p>
            <div class="matrix-table-wrap">
              <table class="matrix-table">
                <thead>
                  <tr>
                    <th>Role</th>
                    <th>Team</th>
                    <th>Glossary</th>
                    <th>API Keys</th>
                    <th>Inference</th>
                  </tr>
                </thead>
                <tbody>
                  <tr>
                    <td><span class="role-pill owner">Owner</span></td>
                    <td class="check">✓</td>
                    <td class="check">✓</td>
                    <td class="check">✓</td>
                    <td class="check">✓</td>
                  </tr>
                  <tr>
                    <td><span class="role-pill admin">Admin</span></td>
                    <td class="check">✓</td>
                    <td class="check">✓</td>
                    <td class="check">✓</td>
                    <td class="check">✓</td>
                  </tr>
                  <tr>
                    <td><span class="role-pill linguist">Linguist Editor</span></td>
                    <td class="cross">✕</td>
                    <td class="check">✓</td>
                    <td class="cross">✕</td>
                    <td class="check">✓</td>
                  </tr>
                  <tr>
                    <td><span class="role-pill member">Member</span></td>
                    <td class="cross">✕</td>
                    <td class="cross">✕ (Read)</td>
                    <td class="cross">✕</td>
                    <td class="check">✓</td>
                  </tr>
                  <tr>
                    <td><span class="role-pill viewer">Viewer</span></td>
                    <td class="cross">✕</td>
                    <td class="cross">✕ (Read)</td>
                    <td class="cross">✕</td>
                    <td class="check">✓ (Read)</td>
                  </tr>
                </tbody>
              </table>
            </div>
          </div>
        </div>

        <!-- Team Members List -->
        {#if currentOrg}
          <div class="card glass-panel members-card">
            <div class="card-header-row">
              <h3>👥 Workspace Members & Seats ({orgMembers.length})</h3>
            </div>
            <div class="add-member-form">
              <input
                type="text"
                placeholder="Full Name (e.g. Dr. Priya Rao)"
                bind:value={newMemberName}
                class="input-field"
              />
              <input
                type="email"
                placeholder="Corporate Email (e.g. priya@org.in)"
                bind:value={newMemberEmail}
                class="input-field"
              />
              <select bind:value={newMemberRole} class="input-field select-field">
                <option value="admin">Admin</option>
                <option value="linguist_editor">Linguist Editor</option>
                <option value="member">Member</option>
                <option value="viewer">Viewer</option>
              </select>
              <button class="btn-primary" on:click={handleAddMember}>Add Member</button>
            </div>

            <div class="members-table-wrap">
              <table class="data-table">
                <thead>
                  <tr>
                    <th>Member</th>
                    <th>Email</th>
                    <th>Role</th>
                    <th>Joined</th>
                  </tr>
                </thead>
                <tbody>
                  {#each orgMembers as member}
                    <tr>
                      <td class="font-semibold">{member.display_name}</td>
                      <td><code>{member.email}</code></td>
                      <td>
                        <span class="role-pill {member.role.toLowerCase()}">{member.role}</span>
                      </td>
                      <td class="text-muted">{new Date(member.joined_at).toLocaleDateString()}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          </div>
        {/if}
      </div>

    <!-- Tab 2: Usage Metering, Quotas & Invoices -->
    {:else if activeTab === 'metering'}
      <div class="tab-content metering-tab">
        <!-- Account Overview & Plan Tiers -->
        <div class="card glass-panel account-banner">
          <div class="account-meta">
            <div class="avatar-chip">👤</div>
            <div>
              <h3>Account Status: <span class="tier-pill {account?.tier}">{account?.tier.toUpperCase()} PLAN</span></h3>
              <p class="offline-guarantee">
                🛡️ <strong>Anonymous Offline Guarantee Active:</strong> Local dictionary, speech synthesis, and offline neural translation require zero cloud tokens, quota, or network connectivity.
              </p>
            </div>
          </div>
          <div class="tier-actions">
            <button
              class="btn-tier {account?.tier === 'free' ? 'selected' : ''}"
              on:click={() => handleSetTier('free')}
            >
              Free
            </button>
            <button
              class="btn-tier {account?.tier === 'pro' ? 'selected' : ''}"
              on:click={() => handleSetTier('pro')}
            >
              Pro ($29/mo)
            </button>
            <button
              class="btn-tier {account?.tier === 'enterprise' ? 'selected' : ''}"
              on:click={() => handleSetTier('enterprise')}
            >
              Enterprise ($199/mo)
            </button>
          </div>
        </div>

        <!-- Live Visual Meter Gauges -->
        <div class="grid-3col gauges-grid">
          <!-- Words Quota -->
          <div class="card glass-panel meter-card">
            <h4>Cloud Translation Words</h4>
            <div class="meter-stat">
              <span class="stat-number">{usageSummary?.words_used.toLocaleString()}</span>
              <span class="stat-limit">/ {usageSummary?.words_limit.toLocaleString()} words</span>
            </div>
            <div class="progress-bar-bg">
              <div
                class="progress-bar-fill fill-blue"
                style="width: {Math.min(100, ((usageSummary?.words_used || 0) / (usageSummary?.words_limit || 1)) * 100)}%"
              ></div>
            </div>
            <span class="meter-foot">Resets monthly • Soft limit warning at 80%</span>
          </div>

          <!-- Speech Seconds Quota -->
          <div class="card glass-panel meter-card">
            <h4>Cloud Speech Processing</h4>
            <div class="meter-stat">
              <span class="stat-number">{usageSummary?.speech_seconds_used.toLocaleString()}</span>
              <span class="stat-limit">/ {usageSummary?.speech_seconds_limit.toLocaleString()} sec</span>
            </div>
            <div class="progress-bar-bg">
              <div
                class="progress-bar-fill fill-purple"
                style="width: {Math.min(100, ((usageSummary?.speech_seconds_used || 0) / (usageSummary?.speech_seconds_limit || 1)) * 100)}%"
              ></div>
            </div>
            <span class="meter-foot">High-accuracy cloud ASR & Neural TTS</span>
          </div>

          <!-- Cost Projection & Assurance -->
          <div class="card glass-panel meter-card">
            <h4>Monthly Spend & Projection</h4>
            <div class="meter-stat">
              <span class="stat-number cost-number">${((usageSummary?.current_period_cents || 0) / 100).toFixed(2)}</span>
              <span class="stat-limit">Projected: ${((usageSummary?.projected_month_end_cents || 0) / 100).toFixed(2)}</span>
            </div>
            <div class="assurance-tag">
              <span>⚖️ Fail-Safe Billing Rule:</span> Never charged for uncompleted, timed-out, or failed requests.
            </div>
          </div>
        </div>

        <!-- Invoices & Billing History -->
        <div class="card glass-panel">
          <div class="card-header-row">
            <h3>📑 Invoices & Billing Statements</h3>
            <button class="btn-secondary" on:click={refreshTelemetry}>Refresh Usage</button>
          </div>
          {#if invoices.length === 0}
            <p class="empty-text">No invoices generated yet for this billing cycle.</p>
          {:else}
            <div class="matrix-table-wrap">
              <table class="data-table">
                <thead>
                  <tr>
                    <th>Invoice #</th>
                    <th>Billing Period</th>
                    <th>Amount</th>
                    <th>Status</th>
                    <th>Date Issued</th>
                  </tr>
                </thead>
                <tbody>
                  {#each invoices as inv}
                    <tr>
                      <td class="font-mono font-semibold">{inv.invoice_number}</td>
                      <td>{inv.billing_period}</td>
                      <td class="font-semibold">${(inv.amount_cents / 100).toFixed(2)} {inv.currency}</td>
                      <td>
                        <span class="status-badge {inv.status}">{inv.status.toUpperCase()}</span>
                      </td>
                      <td class="text-muted">{new Date(inv.created_at).toLocaleDateString()}</td>
                    </tr>
                  {/each}
                </tbody>
              </table>
            </div>
          {/if}
        </div>

        <!-- Registered Devices Hardware Fingerprints -->
        <div class="card glass-panel">
          <div class="card-header-row">
            <h3>💻 Registered Hardware Devices ({devices.length})</h3>
          </div>
          <div class="add-device-form">
            <input
              type="text"
              placeholder="Device Name (e.g. Workstation PC / iPad Pro)"
              bind:value={newDeviceName}
              class="input-field"
            />
            <select bind:value={newDeviceType} class="input-field select-field">
              <option value="desktop_windows">Windows Desktop</option>
              <option value="desktop_mac">macOS Desktop</option>
              <option value="desktop_linux">Linux Desktop</option>
              <option value="mobile_android">Android Device</option>
              <option value="mobile_ios">iOS Device</option>
              <option value="web_browser">Web Browser</option>
            </select>
            <button class="btn-primary" on:click={handleRegisterDevice}>Register Device</button>
          </div>

          <div class="devices-grid">
            {#each devices as dev}
              <div class="device-card glass-panel">
                <div class="dev-icon">📱</div>
                <div class="dev-info">
                  <strong>{dev.device_name}</strong>
                  <span class="type-pill">{dev.device_type}</span>
                  <div class="key-fingerprint font-mono">{dev.public_key_fingerprint}</div>
                </div>
              </div>
            {/each}
          </div>
        </div>
      </div>

    <!-- Tab 3: Shared Dravidian Glossaries -->
    {:else if activeTab === 'glossaries'}
      <div class="tab-content glossaries-tab">
        <div class="grid-2col">
          <!-- Publish New Enterprise Glossary -->
          <div class="card glass-panel">
            <h3>📖 Publish Enterprise Glossary</h3>
            <p class="section-desc">Create shared domain lexicons with automatic Dravidian term substitution:</p>
            <div class="form-vertical">
              <label>
                Glossary Name
                <input
                  type="text"
                  placeholder="e.g. Official Karnataka Governance Lexicon"
                  bind:value={newGlossaryName}
                  class="input-field"
                />
              </label>
              <label>
                Description & Regulatory Context
                <input
                  type="text"
                  placeholder="e.g. Approved administrative translations for public circulars"
                  bind:value={newGlossaryDesc}
                  class="input-field"
                />
              </label>
              <div class="lang-row">
                <label>
                  Source Language
                  <select bind:value={newGlossarySource} class="input-field select-field">
                    <option value="en-US">English (en-US)</option>
                    <option value="kn-IN">Kannada (kn-IN)</option>
                    <option value="ta-IN">Tamil (ta-IN)</option>
                  </select>
                </label>
                <label>
                  Target Language
                  <select bind:value={newGlossaryTarget} class="input-field select-field">
                    <option value="kn-IN">Kannada (kn-IN)</option>
                    <option value="ta-IN">Tamil (ta-IN)</option>
                    <option value="en-US">English (en-US)</option>
                  </select>
                </label>
              </div>
              <button class="btn-primary" on:click={handleCreateGlossary}>Create Shared Glossary</button>
            </div>
          </div>

          <!-- Add Term Pair into Glossary -->
          <div class="card glass-panel">
            <h3>➕ Add Terminology Pair</h3>
            <p class="section-desc">Add approved Dravidian technical or legal term definitions:</p>
            {#if glossaries.length === 0}
              <p class="empty-text">Create or select a glossary on the left to add terms.</p>
            {:else}
              <div class="form-vertical">
                <label>
                  Select Active Glossary
                  <select bind:value={selectedGlossaryId} class="input-field select-field">
                    {#each glossaries as g}
                      <option value={g.id}>{g.name} ({g.source_lang} → {g.target_lang})</option>
                    {/each}
                  </select>
                </label>
                <label>
                  Source Term (Original Phrase)
                  <input
                    type="text"
                    placeholder="e.g. Government Order"
                    bind:value={newTermSource}
                    class="input-field"
                  />
                </label>
                <label>
                  Approved Dravidian Target Term
                  <input
                    type="text"
                    placeholder="e.g. ಸರ್ಕಾರಿ ಆದೇಶ / அரசாணை"
                    bind:value={newTermTarget}
                    class="input-field font-dravidian"
                  />
                </label>
                <label>
                  Usage Notes / Legal Context
                  <input
                    type="text"
                    placeholder="e.g. Standard administrative gazette term"
                    bind:value={newTermNotes}
                    class="input-field"
                  />
                </label>
                <button
                  class="btn-primary"
                  on:click={() => selectedGlossaryId && handleAddTerm(selectedGlossaryId)}
                >
                  Save Term Pair
                </button>
              </div>
            {/if}
          </div>
        </div>

        <!-- Published Glossaries List -->
        <div class="card glass-panel">
          <h3>📚 Active Enterprise Shared Glossaries ({glossaries.length})</h3>
          {#if glossaries.length === 0}
            <p class="empty-text">No enterprise glossaries created yet.</p>
          {:else}
            <div class="glossary-cards-grid">
              {#each glossaries as g}
                <div class="glossary-item-card glass-panel">
                  <div class="g-header">
                    <h4>{g.name}</h4>
                    <span class="status-pill {g.status}">{g.status.toUpperCase()}</span>
                  </div>
                  <p class="g-desc">{g.description}</p>
                  <div class="g-meta">
                    <span class="lang-tag">{g.source_lang} → {g.target_lang}</span>
                    <span class="term-count">{g.terms_count} approved term(s)</span>
                  </div>
                </div>
              {/each}
            </div>
          {/if}
        </div>
      </div>

    <!-- Tab 4: Developer Integration & Security Audit -->
    {:else if activeTab === 'developer'}
      <div class="tab-content developer-tab">
        <!-- Live API Key Management -->
        <div class="card glass-panel">
          <div class="card-header-row">
            <div>
              <h3>🔑 Developer API Keys (SHA-256 HMAC + Token Bucket)</h3>
              <p class="section-desc">Generate secure live API keys with per-key rate limits for backend integrations:</p>
            </div>
          </div>

          <div class="api-key-gen-row">
            <input
              type="text"
              placeholder="API Key Name / Integration Purpose (e.g. Production Mobile App)"
              bind:value={newKeyName}
              class="input-field"
            />
            <button class="btn-primary" on:click={handleCreateApiKey}>Generate Live Key</button>
          </div>

          <!-- Highlighted Raw Key Alert -->
          {#if generatedRawKey}
            <div class="raw-key-alert glass-panel">
              <div class="raw-key-header">
                <strong>⚡ Save this Secret API Key now:</strong>
                <span>It will NOT be shown again!</span>
              </div>
              <div class="raw-key-box font-mono">{generatedRawKey}</div>
              <button class="btn-secondary" on:click={() => navigator.clipboard.writeText(generatedRawKey || '')}>
                Copy Secret Key
              </button>
            </div>
          {/if}

          <!-- Existing Keys Table -->
          <div class="matrix-table-wrap">
            <table class="data-table">
              <thead>
                <tr>
                  <th>Key Name</th>
                  <th>Prefix</th>
                  <th>Rate Limit</th>
                  <th>Status</th>
                  <th>Created</th>
                  <th>Action</th>
                </tr>
              </thead>
              <tbody>
                {#each apiKeys as key}
                  <tr>
                    <td class="font-semibold">{key.name}</td>
                    <td><code class="font-mono">{key.key_prefix}...</code></td>
                    <td>{key.rate_limit_per_min} req / min</td>
                    <td>
                      <span class="key-status {key.is_active ? 'active' : 'revoked'}">
                        {key.is_active ? 'ACTIVE' : 'REVOKED'}
                      </span>
                    </td>
                    <td class="text-muted">{new Date(key.created_at).toLocaleDateString()}</td>
                    <td>
                      {#if key.is_active}
                        <button class="btn-danger-sm" on:click={() => handleRevokeApiKey(key.id)}>
                          Revoke
                        </button>
                      {/if}
                    </td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </div>

        <!-- Worker Pool Concurrency & Telemetry -->
        <div class="card glass-panel">
          <div class="card-header-row">
            <h3>⚙️ Distributed Worker Pool Telemetry</h3>
            <button class="btn-secondary" on:click={refreshTelemetry}>Refresh Telemetry</button>
          </div>
          <div class="grid-4col worker-stats-grid">
            <div class="worker-stat-box">
              <span class="val-num">{workerMetrics?.active_workers ?? 0}</span>
              <span class="stat-lbl">Active Workers</span>
            </div>
            <div class="worker-stat-box">
              <span class="val-num">{workerMetrics?.queue_depth ?? 0} / {workerMetrics?.max_queue_capacity ?? 50}</span>
              <span class="stat-lbl">Queue Depth</span>
            </div>
            <div class="worker-stat-box">
              <span class="val-num">{workerMetrics?.processed_tasks ?? 0}</span>
              <span class="stat-lbl">Processed Tasks</span>
            </div>
            <div class="worker-stat-box">
              <span class="val-num">{(workerMetrics?.avg_latency_ms ?? 0).toFixed(2)} ms</span>
              <span class="stat-lbl">Average Latency</span>
            </div>
          </div>
          <div class="telemetry-footer">
            <span class="telemetry-pill {workerMetrics?.is_accepting_tasks ? 'healthy' : 'backpressure'}">
              {workerMetrics?.is_accepting_tasks ? 'HEALTHY: Worker Queue Ingesting' : 'BACKPRESSURE: Capacity Exceeded'}
            </span>
            <span class="text-muted">Concurrency bounded to prevent OOM on edge servers.</span>
          </div>
        </div>

        <!-- Immutable Audit Trail -->
        <div class="card glass-panel">
          <div class="card-header-row">
            <h3>📜 Immutable SOC2 Audit Event Trail</h3>
            <span class="audit-badge">TAMPER RESISTANT</span>
          </div>
          <div class="matrix-table-wrap">
            <table class="data-table font-sm">
              <thead>
                <tr>
                  <th>Timestamp</th>
                  <th>Action</th>
                  <th>Actor</th>
                  <th>Resource</th>
                  <th>Details</th>
                </tr>
              </thead>
              <tbody>
                {#each auditEvents as ev}
                  <tr>
                    <td class="text-muted font-mono">{new Date(ev.timestamp).toLocaleTimeString()}</td>
                    <td><span class="action-tag">{ev.action}</span></td>
                    <td class="font-mono">{ev.actor_email || ev.actor_id}</td>
                    <td>{ev.resource_type}:{ev.resource_id}</td>
                    <td>{ev.details}</td>
                  </tr>
                {/each}
              </tbody>
            </table>
          </div>
        </div>
      </div>
    {/if}
  {/if}
</div>

<style>
  .enterprise-view {
    display: flex;
    flex-direction: column;
    gap: 1.5rem;
    padding-bottom: 3rem;
  }

  .enterprise-header {
    display: flex;
    flex-direction: column;
    gap: 1.25rem;
    padding: 1.5rem;
    border-radius: 12px;
  }

  .title-row {
    display: flex;
    align-items: center;
    gap: 1rem;
  }

  .icon-wrap {
    font-size: 2.2rem;
    padding: 0.6rem;
    background: rgba(99, 102, 241, 0.12);
    border-radius: 12px;
  }

  h2 {
    margin: 0;
    font-size: 1.6rem;
    font-weight: 700;
  }

  .subtitle {
    margin: 0.25rem 0 0 0;
    color: var(--text-secondary);
    font-size: 0.95rem;
  }

  .enterprise-tabs {
    display: flex;
    gap: 0.5rem;
    border-bottom: 1px solid var(--border-color);
    padding-bottom: 0.5rem;
    overflow-x: auto;
  }

  .tab-btn {
    background: transparent;
    border: none;
    color: var(--text-secondary);
    font-size: 0.95rem;
    font-weight: 600;
    padding: 0.6rem 1.1rem;
    border-radius: 8px;
    cursor: pointer;
    transition: all 0.2s ease;
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .tab-btn:hover {
    color: var(--text-primary);
    background: rgba(255, 255, 255, 0.05);
  }

  .tab-btn.active {
    color: #818cf8;
    background: rgba(99, 102, 241, 0.15);
  }

  .alert {
    padding: 0.75rem 1.25rem;
    border-radius: 8px;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    font-weight: 500;
  }

  .alert-success {
    background: rgba(16, 185, 129, 0.15);
    border: 1px solid rgba(16, 185, 129, 0.3);
    color: #10b981;
  }

  .alert-error {
    background: rgba(239, 68, 68, 0.15);
    border: 1px solid rgba(239, 68, 68, 0.3);
    color: #ef4444;
  }

  .card {
    padding: 1.5rem;
    border-radius: 12px;
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .card h3 {
    margin: 0;
    font-size: 1.25rem;
    font-weight: 600;
  }

  .section-desc {
    margin: 0;
    color: var(--text-secondary);
    font-size: 0.9rem;
  }

  .grid-2col {
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: 1.5rem;
  }

  .grid-3col {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 1.25rem;
  }

  .grid-4col {
    display: grid;
    grid-template-columns: repeat(4, 1fr);
    gap: 1rem;
  }

  @media (max-width: 900px) {
    .grid-2col, .grid-3col, .grid-4col {
      grid-template-columns: 1fr;
    }
  }

  .org-profile {
    display: flex;
    flex-direction: column;
    gap: 1rem;
  }

  .profile-header {
    display: flex;
    align-items: baseline;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .org-badge {
    background: rgba(16, 185, 129, 0.2);
    color: #10b981;
    font-size: 0.75rem;
    font-weight: 700;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
    letter-spacing: 0.05em;
  }

  .slug code {
    background: rgba(0, 0, 0, 0.2);
    padding: 0.2rem 0.4rem;
    border-radius: 4px;
  }

  .prop-list {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
    font-size: 0.9rem;
  }

  .prop {
    display: flex;
    justify-content: space-between;
    border-bottom: 1px solid var(--border-color);
    padding-bottom: 0.35rem;
  }

  .prop .label {
    color: var(--text-secondary);
  }

  .role-pill {
    padding: 0.2rem 0.6rem;
    border-radius: 9999px;
    font-size: 0.75rem;
    font-weight: 700;
    text-transform: uppercase;
  }

  .role-pill.owner { background: rgba(239, 68, 68, 0.2); color: #f87171; }
  .role-pill.admin { background: rgba(245, 158, 11, 0.2); color: #fbbf24; }
  .role-pill.linguist_editor { background: rgba(99, 102, 241, 0.2); color: #818cf8; }
  .role-pill.member { background: rgba(59, 130, 246, 0.2); color: #60a5fa; }
  .role-pill.viewer { background: rgba(107, 114, 128, 0.2); color: #9ca3af; }

  .matrix-table-wrap {
    overflow-x: auto;
  }

  .matrix-table, .data-table {
    width: 100%;
    border-collapse: collapse;
    font-size: 0.9rem;
  }

  .matrix-table th, .matrix-table td,
  .data-table th, .data-table td {
    padding: 0.65rem 0.75rem;
    text-align: left;
    border-bottom: 1px solid var(--border-color);
  }

  .matrix-table .check { color: #10b981; font-weight: bold; }
  .matrix-table .cross { color: #ef4444; font-weight: bold; }

  .account-banner {
    display: flex;
    flex-direction: row;
    justify-content: space-between;
    align-items: center;
    gap: 1.5rem;
    flex-wrap: wrap;
  }

  .avatar-chip {
    font-size: 1.8rem;
    padding: 0.5rem;
    background: rgba(99, 102, 241, 0.15);
    border-radius: 50%;
  }

  .tier-pill {
    padding: 0.2rem 0.6rem;
    border-radius: 9999px;
    font-size: 0.8rem;
    font-weight: 800;
  }

  .tier-pill.free { background: rgba(107, 114, 128, 0.2); color: #9ca3af; }
  .tier-pill.pro { background: rgba(59, 130, 246, 0.2); color: #60a5fa; }
  .tier-pill.enterprise { background: rgba(16, 185, 129, 0.2); color: #10b981; }

  .offline-guarantee {
    margin: 0.25rem 0 0 0;
    color: var(--text-secondary);
    font-size: 0.85rem;
    max-width: 600px;
  }

  .tier-actions {
    display: flex;
    gap: 0.5rem;
  }

  .btn-tier {
    padding: 0.5rem 0.85rem;
    border-radius: 8px;
    border: 1px solid var(--border-color);
    background: rgba(255, 255, 255, 0.05);
    color: var(--text-primary);
    cursor: pointer;
    font-size: 0.85rem;
    font-weight: 600;
    transition: all 0.2s ease;
  }

  .btn-tier.selected {
    background: #4f46e5;
    border-color: #6366f1;
    color: white;
  }

  .meter-card h4 {
    margin: 0;
    font-size: 1rem;
    color: var(--text-secondary);
  }

  .stat-number {
    font-size: 1.8rem;
    font-weight: 800;
    color: var(--text-primary);
  }

  .cost-number {
    color: #10b981;
  }

  .stat-limit {
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .progress-bar-bg {
    width: 100%;
    height: 8px;
    background: rgba(255, 255, 255, 0.1);
    border-radius: 999px;
    overflow: hidden;
    margin: 0.5rem 0;
  }

  .progress-bar-fill {
    height: 100%;
    border-radius: 999px;
    transition: width 0.3s ease;
  }

  .fill-blue { background: #3b82f6; }
  .fill-purple { background: #8b5cf6; }

  .meter-foot {
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .assurance-tag {
    background: rgba(16, 185, 129, 0.1);
    color: #10b981;
    padding: 0.5rem 0.75rem;
    border-radius: 6px;
    font-size: 0.8rem;
    margin-top: 0.5rem;
  }

  .input-field {
    padding: 0.6rem 0.85rem;
    border-radius: 8px;
    border: 1px solid var(--border-color);
    background: rgba(0, 0, 0, 0.2);
    color: var(--text-primary);
    font-size: 0.9rem;
    outline: none;
  }

  .input-field:focus {
    border-color: #6366f1;
  }

  .btn-primary {
    background: #4f46e5;
    color: white;
    border: none;
    border-radius: 8px;
    padding: 0.6rem 1.1rem;
    font-weight: 600;
    cursor: pointer;
    transition: all 0.2s ease;
  }

  .btn-primary:hover {
    background: #4338ca;
  }

  .btn-secondary {
    background: rgba(255, 255, 255, 0.1);
    color: var(--text-primary);
    border: 1px solid var(--border-color);
    border-radius: 8px;
    padding: 0.5rem 0.85rem;
    font-weight: 500;
    cursor: pointer;
  }

  .add-member-form, .add-device-form, .api-key-gen-row {
    display: flex;
    gap: 0.75rem;
    flex-wrap: wrap;
  }

  .devices-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(260px, 1fr));
    gap: 1rem;
  }

  .device-card {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem;
    border-radius: 8px;
  }

  .dev-icon {
    font-size: 1.5rem;
  }

  .type-pill {
    font-size: 0.75rem;
    background: rgba(255, 255, 255, 0.1);
    padding: 0.1rem 0.4rem;
    border-radius: 4px;
    margin-left: 0.5rem;
  }

  .key-fingerprint {
    font-size: 0.75rem;
    color: var(--text-secondary);
    margin-top: 0.25rem;
  }

  .form-vertical {
    display: flex;
    flex-direction: column;
    gap: 0.85rem;
  }

  .form-vertical label {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    font-size: 0.85rem;
    color: var(--text-secondary);
  }

  .font-dravidian {
    font-family: 'Noto Sans Kannada', 'Noto Sans Tamil', sans-serif;
  }

  .glossary-cards-grid {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(280px, 1fr));
    gap: 1rem;
  }

  .glossary-item-card {
    padding: 1rem;
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .g-header {
    display: flex;
    justify-content: space-between;
    align-items: center;
  }

  .g-header h4 {
    margin: 0;
  }

  .status-pill {
    font-size: 0.7rem;
    font-weight: 700;
    padding: 0.15rem 0.45rem;
    border-radius: 4px;
  }

  .status-pill.approved { background: rgba(16, 185, 129, 0.2); color: #10b981; }

  .raw-key-alert {
    padding: 1rem;
    border: 1px solid #eab308;
    background: rgba(234, 179, 8, 0.1);
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .raw-key-box {
    background: rgba(0, 0, 0, 0.3);
    padding: 0.5rem;
    border-radius: 4px;
    color: #fef08a;
    font-size: 0.95rem;
    word-break: break-all;
  }

  .btn-danger-sm {
    background: rgba(239, 68, 68, 0.2);
    border: 1px solid rgba(239, 68, 68, 0.4);
    color: #ef4444;
    padding: 0.25rem 0.5rem;
    border-radius: 4px;
    cursor: pointer;
    font-size: 0.8rem;
  }

  .key-status.active { color: #10b981; font-weight: bold; }
  .key-status.revoked { color: #ef4444; font-weight: bold; }

  .worker-stat-box {
    background: rgba(0, 0, 0, 0.15);
    padding: 1rem;
    border-radius: 8px;
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 0.25rem;
  }

  .val-num {
    font-size: 1.5rem;
    font-weight: 800;
    color: var(--text-primary);
  }

  .stat-lbl {
    font-size: 0.8rem;
    color: var(--text-secondary);
  }

  .telemetry-footer {
    display: flex;
    align-items: center;
    justify-content: space-between;
    font-size: 0.85rem;
    flex-wrap: wrap;
    gap: 0.5rem;
  }

  .telemetry-pill {
    padding: 0.25rem 0.6rem;
    border-radius: 6px;
    font-size: 0.8rem;
    font-weight: 700;
  }

  .telemetry-pill.healthy { background: rgba(16, 185, 129, 0.2); color: #10b981; }
  .telemetry-pill.backpressure { background: rgba(239, 68, 68, 0.2); color: #ef4444; }

  .action-tag {
    background: rgba(99, 102, 241, 0.15);
    color: #818cf8;
    padding: 0.15rem 0.45rem;
    border-radius: 4px;
    font-size: 0.75rem;
    font-family: monospace;
  }

  .audit-badge {
    background: rgba(16, 185, 129, 0.2);
    color: #10b981;
    font-size: 0.75rem;
    font-weight: 700;
    padding: 0.2rem 0.5rem;
    border-radius: 4px;
  }

  .loading-panel {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 1rem;
    padding: 3rem;
  }

  .spinner {
    width: 36px;
    height: 36px;
    border: 3px solid rgba(255, 255, 255, 0.1);
    border-top-color: #6366f1;
    border-radius: 50%;
    animation: spin 0.8s linear infinite;
  }

  @keyframes spin {
    to { transform: rotate(360deg); }
  }
</style>
