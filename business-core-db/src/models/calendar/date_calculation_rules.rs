use chrono::NaiveDate;
use uuid::Uuid;
use heapless::String as HeaplessString;
use serde::{Deserialize, Serialize};
use std::str::FromStr;
use std::collections::HashMap;
use crate::{HasPrimaryKey, IdxModelCache, Indexable};
use crate::models::{IndexAware, Identifiable, Index};
use postgres_index_cache::HasPrimaryKey as HasPrimaryKeyCache;

/// Date Calculation Rules Model
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct DateCalculationRulesModel {
    pub id: Uuid,
    pub country_id: Uuid,
    pub country_subdivision_id: Option<Uuid>,
    pub rule_name: HeaplessString<50>,
    pub rule_purpose: DateRulePurpose,
    pub default_shift_rule: DateShiftRule,
    pub weekend_days_id: Option<Uuid>,
    pub priority: i32,
    pub is_active: bool,
    pub effective_date: NaiveDate,
    pub expiry_date: Option<NaiveDate>,
}

/// Date Rule Purpose enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "date_rule_purpose", rename_all = "PascalCase")]
pub enum DateRulePurpose {
    DateShift,
    MaturityCalculation,
    PaymentDue,
}

/// Date Shift Rule enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "date_shift_rule", rename_all = "PascalCase")]
pub enum DateShiftRule {
    NextBusinessDay,
    PreviousBusinessDay,
    NoShift,
}

impl FromStr for DateShiftRule {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NextBusinessDay" => Ok(DateShiftRule::NextBusinessDay),
            "PreviousBusinessDay" => Ok(DateShiftRule::PreviousBusinessDay),
            "NoShift" => Ok(DateShiftRule::NoShift),
            _ => Err(()),
        }
    }
}

impl FromStr for DateRulePurpose {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DateShift" => Ok(DateRulePurpose::DateShift),
            "MaturityCalculation" => Ok(DateRulePurpose::MaturityCalculation),
            "PaymentDue" => Ok(DateRulePurpose::PaymentDue),
            _ => Err(()),
        }
    }
}

/// Date Calculation Rules Index Model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DateCalculationRulesIdxModel {
    pub id: Uuid,
    pub country_id: Option<Uuid>,
    pub country_subdivision_id: Option<Uuid>,
    pub rule_name_hash: i64,
}

impl Identifiable for DateCalculationRulesModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl IndexAware for DateCalculationRulesModel {
    type IndexType = DateCalculationRulesIdxModel;

    fn to_index(&self) -> Self::IndexType {
        // Calculate hash from rule_name
        let rule_name_hash = calculate_string_hash(self.rule_name.as_str());
        
        DateCalculationRulesIdxModel {
            id: self.id,
            country_id: Some(self.country_id),
            country_subdivision_id: self.country_subdivision_id,
            rule_name_hash,
        }
    }
}

impl Identifiable for DateCalculationRulesIdxModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Index for DateCalculationRulesIdxModel {}

impl Indexable for DateCalculationRulesIdxModel {
    fn i64_keys(&self) -> HashMap<String, Option<i64>> {
        let mut keys = HashMap::new();
        keys.insert("rule_name_hash".to_string(), Some(self.rule_name_hash));
        keys
    }

    fn uuid_keys(&self) -> HashMap<String, Option<Uuid>> {
        let mut keys = HashMap::new();
        keys.insert("country_id".to_string(), self.country_id);
        keys.insert("country_subdivision_id".to_string(), self.country_subdivision_id);
        keys
    }
}

impl HasPrimaryKey for DateCalculationRulesIdxModel {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}

impl HasPrimaryKeyCache for DateCalculationRulesModel {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}

pub type DateCalculationRulesIdxModelCache = IdxModelCache<DateCalculationRulesIdxModel>;

/// Calculate a hash from a string for indexing purposes
fn calculate_string_hash(s: &str) -> i64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};
    
    let mut hasher = DefaultHasher::new();
    s.hash(&mut hasher);
    hasher.finish() as i64
}