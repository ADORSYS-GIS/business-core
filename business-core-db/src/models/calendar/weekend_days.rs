use chrono::NaiveDate;
use uuid::Uuid;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::collections::HashMap;
use crate::{HasPrimaryKey, IdxModelCache, Indexable};
use crate::models::{IndexAware, Identifiable, Index};
use postgres_index_cache::HasPrimaryKey as HasPrimaryKeyCache;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct WeekendDaysModel {
    pub id: Uuid,
    pub country_id: Option<Uuid>,
    pub country_subdivision_id: Option<Uuid>,
    pub weekend_day_01: Option<Weekday>,
    pub weekend_day_02: Option<Weekday>,
    pub weekend_day_03: Option<Weekday>,
    pub weekend_day_04: Option<Weekday>,
    pub weekend_day_05: Option<Weekday>,
    pub weekend_day_06: Option<Weekday>,
    pub weekend_day_07: Option<Weekday>,
    pub effective_date: NaiveDate,
    pub expiry_date: Option<NaiveDate>,
}

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WeekendDaysIdxModel {
    pub id: Uuid,
    pub country_id: Option<Uuid>,
    pub country_subdivision_id: Option<Uuid>,
}

impl Identifiable for WeekendDaysModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl IndexAware for WeekendDaysModel {
    type IndexType = WeekendDaysIdxModel;

    fn to_index(&self) -> Self::IndexType {
        WeekendDaysIdxModel {
            id: self.id,
            country_id: self.country_id,
            country_subdivision_id: self.country_subdivision_id,
        }
    }
}

impl Identifiable for WeekendDaysIdxModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Index for WeekendDaysIdxModel {}

impl Indexable for WeekendDaysIdxModel {
    fn i64_keys(&self) -> HashMap<String, Option<i64>> {
        HashMap::new()
    }

    fn uuid_keys(&self) -> HashMap<String, Option<Uuid>> {
        let mut keys = HashMap::new();
        keys.insert("country_id".to_string(), self.country_id);
        keys.insert("country_subdivision_id".to_string(), self.country_subdivision_id);
        keys
    }
}

impl HasPrimaryKey for WeekendDaysIdxModel {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}

impl HasPrimaryKeyCache for WeekendDaysModel {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}

pub type WeekendDaysIdxModelCache = IdxModelCache<WeekendDaysIdxModel>;