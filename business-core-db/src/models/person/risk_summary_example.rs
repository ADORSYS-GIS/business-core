use chrono::{DateTime, Utc};
use heapless::String as HeaplessString;
use serde::{Deserialize, Serialize};
use super::common_enums::RiskRating;

/// Database model for Customer risk summary
/// # Index
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct RiskSummaryModel {
    #[serde(serialize_with = "super::common_enums::serialize_risk_rating", deserialize_with = "super::common_enums::deserialize_risk_rating")]
    pub current_rating: RiskRating,
    pub last_assessment_date: DateTime<Utc>,
    pub flags_01: HeaplessString<200>,
    pub flags_02: HeaplessString<200>,
    pub flags_03: HeaplessString<200>,
    pub flags_04: HeaplessString<200>,
    pub flags_05: HeaplessString<200>,
}
