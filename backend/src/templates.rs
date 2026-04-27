use crate::models::*;
use crate::db::*;
use chrono::Utc;

pub fn get_vault_templates() -> VaultTemplateList {
    VaultTemplateList {
        templates: vec![
            VaultTemplate {
                id: "simple-inheritance".to_string(),
                name: "Simple Inheritance".to_string(),
                description: "Basic vault for single beneficiary inheritance".to_string(),
                check_in_interval: 86400 * 30, // 30 days
                recommended_for: "Individual asset protection".to_string(),
            },
            VaultTemplate {
                id: "family-trust".to_string(),
                name: "Family Trust".to_string(),
                description: "Multi-beneficiary vault for family wealth distribution".to_string(),
                check_in_interval: 86400 * 90, // 90 days
                recommended_for: "Family wealth management".to_string(),
            },
            VaultTemplate {
                id: "business-succession".to_string(),
                name: "Business Succession".to_string(),
                description: "Vault for business continuity and succession planning".to_string(),
                check_in_interval: 86400 * 60, // 60 days
                recommended_for: "Business asset protection".to_string(),
            },
        ],
    }
}

pub fn create_vault_from_template(
    store: &VaultStore,
    template_id: &str,
    owner: String,
    beneficiary: String,
) -> Result<Vault, String> {
    let templates = get_vault_templates();
    let template = templates
        .templates
        .iter()
        .find(|t| t.id == template_id)
        .ok_or_else(|| "Template not found".to_string())?;

    let vault_id = uuid::Uuid::new_v4().to_string();
    let vault = Vault {
        id: vault_id,
        owner,
        beneficiary,
        balance: 0,
        check_in_interval: template.check_in_interval,
        last_check_in: Utc::now(),
        created_at: Utc::now(),
        status: VaultStatus::Active,
        ttl_remaining: Some(template.check_in_interval),
    };

    store.lock().unwrap().insert(vault.id.clone(), vault.clone());
    Ok(vault)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_vault_templates() {
        let templates = get_vault_templates();
        assert_eq!(templates.templates.len(), 3);
        assert!(templates.templates.iter().any(|t| t.id == "simple-inheritance"));
        assert!(templates.templates.iter().any(|t| t.id == "family-trust"));
        assert!(templates.templates.iter().any(|t| t.id == "business-succession"));
    }

    #[test]
    fn test_simple_inheritance_template() {
        let templates = get_vault_templates();
        let template = templates
            .templates
            .iter()
            .find(|t| t.id == "simple-inheritance")
            .unwrap();
        assert_eq!(template.check_in_interval, 86400 * 30);
        assert!(template.description.contains("single beneficiary"));
    }

    #[test]
    fn test_family_trust_template() {
        let templates = get_vault_templates();
        let template = templates
            .templates
            .iter()
            .find(|t| t.id == "family-trust")
            .unwrap();
        assert_eq!(template.check_in_interval, 86400 * 90);
        assert!(template.description.contains("Multi-beneficiary"));
    }

    #[test]
    fn test_business_succession_template() {
        let templates = get_vault_templates();
        let template = templates
            .templates
            .iter()
            .find(|t| t.id == "business-succession")
            .unwrap();
        assert_eq!(template.check_in_interval, 86400 * 60);
        assert!(template.description.contains("business"));
    }

    #[test]
    fn test_create_vault_from_template() {
        let store = create_vault_store();
        let result = create_vault_from_template(
            &store,
            "simple-inheritance",
            "owner1".to_string(),
            "ben1".to_string(),
        );
        assert!(result.is_ok());
        let vault = result.unwrap();
        assert_eq!(vault.owner, "owner1");
        assert_eq!(vault.beneficiary, "ben1");
        assert_eq!(vault.check_in_interval, 86400 * 30);
    }

    #[test]
    fn test_create_vault_from_family_trust_template() {
        let store = create_vault_store();
        let result = create_vault_from_template(
            &store,
            "family-trust",
            "owner1".to_string(),
            "ben1".to_string(),
        );
        assert!(result.is_ok());
        let vault = result.unwrap();
        assert_eq!(vault.check_in_interval, 86400 * 90);
    }

    #[test]
    fn test_create_vault_from_invalid_template() {
        let store = create_vault_store();
        let result = create_vault_from_template(
            &store,
            "invalid-template",
            "owner1".to_string(),
            "ben1".to_string(),
        );
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }
}
