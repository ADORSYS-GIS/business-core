use business_core_db::models::reason_and_purpose::reason_reference::ReasonReferenceModel;
use business_core_db::models::audit::entity_type::EntityType;
use heapless::String as HeaplessString;
use uuid::Uuid;

pub fn create_test_reason_reference(reason_id: Uuid, entity_id: Uuid) -> ReasonReferenceModel {
    ReasonReferenceModel {
        id: Uuid::new_v4(),
        reason_id,
        entity_id,
        additional_details: None,
        entity_type: EntityType::Person,
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}

pub fn create_test_reason_reference_with_details(
    reason_id: Uuid,
    entity_id: Uuid,
    details: &str,
) -> ReasonReferenceModel {
    ReasonReferenceModel {
        id: Uuid::new_v4(),
        reason_id,
        entity_id,
        additional_details: Some(HeaplessString::try_from(details).unwrap()),
        entity_type: EntityType::Person,
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}

pub fn create_test_reason_reference_with_entity_type(
    reason_id: Uuid,
    entity_id: Uuid,
    entity_type: EntityType,
) -> ReasonReferenceModel {
    ReasonReferenceModel {
        id: Uuid::new_v4(),
        reason_id,
        entity_id,
        additional_details: None,
        entity_type,
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}