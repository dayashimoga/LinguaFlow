use chrono::{DateTime, Utc};
use lingua_core::{
    CircuitBreakerStatus, CircuitState, DomainGlossaryEntry, EvaluationReport, LanguageCode,
    LinguaError,
};
use rusqlite::{params, Connection};
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Clone)]
pub struct GlossaryRepository {
    conn: Arc<Mutex<Connection>>,
}

impl GlossaryRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn list_entries(
        &self,
        domain: Option<&str>,
        source_lang: Option<LanguageCode>,
        target_lang: Option<LanguageCode>,
    ) -> Result<Vec<DomainGlossaryEntry>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        let mut query = "SELECT id, domain, source_term, target_term, source_lang, target_lang, is_exact, created_at FROM domain_glossaries WHERE 1=1".to_string();
        let mut params_vec: Vec<Box<dyn rusqlite::ToSql>> = Vec::new();

        if let Some(d) = domain {
            query.push_str(" AND domain = ?");
            params_vec.push(Box::new(d.to_string()));
        }
        if let Some(sl) = source_lang {
            query.push_str(" AND source_lang = ?");
            params_vec.push(Box::new(sl.as_bcp47().to_string()));
        }
        if let Some(tl) = target_lang {
            query.push_str(" AND target_lang = ?");
            params_vec.push(Box::new(tl.as_bcp47().to_string()));
        }

        query.push_str(" ORDER BY domain, source_term ASC");

        let mut stmt = conn
            .prepare(&query)
            .map_err(|e| LinguaError::Database(format!("Failed to prepare query: {e}")))?;

        let params_slice: Vec<&dyn rusqlite::ToSql> =
            params_vec.iter().map(|b| b.as_ref()).collect();

        let rows = stmt
            .query_map(params_slice.as_slice(), |row| {
                let sl_str: String = row.get(4)?;
                let tl_str: String = row.get(5)?;
                let created_at_str: String = row.get(7)?;

                Ok((
                    row.get::<_, String>(0)?,
                    row.get::<_, String>(1)?,
                    row.get::<_, String>(2)?,
                    row.get::<_, String>(3)?,
                    sl_str,
                    tl_str,
                    row.get::<_, i64>(6)? != 0,
                    created_at_str,
                ))
            })
            .map_err(|e| LinguaError::Database(format!("Query failed: {e}")))?;

        let mut entries = Vec::new();
        for item in rows {
            let (id, dom, s_term, t_term, sl_str, tl_str, is_exact, c_str) =
                item.map_err(|e| LinguaError::Database(format!("Row read error: {e}")))?;

            let sl = LanguageCode::parse(&sl_str).unwrap_or(LanguageCode::EnUs);
            let tl = LanguageCode::parse(&tl_str).unwrap_or(LanguageCode::KnIn);
            let created_at = DateTime::parse_from_rfc3339(&c_str)
                .map(|dt| dt.with_timezone(&Utc))
                .unwrap_or_else(|_| Utc::now());

            entries.push(DomainGlossaryEntry {
                id,
                domain: dom,
                source_term: s_term,
                target_term: t_term,
                source_lang: sl,
                target_lang: tl,
                is_exact,
                created_at,
            });
        }

        Ok(entries)
    }

    pub fn insert_entry(&self, entry: &DomainGlossaryEntry) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        conn.execute(
            "INSERT OR REPLACE INTO domain_glossaries 
             (id, domain, source_term, target_term, source_lang, target_lang, is_exact, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?)",
            params![
                entry.id,
                entry.domain,
                entry.source_term,
                entry.target_term,
                entry.source_lang.as_bcp47(),
                entry.target_lang.as_bcp47(),
                if entry.is_exact { 1 } else { 0 },
                entry.created_at.to_rfc3339()
            ],
        )
        .map_err(|e| LinguaError::Database(format!("Failed to insert glossary entry: {e}")))?;

        Ok(())
    }

    pub fn delete_entry(&self, id: &str) -> Result<bool, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        let affected = conn
            .execute("DELETE FROM domain_glossaries WHERE id = ?", params![id])
            .map_err(|e| LinguaError::Database(format!("Failed to delete glossary entry: {e}")))?;

        Ok(affected > 0)
    }

    pub fn save_evaluation_report(&self, report: &EvaluationReport) -> Result<String, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        let id = format!("eval-report-{}", Uuid::new_v4());
        let report_json = serde_json::to_string(report)
            .map_err(|e| LinguaError::Internal(format!("Failed to serialize report: {e}")))?;

        conn.execute(
            "INSERT INTO evaluation_reports (id, report_json, mean_chrf, mean_wer, mean_cer, created_at)
             VALUES (?, ?, ?, ?, ?, ?)",
            params![
                id,
                report_json,
                report.mean_chrf,
                report.mean_wer,
                report.mean_cer,
                report.evaluated_at.to_rfc3339()
            ],
        )
        .map_err(|e| LinguaError::Database(format!("Failed to save evaluation report: {e}")))?;

        Ok(id)
    }

    pub fn get_latest_evaluation_report(&self) -> Result<Option<EvaluationReport>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        let mut stmt = conn
            .prepare("SELECT report_json FROM evaluation_reports ORDER BY created_at DESC LIMIT 1")
            .map_err(|e| LinguaError::Database(format!("Failed to prepare query: {e}")))?;

        let mut rows = stmt
            .query([])
            .map_err(|e| LinguaError::Database(format!("Query failed: {e}")))?;

        if let Some(row) = rows
            .next()
            .map_err(|e| LinguaError::Database(e.to_string()))?
        {
            let json_str: String = row
                .get(0)
                .map_err(|e| LinguaError::Database(e.to_string()))?;
            let report: EvaluationReport = serde_json::from_str(&json_str)
                .map_err(|e| LinguaError::Internal(format!("Failed to deserialize report: {e}")))?;
            Ok(Some(report))
        } else {
            Ok(None)
        }
    }

    pub fn save_circuit_breaker_metrics(
        &self,
        status: &CircuitBreakerStatus,
    ) -> Result<(), LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        conn.execute(
            "INSERT OR REPLACE INTO circuit_breaker_metrics
             (provider_id, state, failure_count, success_count, total_requests, last_failure_ms, updated_at)
             VALUES (?, ?, ?, ?, ?, ?, datetime('now'))",
            params![
                status.provider_id,
                status.state.as_str(),
                status.failure_count,
                status.success_count,
                status.total_requests,
                status.last_failure_ms.map(|v| v as i64)
            ],
        )
        .map_err(|e| LinguaError::Database(format!("Failed to save circuit breaker metric: {e}")))?;

        Ok(())
    }

    pub fn list_circuit_breaker_metrics(&self) -> Result<Vec<CircuitBreakerStatus>, LinguaError> {
        let conn = self
            .conn
            .lock()
            .map_err(|e| LinguaError::Database(format!("Lock poisoned: {e}")))?;

        let mut stmt = conn
            .prepare("SELECT provider_id, state, failure_count, success_count, total_requests, last_failure_ms FROM circuit_breaker_metrics ORDER BY provider_id")
            .map_err(|e| LinguaError::Database(format!("Failed to prepare query: {e}")))?;

        let rows = stmt
            .query_map([], |row| {
                let state_str: String = row.get(1)?;
                let state = match state_str.as_str() {
                    "open" => CircuitState::Open,
                    "half_open" => CircuitState::HalfOpen,
                    _ => CircuitState::Closed,
                };
                let last_fail: Option<i64> = row.get(5)?;

                Ok(CircuitBreakerStatus {
                    provider_id: row.get(0)?,
                    state,
                    failure_count: row.get(2)?,
                    success_count: row.get(3)?,
                    total_requests: row.get(4)?,
                    last_failure_ms: last_fail.map(|v| v as u64),
                })
            })
            .map_err(|e| LinguaError::Database(format!("Query failed: {e}")))?;

        let mut results = Vec::new();
        for item in rows {
            results.push(item.map_err(|e| LinguaError::Database(e.to_string()))?);
        }
        Ok(results)
    }
}
