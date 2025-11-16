use business_core_db::models::person::entity_reference::{EntityReferenceModel, RelationshipRole};
use heapless::String as HeaplessString;
use uuid::Uuid;

pub fn create_test_entity_reference(person_id: Uuid, reference_external_id: &str) -> EntityReferenceModel {
    EntityReferenceModel {
        id: Uuid::new_v4(),
        person_id,
        entity_role: RelationshipRole::Customer,
        reference_external_id: HeaplessString::try_from(reference_external_id).unwrap(),
        reference_details_l1: None,
        reference_details_l2: None,
        reference_details_l3: None,
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}