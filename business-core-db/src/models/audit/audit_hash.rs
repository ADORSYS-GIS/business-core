use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Extension fields for audit tables of auditable entities.
///
/// When an entity implements the auditable trait, its audit table contains all the same fields
/// as the original entity, plus these additional fields:
/// - `hash`: The hash of the audit log itself, with this hash field set to None in the copy to be hashed
/// - `audit_log_id`: The ID of the corresponding audit log entry. This field, together with the
///   original entity's ID, forms the composite primary key for the audit entry
/// - `antecedent_hash`: The hash retrieved from the antecedent audit log, used for verification
/// - `antecedent_audit_log_id`: The ID of the antecedent (previous) audit log entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuditHashModel {
    pub hash: i64,
    pub audit_log_id: Uuid,
    pub antecedent_hash: i64,
    pub antecedent_audit_log_id: Uuid,
}