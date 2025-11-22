use business_core_db::models::product::account_gl_mapping::AccountGlMappingModel;
use uuid::Uuid;
use heapless::String;

pub fn create_test_account_gl_mapping(customer_account_code: &str) -> AccountGlMappingModel {
    AccountGlMappingModel {
        id: Uuid::new_v4(),
        customer_account_code: String::from(customer_account_code),
        overdraft_code: None,
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}