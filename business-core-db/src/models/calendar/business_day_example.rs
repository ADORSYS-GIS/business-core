use chrono::{DateTime, Utc, NaiveDate};
use uuid::Uuid;
use heapless::String as HeaplessString;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

/// Business Day
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct BusinessDayModel {
    pub id: Uuid,
    pub country_id: Option<Uuid>,
    pub country_subdivision_id: Option<Uuid>,

    pub date: NaiveDate,

    pub weekday: Weekday,

    pub is_business_day: bool,

    pub is_weekend: bool,
    pub weekend_day_01: Option<Uuid>,

    pub is_holiday: bool,
    #[serde(serialize_with = "serialize_holiday_type", deserialize_with = "deserialize_holiday_type")]
    pub holiday_type: Option<HolidayType>, // Use enum instead of HeaplessString
    pub holiday_name: Option<HeaplessString<50>>,

    pub day_scope: DayScope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "holiday_type", rename_all = "PascalCase")]
pub enum DayScope {
    National,
    Regional,
    Religious,
    Banking,
}

impl FromStr for DayScope {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "National" => Ok(DayScope::National),
            "Regional" => Ok(DayScope::Regional),
            "Religious" => Ok(DayScope::Religious),
            "Banking" => Ok(DayScope::Banking),
            _ => Err(()),
        }
    }
}

// Serialization functions for DayScope
fn serialize_holiday_type<S>(holiday_type: &DayScope, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let type_str = match holiday_type {
        DayScope::National => "National",
        DayScope::Regional => "Regional",
        DayScope::Religious => "Religious",
        DayScope::Banking => "Banking",
    };
    serializer.serialize_str(type_str)
}

fn deserialize_holiday_type<'de, D>(deserializer: D) -> Result<DayScope, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "National" => Ok(DayScope::National),
        "Regional" => Ok(DayScope::Regional),
        "Religious" => Ok(DayScope::Religious),
        "Banking" => Ok(DayScope::Banking),
        _ => Err(serde::de::Error::custom(format!("Unknown holiday type: {s}"))),
    }
}
