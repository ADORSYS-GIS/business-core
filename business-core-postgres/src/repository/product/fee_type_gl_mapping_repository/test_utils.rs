#![cfg(test)]

use business_core_db::models::product::fee_type_gl_mapping::{FeeType, FeeTypeGlMappingModel};
use uuid::Uuid;

pub fn create_test_fee_type_gl_mapping(
    fee_type: FeeType,
    gl_code: &str,
) -> FeeTypeGlMappingModel {
    FeeTypeGlMappingModel {
        id: Uuid::new_v4(),
        fee_type,
        gl_code: gl_code.try_into().unwrap(),
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}