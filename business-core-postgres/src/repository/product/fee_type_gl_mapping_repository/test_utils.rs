use business_core_db::models::product::fee_type_gl_mapping::{FeeType, FeeTypeGlMappingModel};
use heapless::String as HeaplessString;
use uuid::Uuid;

pub fn create_test_fee_type_gl_mapping(gl_code: &str) -> FeeTypeGlMappingModel {
    FeeTypeGlMappingModel {
        id: Uuid::new_v4(),
        fee_type: FeeType::MaintenanceFee,
        gl_code: HeaplessString::try_from(gl_code).unwrap(),
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}

pub fn create_test_fee_type_gl_mapping_with_fee_type(
    fee_type: FeeType,
    gl_code: &str,
) -> FeeTypeGlMappingModel {
    FeeTypeGlMappingModel {
        id: Uuid::new_v4(),
        fee_type,
        gl_code: HeaplessString::try_from(gl_code).unwrap(),
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}