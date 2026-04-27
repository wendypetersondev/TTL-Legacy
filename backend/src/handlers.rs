use crate::models::*;
use crate::db::*;
use chrono::Utc;
use serde_json::json;
use std::io::Write;

pub fn search_vaults_handler(
    store: &VaultStore,
    query: SearchQuery,
) -> SearchResult {
    search_vaults(store, &query)
}

pub fn compare_vaults_handler(
    store: &VaultStore,
    vault_ids: Vec<String>,
) -> ComparisonResult {
    let vaults = store.lock().unwrap();
    let comparison_vaults: Vec<Vault> = vault_ids
        .iter()
        .filter_map(|id| vaults.get(id).cloned())
        .collect();

    ComparisonResult {
        vaults: comparison_vaults,
    }
}

pub fn export_vaults_handler(
    store: &VaultStore,
    event_store: &EventStore,
    audit_store: &AuditStore,
    vault_id: &str,
    format: &str,
) -> Result<String, String> {
    let vaults = store.lock().unwrap();
    let vault = vaults
        .get(vault_id)
        .cloned()
        .ok_or_else(|| "Vault not found".to_string())?;

    let history = get_vault_history(event_store, vault_id);
    let audit_log = get_vault_audit_log(audit_store, vault_id);

    let export_data = ExportData {
        vault,
        history,
        audit_log,
    };

    match format {
        "json" => Ok(serde_json::to_string_pretty(&export_data)
            .map_err(|e| e.to_string())?),
        "csv" => export_to_csv(&export_data),
        _ => Err("Unsupported format".to_string()),
    }
}

fn export_to_csv(data: &ExportData) -> Result<String, String> {
    let mut wtr = csv::Writer::from_writer(vec![]);

    // Write vault info
    wtr.write_record(&[
        "Type",
        "ID",
        "Owner",
        "Beneficiary",
        "Balance",
        "Status",
        "Created",
    ])
    .map_err(|e| e.to_string())?;

    wtr.write_record(&[
        "Vault",
        &data.vault.id,
        &data.vault.owner,
        &data.vault.beneficiary,
        &data.vault.balance.to_string(),
        &format!("{:?}", data.vault.status),
        &data.vault.created_at.to_rfc3339(),
    ])
    .map_err(|e| e.to_string())?;

    // Write events
    wtr.write_record(&["", "", "", "", "", "", ""])
        .map_err(|e| e.to_string())?;
    wtr.write_record(&["Event", "Type", "Timestamp", "Data", "", "", ""])
        .map_err(|e| e.to_string())?;

    for event in &data.history {
        wtr.write_record(&[
            "Event",
            &format!("{:?}", event.event_type),
            &event.timestamp.to_rfc3339(),
            &event.data.to_string(),
            "",
            "",
            "",
        ])
        .map_err(|e| e.to_string())?;
    }

    let buffer = wtr.into_inner().map_err(|e| e.to_string())?;
    String::from_utf8(buffer).map_err(|e| e.to_string())
}

pub fn generate_compliance_report(
    store: &VaultStore,
    event_store: &EventStore,
    vault_id: &str,
) -> Result<ComplianceReport, String> {
    let vaults = store.lock().unwrap();
    let vault = vaults
        .get(vault_id)
        .cloned()
        .ok_or_else(|| "Vault not found".to_string())?;

    let history = get_vault_history(event_store, vault_id);
    
    let mut fund_movements = Vec::new();
    let mut beneficiary_changes = Vec::new();
    let mut ttl_history = Vec::new();
    let mut total_deposits = 0i128;
    let mut total_withdrawals = 0i128;

    for event in history {
        match event.event_type {
            EventType::Deposit => {
                if let Some(amount) = event.data.get("amount").and_then(|v| v.as_i64()) {
                    total_deposits += amount as i128;
                    fund_movements.push(FundMovement {
                        timestamp: event.timestamp,
                        movement_type: "deposit".to_string(),
                        amount: amount as i128,
                        balance_after: vault.balance,
                    });
                }
            }
            EventType::Withdrawal => {
                if let Some(amount) = event.data.get("amount").and_then(|v| v.as_i64()) {
                    total_withdrawals += amount as i128;
                    fund_movements.push(FundMovement {
                        timestamp: event.timestamp,
                        movement_type: "withdrawal".to_string(),
                        amount: amount as i128,
                        balance_after: vault.balance,
                    });
                }
            }
            EventType::TtlUpdate => {
                if let Some(ttl) = event.data.get("ttl_remaining").and_then(|v| v.as_u64()) {
                    ttl_history.push(TtlEvent {
                        timestamp: event.timestamp,
                        event_type: "ttl_extended".to_string(),
                        ttl_remaining: Some(ttl),
                    });
                }
            }
            EventType::StatusChange => {
                if let Some(old_ben) = event.data.get("old_beneficiary").and_then(|v| v.as_str()) {
                    if let Some(new_ben) = event.data.get("new_beneficiary").and_then(|v| v.as_str()) {
                        beneficiary_changes.push(BeneficiaryChange {
                            timestamp: event.timestamp,
                            old_beneficiary: old_ben.to_string(),
                            new_beneficiary: new_ben.to_string(),
                        });
                    }
                }
            }
            _ => {}
        }
    }

    Ok(ComplianceReport {
        vault_id: vault.id,
        owner: vault.owner,
        beneficiary: vault.beneficiary,
        report_generated_at: Utc::now(),
        fund_movements,
        beneficiary_changes,
        ttl_history,
        total_deposits,
        total_withdrawals,
        current_balance: vault.balance,
    })
}

