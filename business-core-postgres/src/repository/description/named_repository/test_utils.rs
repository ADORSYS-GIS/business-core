use business_core_db::models::description::named::NamedModel;
use business_core_db::models::description::named_entity_type::NamedEntityType;
use heapless::String as HeaplessString;
use uuid::Uuid;

/// Create a test Named entity with default values
pub fn create_test_named() -> NamedModel {
    NamedModel {
        id: Uuid::new_v4(),
        entity_type: NamedEntityType::Person,
        name_l1: HeaplessString::try_from("Test Name").unwrap(),
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

/// Create a test Named entity with all language variants
pub fn create_test_named_with_all_languages() -> NamedModel {
    NamedModel {
        id: Uuid::new_v4(),
        entity_type: NamedEntityType::Person,
        name_l1: HeaplessString::try_from("English Name").unwrap(),
        name_l2: Some(HeaplessString::try_from("French Name").unwrap()),
        name_l3: Some(HeaplessString::try_from("German Name").unwrap()),
        name_l4: Some(HeaplessString::try_from("Spanish Name").unwrap()),
        description_l1: Some(HeaplessString::try_from("English Description").unwrap()),
        description_l2: Some(HeaplessString::try_from("French Description").unwrap()),
        description_l3: Some(HeaplessString::try_from("German Description").unwrap()),
        description_l4: Some(HeaplessString::try_from("Spanish Description").unwrap()),
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}