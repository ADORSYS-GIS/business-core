#[cfg(test)]
pub mod test_utils {
    use business_core_db::models::calendar::business_day::{BusinessDayModel, Weekday, DayScope};
    use chrono::NaiveDate;
    use uuid::Uuid;
    use heapless::String as HeaplessString;

    pub fn create_test_business_day(
        country_id: Option<Uuid>,
        country_subdivision_id: Option<Uuid>,
    ) -> BusinessDayModel {
        BusinessDayModel {
            id: Uuid::new_v4(),
            country_id,
            country_subdivision_id,
            date: NaiveDate::from_ymd_opt(2024, 1, 15).unwrap(),
            weekday: Weekday::Monday,
            is_business_day: true,
            is_weekend: false,
            weekend_day_01: None,
            is_holiday: false,
            holiday_name: None,
            day_scope: DayScope::National,
        }
    }

    pub fn create_test_business_day_with_date(
        date: NaiveDate,
    ) -> BusinessDayModel {
        BusinessDayModel {
            id: Uuid::new_v4(),
            country_id: None,
            country_subdivision_id: None,
            date,
            weekday: Weekday::Monday,
            is_business_day: true,
            is_weekend: false,
            weekend_day_01: None,
            is_holiday: false,
            holiday_name: None,
            day_scope: DayScope::National,
        }
    }

    pub fn create_test_business_day_holiday(
        country_id: Option<Uuid>,
        holiday_name: &str,
    ) -> BusinessDayModel {
        let mut name = HeaplessString::new();
        let _ = name.push_str(holiday_name);
        
        BusinessDayModel {
            id: Uuid::new_v4(),
            country_id,
            country_subdivision_id: None,
            date: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
            weekday: Weekday::Monday,
            is_business_day: false,
            is_weekend: false,
            weekend_day_01: None,
            is_holiday: true,
            holiday_name: Some(name),
            day_scope: DayScope::National,
        }
    }
}