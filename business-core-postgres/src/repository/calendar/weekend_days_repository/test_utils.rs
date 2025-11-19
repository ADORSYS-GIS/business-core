#[cfg(test)]
pub mod test_utils {
    use business_core_db::models::calendar::weekend_days::{WeekendDaysModel, Weekday};
    use chrono::NaiveDate;
    use uuid::Uuid;

    pub fn create_test_weekend_days(
        country_id: Option<Uuid>,
        country_subdivision_id: Option<Uuid>,
    ) -> WeekendDaysModel {
        WeekendDaysModel {
            id: Uuid::new_v4(),
            country_id,
            country_subdivision_id,
            weekend_day_01: Some(Weekday::Saturday),
            weekend_day_02: Some(Weekday::Sunday),
            weekend_day_03: None,
            weekend_day_04: None,
            weekend_day_05: None,
            weekend_day_06: None,
            weekend_day_07: None,
            effective_date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            expiry_date: None,
        }
    }
}