#[cfg(test)]
pub mod test_utils {
    use business_core_db::models::reason_and_purpose::compliance_metadata::ComplianceMetadataModel;
    use heapless::String as HeaplessString;
    use uuid::Uuid;

    pub fn create_test_compliance_metadata(
        regulatory_code: Option<&str>,
        reportable: bool,
        requires_sar: bool,
    ) -> ComplianceMetadataModel {
        ComplianceMetadataModel {
            id: Uuid::new_v4(),
            regulatory_code: regulatory_code.map(|code| HeaplessString::try_from(code).unwrap()),
            reportable,
            requires_sar,
            requires_ctr: false,
            retention_years: 7,
            escalation_required: false,
            risk_score_impact: Some(50),
            no_tipping_off: false,
            jurisdictions1: HeaplessString::try_from("US").unwrap(),
            jurisdictions2: HeaplessString::try_from("EU").unwrap(),
            jurisdictions3: HeaplessString::try_from("UK").unwrap(),
            jurisdictions4: HeaplessString::try_from("CA").unwrap(),
            jurisdictions5: HeaplessString::try_from("AU").unwrap(),
        }
    }
}