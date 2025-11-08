use uuid::Uuid;

use super::identifiable::Identifiable;

/// Trait for entities for which audit logs are maintained
pub trait Auditable: Identifiable {
    /// Returns the ID of the audit log entry for this record, if any
    fn get_audit_log_id(&self) -> Option<Uuid>;
}