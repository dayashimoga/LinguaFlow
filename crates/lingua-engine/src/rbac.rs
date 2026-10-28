use lingua_core::{LinguaError, OrgGlossaryTerm, OrgRole};

/// Role-Based Access Control & Multi-Tenant Isolation Engine (Phase 8.3).
pub struct RbacController;

impl RbacController {
    /// Authorize whether an organization role can perform a privileged action.
    pub fn authorize_action(role: OrgRole, action: &str) -> Result<(), LinguaError> {
        let authorized = match action {
            "manage_team" | "invite_member" | "remove_member" | "change_role" => {
                role.can_manage_team()
            }
            "create_glossary" | "edit_glossary" | "add_term" | "delete_term" => {
                role.can_edit_glossary()
            }
            "manage_api_keys" | "revoke_api_key" => role.can_manage_api_keys(),
            "view_glossary" | "view_members" | "translate_with_glossary" => true,
            _ => matches!(role, OrgRole::Owner | OrgRole::Admin),
        };

        if authorized {
            Ok(())
        } else {
            Err(LinguaError::Forbidden(format!(
                "Role '{:?}' is not permitted to perform '{action}'",
                role
            )))
        }
    }

    /// Strict Multi-Tenant Isolation Enforcement:
    /// Ensures requests cannot access or cross-contaminate resources across organizations.
    pub fn enforce_tenant_isolation(
        actor_org: Option<&str>,
        target_org: Option<&str>,
    ) -> Result<(), LinguaError> {
        match (actor_org, target_org) {
            (Some(a), Some(t)) if a == t => Ok(()),
            (None, None) => Ok(()),
            (Some(a), Some(t)) => Err(LinguaError::Forbidden(format!(
                "Cross-tenant isolation violation: Actor org '{a}' cannot access target org '{t}'"
            ))),
            (None, Some(t)) => Err(LinguaError::Unauthorized(format!(
                "Authentication required for organization '{t}'"
            ))),
            (Some(a), None) => Err(LinguaError::Forbidden(format!(
                "Tenant '{a}' cannot perform un-scoped operations"
            ))),
        }
    }

    /// Apply shared enterprise glossary term substitutions to translated text.
    pub fn apply_org_glossaries(text: &str, terms: &[OrgGlossaryTerm]) -> (String, Vec<String>) {
        let mut result = text.to_string();
        let mut applied = Vec::new();

        for term in terms {
            let replaced = if term.case_sensitive {
                if result.contains(&term.source_term) {
                    result = result.replace(&term.source_term, &term.target_term);
                    true
                } else {
                    false
                }
            } else {
                let lower_res = result.to_lowercase();
                let lower_src = term.source_term.to_lowercase();
                if let Some(pos) = lower_res.find(&lower_src) {
                    result.replace_range(pos..(pos + term.source_term.len()), &term.target_term);
                    true
                } else {
                    false
                }
            };

            if replaced {
                applied.push(format!("{} -> {}", term.source_term, term.target_term));
            }
        }

        (result, applied)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rbac_permissions() {
        assert!(RbacController::authorize_action(OrgRole::Owner, "manage_team").is_ok());
        assert!(RbacController::authorize_action(OrgRole::Admin, "invite_member").is_ok());
        assert!(RbacController::authorize_action(OrgRole::Member, "invite_member").is_err());
        assert!(RbacController::authorize_action(OrgRole::LinguistEditor, "add_term").is_ok());
        assert!(RbacController::authorize_action(OrgRole::Viewer, "add_term").is_err());
        assert!(RbacController::authorize_action(OrgRole::Viewer, "view_glossary").is_ok());
    }

    #[test]
    fn test_tenant_isolation_boundary() {
        assert!(RbacController::enforce_tenant_isolation(Some("org-1"), Some("org-1")).is_ok());
        assert!(RbacController::enforce_tenant_isolation(Some("org-1"), Some("org-2")).is_err());
        assert!(RbacController::enforce_tenant_isolation(None, Some("org-1")).is_err());
    }

    #[test]
    fn test_shared_glossary_replacement() {
        let terms = vec![OrgGlossaryTerm {
            id: "t1".into(),
            glossary_id: "g1".into(),
            source_term: "cloud storage".into(),
            target_term: "ಮೋಡ ಸಂಗ್ರಹಣೆ".into(),
            notes: None,
            case_sensitive: false,
        }];

        let (replaced, applied) = RbacController::apply_org_glossaries(
            "We recommend Cloud Storage for all backups.",
            &terms,
        );

        assert_eq!(replaced, "We recommend ಮೋಡ ಸಂಗ್ರಹಣೆ for all backups.");
        assert_eq!(applied.len(), 1);
    }
}
