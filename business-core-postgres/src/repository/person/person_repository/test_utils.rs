use business_core_db::models::person::person::{PersonModel, PersonType, IdentityType};
use business_core_db::models::person::common_enums::{RiskRating, PersonStatus};
use heapless::String as HeaplessString;
use uuid::Uuid;

pub fn create_test_person(
    display_name: &str,
    person_type: PersonType,
) -> PersonModel {
    PersonModel {
        id: Uuid::new_v4(),
        person_type,
        risk_rating: RiskRating::Low,
        status: PersonStatus::Active,
        display_name: HeaplessString::try_from(display_name).unwrap(),
        external_identifier: None,
        id_type: IdentityType::NationalId,
        id_number: HeaplessString::try_from("TEST123456789").unwrap(),
        entity_reference_count: 0,
        organization_person_id: None,
        messaging_info1: None,
        messaging_info2: None,
        messaging_info3: None,
        messaging_info4: None,
        messaging_info5: None,
        department: None,
        location_id: None,
        duplicate_of_person_id: None,
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}

pub fn create_test_person_with_external_id(
    display_name: &str,
    person_type: PersonType,
    external_id: &str,
) -> PersonModel {
    PersonModel {
        id: Uuid::new_v4(),
        person_type,
        risk_rating: RiskRating::Low,
        status: PersonStatus::Active,
        display_name: HeaplessString::try_from(display_name).unwrap(),
        external_identifier: Some(HeaplessString::try_from(external_id).unwrap()),
        id_type: IdentityType::NationalId,
        id_number: HeaplessString::try_from("TEST123456789").unwrap(),
        entity_reference_count: 0,
        organization_person_id: None,
        messaging_info1: None,
        messaging_info2: None,
        messaging_info3: None,
        messaging_info4: None,
        messaging_info5: None,
        department: None,
        location_id: None,
        duplicate_of_person_id: None,
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}