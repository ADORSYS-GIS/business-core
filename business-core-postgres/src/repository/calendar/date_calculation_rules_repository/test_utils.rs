
#[cfg(test)]
pub mod test_utils {
    use business_core_db::models::calendar::date_calculation_rules::{DateCalculationRulesModel, DateRulePurpose, DateShiftRule};
    use chrono::NaiveDate;
    use heapless::String as HeaplessString;
    use uuid::Uuid;

    pub fn create_test_date_calculation_rule(
        country_id: Uuid,
        country_subdivision_id: Option<Uuid>,
        rule_name: &str,
    ) -> DateCalculationRulesModel {
        DateCalculationRulesModel {
            id: Uuid::new_v4(),
            country_id,
            country_subdivision_id,
            rule_name: HeaplessString::try_from(rule_name).unwrap(),
            rule_purpose: DateRulePurpose::DateShift,
            default_shift_rule: DateShiftRule::NextBusinessDay,
            weekend_days_id: None,
            priority: 1,
            is_active: true,
            effective_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            expiry_date: None,
        }
    }
}