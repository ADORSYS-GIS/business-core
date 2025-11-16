use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::FromRow;
use super::EntityType;

/// # Documentation
/// - This struct is used to track all entities modified in a single transaction.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLinkModel {
    pub audit_log_id: Uuid,
    pub entity_id: Uuid,
    pub entity_type: EntityType,
}