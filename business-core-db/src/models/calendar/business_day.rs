use chrono::NaiveDate;
use uuid::Uuid;
use heapless::String as HeaplessString;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::collections::HashMap;
use crate::{HasPrimaryKey, IdxModelCache, Indexable};
use crate::models::{IndexAware, Identifiable, Index};
use postgres_index_cache::HasPrimaryKey as HasPrimaryKeyCache;

/// Business Day Model
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
    pub holiday_name: Option<HeaplessString<50>>,
    pub day_scope: DayScope,
}

/// Weekday enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "weekday", rename_all = "PascalCase")]
pub enum Weekday {
    Monday,
    Tuesday,
    Wednesday,
    Thursday,
    Friday,
    Saturday,
    Sunday,
}

impl FromStr for Weekday {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Monday" => Ok(Weekday::Monday),
            "Tuesday" => Ok(Weekday::Tuesday),
            "Wednesday" => Ok(Weekday::Wednesday),
            "Thursday" => Ok(Weekday::Thursday),
            "Friday" => Ok(Weekday::Friday),
            "Saturday" => Ok(Weekday::Saturday),
            "Sunday" => Ok(Weekday::Sunday),
            _ => Err(()),
        }
    }
}

/// Day Scope enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "day_scope", rename_all = "PascalCase")]
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

/// Business Day Index Model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessDayIdxModel {
    pub id: Uuid,
    pub country_id: Option<Uuid>,
    pub country_subdivision_id: Option<Uuid>,
    pub date_hash: i64,
}

impl Identifiable for BusinessDayModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl IndexAware for BusinessDayModel {
    type IndexType = BusinessDayIdxModel;

    fn to_index(&self) -> Self::IndexType {
        // Calculate date_hash from date (using number of days since epoch)
        let date_hash = self.date.and_hms_opt(0, 0, 0)
            .map(|dt| dt.and_utc().timestamp() / 86400)
            .unwrap_or(0);
        
        BusinessDayIdxModel {
            id: self.id,
            country_id: self.country_id,
            country_subdivision_id: self.country_subdivision_id,
            date_hash,
        }
    }
}

impl Identifiable for BusinessDayIdxModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Index for BusinessDayIdxModel {}

impl Indexable for BusinessDayIdxModel {
    fn i64_keys(&self) -> HashMap<String, Option<i64>> {
        let mut keys = HashMap::new();
        keys.insert("date_hash".to_string(), Some(self.date_hash));
        keys
    }

    fn uuid_keys(&self) -> HashMap<String, Option<Uuid>> {
        let mut keys = HashMap::new();
        keys.insert("country_id".to_string(), self.country_id);
        keys.insert("country_subdivision_id".to_string(), self.country_subdivision_id);
        keys
    }
}

impl HasPrimaryKey for BusinessDayIdxModel {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}

impl HasPrimaryKeyCache for BusinessDayModel {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}

pub type BusinessDayIdxModelCache = IdxModelCache<BusinessDayIdxModel>;