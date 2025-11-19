use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Database model for Customer Portfolio summary
/// # Audit
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct ActivityLogModel {
    pub id: Uuid,
    pub person_id: Uuid,
    pub activity_summary: Option<HeaplessString<250>>,
}
