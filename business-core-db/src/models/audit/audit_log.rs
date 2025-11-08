use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use sqlx::FromRow;
use crate::models::Identifiable;

/// # Documentation
/// - Struct to maintain an audit log
/// - One audit log per database transaction, all objects referenced in the change set reference the same audit log.
/// - Audit log is created by the client.
/// - Command service validates all auditable structs that have changed carry the uuid of the given audit log.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AuditLogModel {
    pub id: Uuid,
    pub updated_at: DateTime<Utc>,
    pub updated_by_person_id: Uuid,
}

impl Identifiable for AuditLogModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}