pub fn export_compliance_report(
    report: &ComplianceReport,
    format: &str,
) -> Result<String, String> {
    match format {
        "json" => Ok(serde_json::to_string_pretty(report).map_err(|e| e.to_string())?),
        "pdf" => {
            // Minimal PDF export as text representation
            let mut pdf_content = String::new();
            pdf_content.push_str(&format!("COMPLIANCE REPORT\n"));
            pdf_content.push_str(&format!("Generated: {}\n\n", report.report_generated_at));
            pdf_content.push_str(&format!("Vault ID: {}\n", report.vault_id));
            pdf_content.push_str(&format!("Owner: {}\n", report.owner));
            pdf_content.push_str(&format!("Beneficiary: {}\n", report.beneficiary));
            pdf_content.push_str(&format!("Current Balance: {}\n", report.current_balance));
            pdf_content.push_str(&format!("Total Deposits: {}\n", report.total_deposits));
            pdf_content.push_str(&format!("Total Withdrawals: {}\n\n", report.total_withdrawals));
            
            pdf_content.push_str("FUND MOVEMENTS:\n");
            for movement in &report.fund_movements {
                pdf_content.push_str(&format!(
                    "{} - {} {}\n",
                    movement.timestamp, movement.movement_type, movement.amount
                ));
            }
            
            pdf_content.push_str("\nBENEFICIARY CHANGES:\n");
            for change in &report.beneficiary_changes {
                pdf_content.push_str(&format!(
                    "{} - {} -> {}\n",
                    change.timestamp, change.old_beneficiary, change.new_beneficiary
                ));
            }
            
            Ok(pdf_content)
        }
        _ => Err("Unsupported format".to_string()),
    }
}

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
    fn test_search_vaults_handler() {
        let store = create_vault_store();
        let vault = Vault {
            id: "v1".to_string(),
            owner: "owner1".to_string(),
            beneficiary: "ben1".to_string(),
            balance: 1000,
            check_in_interval: 86400,
            last_check_in: Utc::now(),
            created_at: Utc::now(),
            status: VaultStatus::Active,
            ttl_remaining: Some(100000),
        };
        store.lock().unwrap().insert("v1".to_string(), vault);

        let query = SearchQuery {
            owner: Some("owner1".to_string()),
            beneficiary: None,
            status: None,
            created_after: None,
            created_before: None,
            page: None,
            limit: None,
        };

        let result = search_vaults_handler(&store, query);
        assert_eq!(result.vaults.len(), 1);
    }

    #[test]
    fn test_compare_vaults_handler() {
        let store = create_vault_store();
        let vault1 = Vault {
            id: "v1".to_string(),
            owner: "owner1".to_string(),
            beneficiary: "ben1".to_string(),
            balance: 1000,
            check_in_interval: 86400,
            last_check_in: Utc::now(),
            created_at: Utc::now(),
            status: VaultStatus::Active,
            ttl_remaining: Some(100000),
        };
        let vault2 = Vault {
            id: "v2".to_string(),
            owner: "owner2".to_string(),
            beneficiary: "ben2".to_string(),
            balance: 2000,
            check_in_interval: 172800,
            last_check_in: Utc::now(),
            created_at: Utc::now(),
            status: VaultStatus::Active,
            ttl_remaining: Some(200000),
        };
        store.lock().unwrap().insert("v1".to_string(), vault1);
        store.lock().unwrap().insert("v2".to_string(), vault2);

        let result = compare_vaults_handler(&store, vec!["v1".to_string(), "v2".to_string()]);
        assert_eq!(result.vaults.len(), 2);
    }

    #[test]
    fn test_export_vaults_handler_json() {
        let store = create_vault_store();
        let event_store = create_event_store();
        let audit_store = create_audit_store();

        let vault = Vault {
            id: "v1".to_string(),
            owner: "owner1".to_string(),
            beneficiary: "ben1".to_string(),
            balance: 1000,
            check_in_interval: 86400,
            last_check_in: Utc::now(),
            created_at: Utc::now(),
            status: VaultStatus::Active,
            ttl_remaining: Some(100000),
        };
        store.lock().unwrap().insert("v1".to_string(), vault);

        let result = export_vaults_handler(&store, &event_store, &audit_store, "v1", "json");
        assert!(result.is_ok());
        let json_str = result.unwrap();
        assert!(json_str.contains("v1"));
    }

    #[test]
    fn test_export_vaults_handler_csv() {
        let store = create_vault_store();
        let event_store = create_event_store();
        let audit_store = create_audit_store();

        let vault = Vault {
            id: "v1".to_string(),
            owner: "owner1".to_string(),
            beneficiary: "ben1".to_string(),
            balance: 1000,
            check_in_interval: 86400,
            last_check_in: Utc::now(),
            created_at: Utc::now(),
            status: VaultStatus::Active,
            ttl_remaining: Some(100000),
        };
        store.lock().unwrap().insert("v1".to_string(), vault);

        let result = export_vaults_handler(&store, &event_store, &audit_store, "v1", "csv");
        assert!(result.is_ok());
        let csv_str = result.unwrap();
        assert!(csv_str.contains("v1"));
    }

    #[test]
    fn test_generate_compliance_report() {
        let store = create_vault_store();
        let event_store = create_event_store();

        let vault = Vault {
            id: "v1".to_string(),
            owner: "owner1".to_string(),
            beneficiary: "ben1".to_string(),
            balance: 1000,
            check_in_interval: 86400,
            last_check_in: Utc::now(),
            created_at: Utc::now(),
            status: VaultStatus::Active,
            ttl_remaining: Some(100000),
        };
        store.lock().unwrap().insert("v1".to_string(), vault);

        let result = generate_compliance_report(&store, &event_store, "v1");
        assert!(result.is_ok());
        let report = result.unwrap();
        assert_eq!(report.vault_id, "v1");
        assert_eq!(report.owner, "owner1");
        assert_eq!(report.current_balance, 1000);
    }

    #[test]
    fn test_export_compliance_report_json() {
        let report = ComplianceReport {
            vault_id: "v1".to_string(),
            owner: "owner1".to_string(),
            beneficiary: "ben1".to_string(),
            report_generated_at: Utc::now(),
            fund_movements: vec![],
            beneficiary_changes: vec![],
            ttl_history: vec![],
            total_deposits: 1000,
            total_withdrawals: 0,
            current_balance: 1000,
        };

        let result = export_compliance_report(&report, "json");
        assert!(result.is_ok());
        let json_str = result.unwrap();
        assert!(json_str.contains("v1"));
    }

    #[test]
    fn test_export_compliance_report_pdf() {
        let report = ComplianceReport {
            vault_id: "v1".to_string(),
            owner: "owner1".to_string(),
            beneficiary: "ben1".to_string(),
            report_generated_at: Utc::now(),
            fund_movements: vec![],
            beneficiary_changes: vec![],
            ttl_history: vec![],
            total_deposits: 1000,
            total_withdrawals: 0,
            current_balance: 1000,
        };

        let result = export_compliance_report(&report, "pdf");
        assert!(result.is_ok());
        let pdf_str = result.unwrap();
        assert!(pdf_str.contains("COMPLIANCE REPORT"));
        assert!(pdf_str.contains("v1"));
    }
}
