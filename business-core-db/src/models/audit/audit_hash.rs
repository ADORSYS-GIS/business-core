use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Extension fields for audit tables of auditable entities.
///
/// When an entity implements the auditable trait, its audit table contains all the same fields
/// as the original entity, plus these two additional fields:
/// - `hash`: The hash of the original entity at the time of the modification
/// - `audit_log_id`: The ID of the corresponding audit log entry. This field, together with the
///   original entity's ID, forms the composite primary key for the audit entry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditHashModel {
    pub hash: i64,
    pub audit_log_id: Uuid,
}