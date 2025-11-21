use chrono::{DateTime, Utc};
use heapless::String as HeaplessString;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use std::collections::HashMap;
use crate::{HasPrimaryKey, IdxModelCache, Indexable};
use crate::models::{IndexAware, Identifiable, Index};
use super::common_enums::RiskRating;

/// Database model for Customer risk summary
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RiskSummaryModel {
    pub id: Uuid,
    #[serde(serialize_with = "super::common_enums::serialize_risk_rating", deserialize_with = "super::common_enums::deserialize_risk_rating")]
    pub current_rating: RiskRating,
    pub last_assessment_date: DateTime<Utc>,
    pub flags_01: HeaplessString<200>,
    pub flags_02: HeaplessString<200>,
    pub flags_03: HeaplessString<200>,
    pub flags_04: HeaplessString<200>,
    pub flags_05: HeaplessString<200>,
}

/// Index model for RiskSummary
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct RiskSummaryIdxModel {
    pub id: Uuid,
}

// Trait implementations
impl HasPrimaryKey for RiskSummaryIdxModel {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}

impl HasPrimaryKey for RiskSummaryModel {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}

impl Identifiable for RiskSummaryModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl IndexAware for RiskSummaryModel {
    type IndexType = RiskSummaryIdxModel;
    
    fn to_index(&self) -> Self::IndexType {
        RiskSummaryIdxModel {
            id: self.id,
        }
    }
}

impl Identifiable for RiskSummaryIdxModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Index for RiskSummaryIdxModel {}

impl Indexable for RiskSummaryIdxModel {
    fn i64_keys(&self) -> HashMap<String, Option<i64>> {
        HashMap::new()
    }

    fn uuid_keys(&self) -> HashMap<String, Option<Uuid>> {
        HashMap::new()
    }
}

pub type RiskSummaryIdxModelCache = IdxModelCache<RiskSummaryIdxModel>;