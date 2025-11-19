use chrono::{DateTime, Utc};
use heapless::String as HeaplessString;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Database model for Customer audit trail
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct CustomerAuditModel {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub field_name: HeaplessString<50>,
    pub old_value: Option<HeaplessString<255>>,
    pub new_value: Option<HeaplessString<255>>,
    pub changed_at: DateTime<Utc>,
    /// References Person.person_id
    pub changed_by: Uuid,
    pub reason: Option<HeaplessString<255>>,
}