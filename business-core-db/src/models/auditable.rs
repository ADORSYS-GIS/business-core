use uuid::Uuid;

use super::identifiable::Identifiable;

/// Trait for entities for which audit logs are maintained
pub trait Auditable: Identifiable {
    /// Returns the ID of the last audit log entry for this entity, if any
    fn get_last_audit_log_id(&self) -> Option<Uuid>;
}