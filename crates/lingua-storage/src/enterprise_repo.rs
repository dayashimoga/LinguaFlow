use chrono::{DateTime, Utc};
use lingua_core::{
    AccountStatus, AccountTier, ApiKeyRecord, AuditEvent, BillingInvoiceRecord, DeviceRegistration,
    DeviceType, InvoiceLineItem, InvoiceStatus, LanguageCode, LinguaError, OrgGlossary,
    OrgGlossaryStatus, OrgGlossaryTerm, OrgMember, OrgRole, Organization, UsageRecord,
    UsageSummary, UserAccount,
};
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};

#[derive(Clone)]
pub struct EnterpriseRepository {
    conn: Arc<Mutex<Connection>>,
}

impl EnterpriseRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    // -------------------------------------------------------------------------
    // Accounts & Entitlements (Phase 8.1)
    // -------------------------------------------------------------------------

    pub fn get_or_create_anonymous_account(&self) -> Result<UserAccount, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock failure: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, email, display_name, tier, is_anonymous, status, created_at
                 FROM user_accounts
                 WHERE is_anonymous = 1
                 LIMIT 1",
            )
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        let res = stmt.query_row([], |row| {
            let tier_str: String = row.get(3)?;
            let status_str: String = row.get(5)?;
            let created_at_str: String = row.get(6)?;

            let tier = match tier_str.as_str() {
                "pro" => AccountTier::Pro,
                "enterprise" => AccountTier::Enterprise,
                _ => AccountTier::Free,
            };

            let status = match status_str.as_str() {
                "suspended" => AccountStatus::Suspended,
                "pending" => AccountStatus::PendingVerification,
                _ => AccountStatus::Active,
            };

            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(UserAccount {
                id: row.get(0)?,
                email: row.get(1)?,
                display_name: row.get(2)?,
                tier,
                is_anonymous: row.get::<_, i32>(4)? != 0,
                status,
                created_at,
            })
        });

        match res {
            Ok(acc) => Ok(acc),
            Err(_) => {
                let id = "acc-anonymous-local";
                let email = "anonymous@local.linguaflow";
                let display_name = "Local Anonymous User";
                let now = Utc::now().to_rfc3339();

                conn.execute(
                    "INSERT OR IGNORE INTO user_accounts (id, email, display_name, tier, is_anonymous, status, created_at)
                     VALUES (?1, ?2, ?3, 'free', 1, 'active', ?4)",
                    params![id, email, display_name, now],
                )
                .map_err(|e| LinguaError::Database(e.to_string()))?;

                Ok(UserAccount {
                    id: id.into(),
                    email: email.into(),
                    display_name: display_name.into(),
                    tier: AccountTier::Free,
                    is_anonymous: true,
                    status: AccountStatus::Active,
                    created_at: Utc::now(),
                })
            }
        }
    }

    pub fn get_account(&self, account_id: &str) -> Result<UserAccount, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock failure: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, email, display_name, tier, is_anonymous, status, created_at
                 FROM user_accounts
                 WHERE id = ?1",
            )
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        stmt.query_row(params![account_id], |row| {
            let tier_str: String = row.get(3)?;
            let status_str: String = row.get(5)?;
            let created_at_str: String = row.get(6)?;

            let tier = match tier_str.as_str() {
                "pro" => AccountTier::Pro,
                "enterprise" => AccountTier::Enterprise,
                _ => AccountTier::Free,
            };

            let status = match status_str.as_str() {
                "suspended" => AccountStatus::Suspended,
                "pending" => AccountStatus::PendingVerification,
                _ => AccountStatus::Active,
            };

            let created_at = DateTime::parse_from_rfc3339(&created_at_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(UserAccount {
                id: row.get(0)?,
                email: row.get(1)?,
                display_name: row.get(2)?,
                tier,
                is_anonymous: row.get::<_, i32>(4)? != 0,
                status,
                created_at,
            })
        })
        .map_err(|_| LinguaError::NotFound(format!("Account '{account_id}' not found")))
    }

    pub fn update_account_tier(
        &self,
        account_id: &str,
        tier: AccountTier,
    ) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock failure: {e}")))?;

        conn.execute(
            "UPDATE user_accounts SET tier = ?1 WHERE id = ?2",
            params![tier.as_str(), account_id],
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

        Ok(())
    }

    // -------------------------------------------------------------------------
    // Device Registration (Phase 8.1)
    // -------------------------------------------------------------------------

    pub fn register_device(&self, device: &DeviceRegistration) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock failure: {e}")))?;

        let type_str = match device.device_type {
            DeviceType::DesktopWindows => "desktop_windows",
            DeviceType::DesktopMac => "desktop_mac",
            DeviceType::DesktopLinux => "desktop_linux",
            DeviceType::MobileAndroid => "mobile_android",
            DeviceType::MobileIos => "mobile_ios",
            DeviceType::WebBrowser => "web_browser",
        };

        conn.execute(
            "INSERT OR REPLACE INTO device_registrations
             (id, user_id, device_name, device_type, public_key_fingerprint, registered_at, last_seen_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                device.id,
                device.user_id,
                device.device_name,
                type_str,
                device.public_key_fingerprint,
                device.registered_at.to_rfc3339(),
                device.last_seen_at.to_rfc3339(),
            ],
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn list_devices(&self, user_id: &str) -> Result<Vec<DeviceRegistration>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock failure: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, user_id, device_name, device_type, public_key_fingerprint, registered_at, last_seen_at
                 FROM device_registrations
                 WHERE user_id = ?1
                 ORDER BY last_seen_at DESC",
            )
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![user_id], |row| {
                let type_str: String = row.get(3)?;
                let device_type = match type_str.as_str() {
                    "desktop_mac" => DeviceType::DesktopMac,
                    "desktop_linux" => DeviceType::DesktopLinux,
                    "mobile_android" => DeviceType::MobileAndroid,
                    "mobile_ios" => DeviceType::MobileIos,
                    "web_browser" => DeviceType::WebBrowser,
                    _ => DeviceType::DesktopWindows,
                };

                let reg_str: String = row.get(5)?;
                let seen_str: String = row.get(6)?;

                let registered_at = DateTime::parse_from_rfc3339(&reg_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                let last_seen_at = DateTime::parse_from_rfc3339(&seen_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(DeviceRegistration {
                    id: row.get(0)?,
                    user_id: row.get(1)?,
                    device_name: row.get(2)?,
                    device_type,
                    public_key_fingerprint: row.get(4)?,
                    registered_at,
                    last_seen_at,
                })
            })
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| LinguaError::Database(e.to_string()))?);
        }
        Ok(list)
    }

    // -------------------------------------------------------------------------
    // API Keys (Phase 8.4)
    // -------------------------------------------------------------------------

    pub fn create_api_key(&self, record: &ApiKeyRecord) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock failure: {e}")))?;

        conn.execute(
            "INSERT INTO api_keys
             (id, user_id, org_id, key_hash, key_prefix, name, rate_limit_per_min, is_active, created_at, expires_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                record.id,
                record.user_id,
                record.org_id,
                record.key_hash,
                record.key_prefix,
                record.name,
                record.rate_limit_per_min,
                if record.is_active { 1 } else { 0 },
                record.created_at.to_rfc3339(),
                record.expires_at.map(|dt| dt.to_rfc3339()),
            ],
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn find_api_key_by_hash(&self, key_hash: &str) -> Result<ApiKeyRecord, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock failure: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, user_id, org_id, key_hash, key_prefix, name, rate_limit_per_min, is_active, created_at, expires_at
                 FROM api_keys
                 WHERE key_hash = ?1 AND is_active = 1",
            )
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        stmt.query_row(params![key_hash], |row| {
            let created_str: String = row.get(8)?;
            let expires_opt: Option<String> = row.get(9)?;

            let created_at = DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            let expires_at = expires_opt.and_then(|s| {
                DateTime::parse_from_rfc3339(&s)
                    .map(|dt| dt.with_timezone(&Utc))
                    .ok()
            });

            Ok(ApiKeyRecord {
                id: row.get(0)?,
                user_id: row.get(1)?,
                org_id: row.get(2)?,
                key_hash: row.get(3)?,
                key_prefix: row.get(4)?,
                name: row.get(5)?,
                rate_limit_per_min: row.get(6)?,
                is_active: row.get::<_, i32>(7)? != 0,
                created_at,
                expires_at,
            })
        })
        .map_err(|_| LinguaError::ApiKeyNotFound(key_hash.to_string()))
    }

    pub fn list_api_keys(&self, user_id: &str) -> Result<Vec<ApiKeyRecord>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock failure: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, user_id, org_id, key_hash, key_prefix, name, rate_limit_per_min, is_active, created_at, expires_at
                 FROM api_keys
                 WHERE user_id = ?1
                 ORDER BY created_at DESC",
            )
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![user_id], |row| {
                let created_str: String = row.get(8)?;
                let expires_opt: Option<String> = row.get(9)?;

                let created_at = DateTime::parse_from_rfc3339(&created_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                let expires_at = expires_opt.and_then(|s| {
                    DateTime::parse_from_rfc3339(&s)
                        .map(|dt| dt.with_timezone(&Utc))
                        .ok()
                });

                Ok(ApiKeyRecord {
                    id: row.get(0)?,
                    user_id: row.get(1)?,
                    org_id: row.get(2)?,
                    key_hash: row.get(3)?,
                    key_prefix: row.get(4)?,
                    name: row.get(5)?,
                    rate_limit_per_min: row.get(6)?,
                    is_active: row.get::<_, i32>(7)? != 0,
                    created_at,
                    expires_at,
                })
            })
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| LinguaError::Database(e.to_string()))?);
        }
        Ok(list)
    }

    pub fn revoke_api_key(&self, key_id: &str) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock failure: {e}")))?;

        conn.execute(
            "UPDATE api_keys SET is_active = 0 WHERE id = ?1",
            params![key_id],
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

        Ok(())
    }

    // -------------------------------------------------------------------------
    // Organizations & Workspaces (Phase 8.3)
    // -------------------------------------------------------------------------

    pub fn create_organization(
        &self,
        org: &Organization,
        owner_id: &str,
    ) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock failure: {e}")))?;

        conn.execute(
            "INSERT INTO organizations
             (id, name, slug, max_seats, sso_enabled, data_retention_days, enforce_zero_transcript_logging, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                org.id,
                org.name,
                org.slug,
                org.max_seats,
                if org.sso_enabled { 1 } else { 0 },
                org.data_retention_days,
                if org.enforce_zero_transcript_logging { 1 } else { 0 },
                org.created_at.to_rfc3339(),
            ],
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

        // Add owner member
        let member_id = format!("mem-{}", uuid::Uuid::new_v4());
        conn.execute(
            "INSERT INTO org_members (id, org_id, user_id, email, display_name, role, joined_at)
             VALUES (?1, ?2, ?3, 'owner@org.local', 'Org Owner', 'owner', ?4)",
            params![member_id, org.id, owner_id, Utc::now().to_rfc3339()],
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn get_organization(&self, org_id: &str) -> Result<Organization, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock failure: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, name, slug, max_seats, sso_enabled, data_retention_days, enforce_zero_transcript_logging, created_at
                 FROM organizations
                 WHERE id = ?1",
            )
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        stmt.query_row(params![org_id], |row| {
            let created_str: String = row.get(7)?;
            let created_at = DateTime::parse_from_rfc3339(&created_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            Ok(Organization {
                id: row.get(0)?,
                name: row.get(1)?,
                slug: row.get(2)?,
                max_seats: row.get(3)?,
                sso_enabled: row.get::<_, i32>(4)? != 0,
                data_retention_days: row.get(5)?,
                enforce_zero_transcript_logging: row.get::<_, i32>(6)? != 0,
                created_at,
            })
        })
        .map_err(|_| LinguaError::OrganizationNotFound(org_id.to_string()))
    }

    pub fn add_org_member(&self, member: &OrgMember) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock failure: {e}")))?;

        let role_str = match member.role {
            OrgRole::Owner => "owner",
            OrgRole::Admin => "admin",
            OrgRole::LinguistEditor => "linguist_editor",
            OrgRole::Viewer => "viewer",
            OrgRole::Member => "member",
        };

        conn.execute(
            "INSERT INTO org_members (id, org_id, user_id, email, display_name, role, joined_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                member.id,
                member.org_id,
                member.user_id,
                member.email,
                member.display_name,
                role_str,
                member.joined_at.to_rfc3339(),
            ],
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn list_org_members(&self, org_id: &str) -> Result<Vec<OrgMember>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock failure: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, org_id, user_id, email, display_name, role, joined_at
                 FROM org_members
                 WHERE org_id = ?1
                 ORDER BY joined_at ASC",
            )
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![org_id], |row| {
                let role_str: String = row.get(5)?;
                let role = match role_str.as_str() {
                    "owner" => OrgRole::Owner,
                    "admin" => OrgRole::Admin,
                    "linguist_editor" => OrgRole::LinguistEditor,
                    "viewer" => OrgRole::Viewer,
                    _ => OrgRole::Member,
                };

                let joined_str: String = row.get(6)?;
                let joined_at = DateTime::parse_from_rfc3339(&joined_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(OrgMember {
                    id: row.get(0)?,
                    org_id: row.get(1)?,
                    user_id: row.get(2)?,
                    email: row.get(3)?,
                    display_name: row.get(4)?,
                    role,
                    joined_at,
                })
            })
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| LinguaError::Database(e.to_string()))?);
        }
        Ok(list)
    }

    // -------------------------------------------------------------------------
    // Shared Org Glossaries (Phase 8.3)
    // -------------------------------------------------------------------------

    pub fn create_org_glossary(&self, glossary: &OrgGlossary) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock failure: {e}")))?;

        let status_str = match glossary.status {
            OrgGlossaryStatus::Draft => "draft",
            OrgGlossaryStatus::Deprecated => "deprecated",
            OrgGlossaryStatus::Approved => "approved",
        };

        conn.execute(
            "INSERT INTO org_glossaries
             (id, org_id, name, description, source_lang, target_lang, status, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                glossary.id,
                glossary.org_id,
                glossary.name,
                glossary.description,
                glossary.source_lang.as_str(),
                glossary.target_lang.as_str(),
                status_str,
                glossary.created_at.to_rfc3339(),
                glossary.updated_at.to_rfc3339(),
            ],
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn add_org_glossary_term(&self, term: &OrgGlossaryTerm) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock failure: {e}")))?;

        conn.execute(
            "INSERT OR REPLACE INTO org_glossary_terms
             (id, glossary_id, source_term, target_term, notes, case_sensitive)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                term.id,
                term.glossary_id,
                term.source_term,
                term.target_term,
                term.notes,
                if term.case_sensitive { 1 } else { 0 },
            ],
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn list_org_glossaries(&self, org_id: &str) -> Result<Vec<OrgGlossary>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock failure: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT g.id, g.org_id, g.name, g.description, g.source_lang, g.target_lang, g.status,
                        (SELECT COUNT(*) FROM org_glossary_terms t WHERE t.glossary_id = g.id),
                        g.created_at, g.updated_at
                 FROM org_glossaries g
                 WHERE g.org_id = ?1
                 ORDER BY g.created_at DESC",
            )
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![org_id], |row| {
                let src_str: String = row.get(4)?;
                let tgt_str: String = row.get(5)?;
                let status_str: String = row.get(6)?;

                let status = match status_str.as_str() {
                    "draft" => OrgGlossaryStatus::Draft,
                    "deprecated" => OrgGlossaryStatus::Deprecated,
                    _ => OrgGlossaryStatus::Approved,
                };

                let created_str: String = row.get(8)?;
                let updated_str: String = row.get(9)?;

                let created_at = DateTime::parse_from_rfc3339(&created_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                let updated_at = DateTime::parse_from_rfc3339(&updated_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(OrgGlossary {
                    id: row.get(0)?,
                    org_id: row.get(1)?,
                    name: row.get(2)?,
                    description: row.get(3)?,
                    source_lang: LanguageCode::parse(&src_str).unwrap_or(LanguageCode::EnUs),
                    target_lang: LanguageCode::parse(&tgt_str).unwrap_or(LanguageCode::KnIn),
                    status,
                    terms_count: row.get(7)?,
                    created_at,
                    updated_at,
                })
            })
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| LinguaError::Database(e.to_string()))?);
        }
        Ok(list)
    }

    pub fn get_org_glossary_terms(
        &self,
        glossary_id: &str,
    ) -> Result<Vec<OrgGlossaryTerm>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock failure: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, glossary_id, source_term, target_term, notes, case_sensitive
                 FROM org_glossary_terms
                 WHERE glossary_id = ?1",
            )
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![glossary_id], |row| {
                Ok(OrgGlossaryTerm {
                    id: row.get(0)?,
                    glossary_id: row.get(1)?,
                    source_term: row.get(2)?,
                    target_term: row.get(3)?,
                    notes: row.get(4)?,
                    case_sensitive: row.get::<_, i32>(5)? != 0,
                })
            })
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| LinguaError::Database(e.to_string()))?);
        }
        Ok(list)
    }

    // -------------------------------------------------------------------------
    // Usage Metering (Phase 8.2)
    // -------------------------------------------------------------------------

    pub fn record_usage(&self, record: &UsageRecord) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock failure: {e}")))?;

        conn.execute(
            "INSERT INTO usage_records
             (id, account_id, org_id, usage_type, quantity, billable_cents, was_successful, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                record.id,
                record.account_id,
                record.org_id,
                record.usage_type.as_str(),
                record.quantity,
                record.billable_cents,
                if record.was_successful { 1 } else { 0 },
                record.timestamp.to_rfc3339(),
            ],
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn get_usage_summary(&self, account_id: &str) -> Result<UsageSummary, LinguaError> {
        let account = self.get_account(account_id)?;
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock failure: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT usage_type, SUM(quantity), SUM(billable_cents)
                 FROM usage_records
                 WHERE account_id = ?1 AND was_successful = 1
                 GROUP BY usage_type",
            )
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![account_id], |row| {
                let u_type: String = row.get(0)?;
                let qty: u64 = row.get::<_, Option<u64>>(1)?.unwrap_or(0);
                let cents: u64 = row.get::<_, Option<u64>>(2)?.unwrap_or(0);
                Ok((u_type, qty, cents))
            })
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        let mut words_used = 0;
        let mut speech_seconds_used = 0;
        let mut ocr_pages_used = 0;
        let mut current_period_cents = 0;

        for r in rows {
            let (u_type, qty, cents) = r.map_err(|e| LinguaError::Database(e.to_string()))?;
            match u_type.as_str() {
                "cloud_translation_words" => words_used += qty,
                "cloud_speech_seconds" => speech_seconds_used += qty,
                "cloud_ocr_pages" => ocr_pages_used += qty,
                _ => {}
            }
            current_period_cents += cents;
        }

        let words_limit = account.tier.max_monthly_words();
        let speech_seconds_limit = account.tier.max_monthly_audio_seconds();
        let ocr_pages_limit = match account.tier {
            AccountTier::Free => 5,
            AccountTier::Pro => 500,
            AccountTier::Enterprise => 100_000,
        };

        // Projected cents based on current run rate
        let projected_month_end_cents = (current_period_cents as f64 * 1.25) as u64;

        Ok(UsageSummary {
            account_id: account_id.to_string(),
            tier: account.tier,
            words_used,
            words_limit,
            speech_seconds_used,
            speech_seconds_limit,
            ocr_pages_used,
            ocr_pages_limit,
            current_period_cents,
            projected_month_end_cents,
        })
    }

    // -------------------------------------------------------------------------
    // Billing & Invoices (Phase 8.2)
    // -------------------------------------------------------------------------

    pub fn create_invoice(&self, invoice: &BillingInvoiceRecord) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock failure: {e}")))?;

        let status_str = match invoice.status {
            InvoiceStatus::Pending => "pending",
            InvoiceStatus::Refunded => "refunded",
            InvoiceStatus::Paid => "paid",
        };

        let items_json = serde_json::to_string(&invoice.line_items)
            .map_err(|e| LinguaError::Validation(format!("JSON serialization error: {e}")))?;

        conn.execute(
            "INSERT INTO billing_invoices
             (id, account_id, org_id, invoice_number, billing_period, amount_cents, currency, status, line_items_json, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                invoice.id,
                invoice.account_id,
                invoice.org_id,
                invoice.invoice_number,
                invoice.billing_period,
                invoice.amount_cents,
                invoice.currency,
                status_str,
                items_json,
                invoice.created_at.to_rfc3339(),
            ],
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn list_invoices(
        &self,
        account_id: &str,
    ) -> Result<Vec<BillingInvoiceRecord>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock failure: {e}")))?;

        let mut stmt = conn
            .prepare(
                "SELECT id, account_id, org_id, invoice_number, billing_period, amount_cents, currency, status, line_items_json, created_at
                 FROM billing_invoices
                 WHERE account_id = ?1
                 ORDER BY created_at DESC",
            )
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        let rows = stmt
            .query_map(params![account_id], |row| {
                let status_str: String = row.get(7)?;
                let status = match status_str.as_str() {
                    "pending" => InvoiceStatus::Pending,
                    "refunded" => InvoiceStatus::Refunded,
                    _ => InvoiceStatus::Paid,
                };

                let items_json: String = row.get(8)?;
                let line_items: Vec<InvoiceLineItem> =
                    serde_json::from_str(&items_json).unwrap_or_default();

                let created_str: String = row.get(9)?;
                let created_at = DateTime::parse_from_rfc3339(&created_str)
                    .map(|dt| dt.with_timezone(&Utc))
                    .unwrap_or_else(|_| Utc::now());

                Ok(BillingInvoiceRecord {
                    id: row.get(0)?,
                    account_id: row.get(1)?,
                    org_id: row.get(2)?,
                    invoice_number: row.get(3)?,
                    billing_period: row.get(4)?,
                    amount_cents: row.get(5)?,
                    currency: row.get(6)?,
                    status,
                    line_items,
                    created_at,
                })
            })
            .map_err(|e| LinguaError::Database(e.to_string()))?;

        let mut list = Vec::new();
        for r in rows {
            list.push(r.map_err(|e| LinguaError::Database(e.to_string()))?);
        }
        Ok(list)
    }

    // -------------------------------------------------------------------------
    // Audit Logging & SOC2 Events (Phase 8.3)
    // -------------------------------------------------------------------------

    pub fn record_audit_event(&self, event: &AuditEvent) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock failure: {e}")))?;

        conn.execute(
            "INSERT INTO audit_events
             (id, org_id, actor_id, actor_email, action, resource_type, resource_id, details, ip_address, timestamp)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                event.id,
                event.org_id,
                event.actor_id,
                event.actor_email,
                event.action,
                event.resource_type,
                event.resource_id,
                event.details,
                event.ip_address,
                event.timestamp.to_rfc3339(),
            ],
        )
        .map_err(|e| LinguaError::Database(e.to_string()))?;

        Ok(())
    }

    pub fn list_audit_events(
        &self,
        org_id: Option<&str>,
        limit: usize,
    ) -> Result<Vec<AuditEvent>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock failure: {e}")))?;

        let mut list = Vec::new();
        if let Some(oid) = org_id {
            let mut stmt = conn
                .prepare(
                    "SELECT id, org_id, actor_id, actor_email, action, resource_type, resource_id, details, ip_address, timestamp
                     FROM audit_events
                     WHERE org_id = ?1
                     ORDER BY timestamp DESC
                     LIMIT ?2",
                )
                .map_err(|e| LinguaError::Database(e.to_string()))?;

            let rows = stmt
                .query_map(params![oid, limit as i64], |row| {
                    let ts_str: String = row.get(9)?;
                    let timestamp = DateTime::parse_from_rfc3339(&ts_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now());

                    Ok(AuditEvent {
                        id: row.get(0)?,
                        org_id: row.get(1)?,
                        actor_id: row.get(2)?,
                        actor_email: row.get(3)?,
                        action: row.get(4)?,
                        resource_type: row.get(5)?,
                        resource_id: row.get(6)?,
                        details: row.get(7)?,
                        ip_address: row.get(8)?,
                        timestamp,
                    })
                })
                .map_err(|e| LinguaError::Database(e.to_string()))?;

            for r in rows {
                list.push(r.map_err(|e| LinguaError::Database(e.to_string()))?);
            }
        } else {
            let mut stmt = conn
                .prepare(
                    "SELECT id, org_id, actor_id, actor_email, action, resource_type, resource_id, details, ip_address, timestamp
                     FROM audit_events
                     ORDER BY timestamp DESC
                     LIMIT ?1",
                )
                .map_err(|e| LinguaError::Database(e.to_string()))?;

            let rows = stmt
                .query_map(params![limit as i64], |row| {
                    let ts_str: String = row.get(9)?;
                    let timestamp = DateTime::parse_from_rfc3339(&ts_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now());

                    Ok(AuditEvent {
                        id: row.get(0)?,
                        org_id: row.get(1)?,
                        actor_id: row.get(2)?,
                        actor_email: row.get(3)?,
                        action: row.get(4)?,
                        resource_type: row.get(5)?,
                        resource_id: row.get(6)?,
                        details: row.get(7)?,
                        ip_address: row.get(8)?,
                        timestamp,
                    })
                })
                .map_err(|e| LinguaError::Database(e.to_string()))?;

            for r in rows {
                list.push(r.map_err(|e| LinguaError::Database(e.to_string()))?);
            }
        }

        Ok(list)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::db::StorageService;

    #[test]
    fn test_anonymous_account_and_entitlements() {
        let storage = StorageService::open_in_memory().expect("in-memory db");
        let repo = storage.enterprise();

        let anon = repo.get_or_create_anonymous_account().expect("get anon");
        assert!(anon.is_anonymous);
        assert_eq!(anon.tier, AccountTier::Free);

        repo.update_account_tier(&anon.id, AccountTier::Pro)
            .expect("update tier");
        let updated = repo.get_account(&anon.id).expect("get updated");
        assert_eq!(updated.tier, AccountTier::Pro);
    }

    #[test]
    fn test_devices_and_api_keys() {
        let storage = StorageService::open_in_memory().expect("in-memory db");
        let repo = storage.enterprise();
        let anon = repo.get_or_create_anonymous_account().expect("get anon");

        let device = DeviceRegistration {
            id: "dev-01".into(),
            user_id: anon.id.clone(),
            device_name: "MacBook Pro".into(),
            device_type: DeviceType::DesktopMac,
            public_key_fingerprint: "SHA256:abc123xyz".into(),
            registered_at: Utc::now(),
            last_seen_at: Utc::now(),
        };
        repo.register_device(&device).expect("register device");

        let devices = repo.list_devices(&anon.id).expect("list devices");
        assert_eq!(devices.len(), 1);
        assert_eq!(devices[0].device_name, "MacBook Pro");

        let key = ApiKeyRecord {
            id: "key-01".into(),
            user_id: anon.id.clone(),
            org_id: None,
            key_hash: "hash_secret_key_123".into(),
            key_prefix: "lf_live_123".into(),
            name: "Test API Key".into(),
            rate_limit_per_min: 300,
            is_active: true,
            created_at: Utc::now(),
            expires_at: None,
        };
        repo.create_api_key(&key).expect("create key");

        let found = repo
            .find_api_key_by_hash("hash_secret_key_123")
            .expect("find key");
        assert_eq!(found.name, "Test API Key");
        assert_eq!(found.rate_limit_per_min, 300);
    }

    #[test]
    fn test_organizations_and_audit_logging() {
        let storage = StorageService::open_in_memory().expect("in-memory db");
        let repo = storage.enterprise();
        let anon = repo.get_or_create_anonymous_account().expect("get anon");

        let org = Organization {
            id: "org-acme".into(),
            name: "Acme Dravidian Translators".into(),
            slug: "acme-corp".into(),
            max_seats: 25,
            sso_enabled: true,
            data_retention_days: 180,
            enforce_zero_transcript_logging: true,
            created_at: Utc::now(),
        };
        repo.create_organization(&org, &anon.id)
            .expect("create org");

        let members = repo.list_org_members(&org.id).expect("list members");
        assert_eq!(members.len(), 1);
        assert_eq!(members[0].role, OrgRole::Owner);

        let audit = AuditEvent {
            id: "aud-01".into(),
            org_id: Some(org.id.clone()),
            actor_id: anon.id.clone(),
            actor_email: anon.email.clone(),
            action: "org.created".into(),
            resource_type: "organization".into(),
            resource_id: org.id.clone(),
            details: "Created Acme Dravidian Translators workspace".into(),
            ip_address: "127.0.0.1".into(),
            timestamp: Utc::now(),
        };
        repo.record_audit_event(&audit).expect("record audit");

        let events = repo
            .list_audit_events(Some(&org.id), 10)
            .expect("list audit");
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].action, "org.created");
    }
}
