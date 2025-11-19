use business_core_db::models::audit::AuditLogModel;
use business_core_db::models::person::country::CountryModel;
use business_core_db::models::person::country_subdivision::CountrySubdivisionModel;
use business_core_db::models::person::entity_reference::{EntityReferenceModel, RelationshipRole};
use business_core_db::models::person::locality::LocalityModel;
use business_core_db::models::person::location::{LocationModel, LocationType};
use business_core_db::models::person::person::{IdentityType, PersonModel, PersonType};
use business_core_db::models::person::common_enums::{RiskRating, PersonStatus};
use chrono::Utc;
use heapless::String as HeaplessString;
use uuid::Uuid;

pub fn create_test_audit_log() -> AuditLogModel {
    AuditLogModel {
        id: Uuid::new_v4(),
        updated_at: Utc::now(),
        updated_by_person_id: Uuid::new_v4(),
    }
}

pub fn create_test_country(iso2: &str, name: &str) -> CountryModel {
    CountryModel {
        id: Uuid::new_v4(),
        iso2: HeaplessString::try_from(iso2).unwrap(),
        name_l1: HeaplessString::try_from(name).unwrap(),
        name_l2: None,
        name_l3: None,
    }
}

pub fn create_test_country_subdivision(
    country_id: Uuid,
    code: &str,
    name: &str,
) -> CountrySubdivisionModel {
    CountrySubdivisionModel {
        id: Uuid::new_v4(),
        country_id,
        code: HeaplessString::try_from(code).unwrap(),
        name_l1: HeaplessString::try_from(name).unwrap(),
        name_l2: None,
        name_l3: None,
    }
}

pub fn create_test_locality(
    country_subdivision_id: Uuid,
    code: &str,
    name: &str,
) -> LocalityModel {
    LocalityModel {
        id: Uuid::new_v4(),
        country_subdivision_id,
        code: HeaplessString::try_from(code).unwrap(),
        name_l1: HeaplessString::try_from(name).unwrap(),
        name_l2: None,
        name_l3: None,
    }
}

pub fn create_test_location(locality_id: Uuid, street_line1: &str) -> LocationModel {
    LocationModel {
        id: Uuid::new_v4(),
        street_line1: HeaplessString::try_from(street_line1).unwrap(),
        street_line2: None,
        street_line3: None,
        street_line4: None,
        locality_id,
        postal_code: None,
        latitude: None,
        longitude: None,
        accuracy_meters: None,
        location_type: LocationType::Residential,
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}

pub fn create_test_person(display_name: &str) -> PersonModel {
    PersonModel {
        id: Uuid::new_v4(),
        person_type: PersonType::Natural,
        risk_rating: RiskRating::Low,
        status: PersonStatus::Active,
        display_name: HeaplessString::try_from(display_name).unwrap(),
        external_identifier: None,
        id_type: IdentityType::NationalId,
        id_number: HeaplessString::try_from("TEST123456").unwrap(),
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