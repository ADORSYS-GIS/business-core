#![allow(dead_code)]

use business_core_db::models::product::account_gl_mapping::AccountGlMappingModel;
use uuid::Uuid;

pub fn create_test_account_gl_mapping(
    customer_account_code: &str,
    overdraft_code: Option<&str>,
) -> AccountGlMappingModel {
    AccountGlMappingModel {
        id: Uuid::new_v4(),
        customer_account_code: customer_account_code.try_into().unwrap(),
        overdraft_code: overdraft_code.map(|s| s.try_into().unwrap()),
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}