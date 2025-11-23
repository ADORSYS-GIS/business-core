use business_core_db::models::description::{named::NamedModel, named_entity_type::NamedEntityType};
use heapless::String as HeaplessString;
use uuid::Uuid;

pub fn create_test_named(name: &str) -> NamedModel {
    NamedModel {
        id: Uuid::new_v4(),
        entity_type: NamedEntityType::Person,
        name_l1: HeaplessString::try_from(name).unwrap(),
        name_l2: None,
        name_l3: None,
        name_l4: None,
        description_l1: None,
        description_l2: None,
        description_l3: None,
        description_l4: None,
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}