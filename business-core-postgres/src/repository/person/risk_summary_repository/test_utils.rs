#[cfg(test)]
pub mod test_utils {
    use business_core_db::models::person::risk_summary::RiskSummaryModel;
    use business_core_db::models::person::common_enums::RiskRating;
    use heapless::String as HeaplessString;
    use uuid::Uuid;
    use chrono::Utc;

    pub fn create_test_risk_summary() -> RiskSummaryModel {
        RiskSummaryModel {
            id: Uuid::new_v4(),
            current_rating: RiskRating::Low,
            last_assessment_date: Utc::now(),
            flags_01: HeaplessString::try_from("flag1").unwrap(),
            flags_02: HeaplessString::try_from("flag2").unwrap(),
            flags_03: HeaplessString::try_from("flag3").unwrap(),
            flags_04: HeaplessString::try_from("flag4").unwrap(),
            flags_05: HeaplessString::try_from("flag5").unwrap(),
        }
    }

    pub fn create_test_person() -> business_core_db::models::person::person::PersonModel {
        use business_core_db::models::person::common_enums::{RiskRating, PersonStatus};
        use business_core_db::models::person::person::{PersonModel, PersonType, IdentityType};
        
        PersonModel {
            id: Uuid::new_v4(),
            person_type: PersonType::Natural,
            risk_rating: RiskRating::Low,
            status: PersonStatus::Active,
            display_name: HeaplessString::try_from("Test Person").unwrap(),
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
            last_activity_log: None,
            last_compliance_status: None,
            last_document: None,
            last_portfolio: None,
            antecedent_hash: 0,
            antecedent_audit_log_id: Uuid::nil(),
            hash: 0,
            audit_log_id: None,
        }
    }
}