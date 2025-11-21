use business_core_db::models::person::compliance_status::ComplianceStatusModel;
use business_core_db::models::person::compliance_status::KycStatus;
use uuid::Uuid;

pub fn create_test_compliance_status(person_id: Uuid) -> ComplianceStatusModel {
    ComplianceStatusModel {
        id: Uuid::new_v4(),
        person_id,
        kyc_status: KycStatus::NotStarted,
        sanctions_checked: false,
        last_screening_date: None,
        predecessor_1: None,
        predecessor_2: None,
        predecessor_3: None,
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}

pub fn create_test_compliance_status_with_status(
    person_id: Uuid,
    kyc_status: KycStatus,
    sanctions_checked: bool,
) -> ComplianceStatusModel {
    ComplianceStatusModel {
        id: Uuid::new_v4(),
        person_id,
        kyc_status,
        sanctions_checked,
        last_screening_date: Some(chrono::Utc::now()),
        predecessor_1: None,
        predecessor_2: None,
        predecessor_3: None,
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}