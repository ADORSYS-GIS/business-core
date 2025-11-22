use business_core_db::models::product::account_gl_mapping::AccountGlMappingModel;
use heapless::String as HeaplessString;
use uuid::Uuid;

pub fn create_test_account_gl_mapping(customer_account_code: &str) -> AccountGlMappingModel {
    AccountGlMappingModel {
        id: Uuid::new_v4(),
        customer_account_code: HeaplessString::try_from(customer_account_code).unwrap(),
        overdraft_code: None,
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}

pub fn create_test_account_gl_mapping_with_overdraft(
    customer_account_code: &str,
    overdraft_code: &str,
) -> AccountGlMappingModel {
    AccountGlMappingModel {
        id: Uuid::new_v4(),
        customer_account_code: HeaplessString::try_from(customer_account_code).unwrap(),
        overdraft_code: Some(HeaplessString::try_from(overdraft_code).unwrap()),
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}