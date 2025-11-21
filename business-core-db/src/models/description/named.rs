use heapless::String as HeaplessString;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use crate::models::auditable::Auditable;
use crate::models::identifiable::Identifiable;
use super::named_entity_type::NamedEntityType;

/// # Documentation
/// Named entity provides multilingual support for names and descriptions.
/// 
/// This entity is auditable but not indexable - accessed by ID only.
/// Supports up to 4 language variants (l1, l2, l3, l4) for both names and descriptions.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NamedModel {
    pub id: Uuid,

    /// Entity type indicating which table this Named entity is connected to
    pub entity_type: NamedEntityType,

    /// Primary name (language 1) - required
    pub name_l1: HeaplessString<50>,

    /// Secondary name (language 2) - optional
    pub name_l2: Option<HeaplessString<50>>,

    /// Tertiary name (language 3) - optional
    pub name_l3: Option<HeaplessString<50>>,

    /// Quaternary name (language 4) - optional
    pub name_l4: Option<HeaplessString<50>>,

    /// Primary description (language 1) - optional
    pub description_l1: Option<HeaplessString<255>>,

    /// Secondary description (language 2) - optional
    pub description_l2: Option<HeaplessString<255>>,

    /// Tertiary description (language 3) - optional
    pub description_l3: Option<HeaplessString<255>>,

    /// Quaternary description (language 4) - optional
    pub description_l4: Option<HeaplessString<255>>,

    /// Hash from the previous audit record for chain verification (0 for initial create)
    pub antecedent_hash: i64,
    
    /// Reference to the previous audit log entry (Uuid::nil() for initial create)
    pub antecedent_audit_log_id: Uuid,
    
    /// Hash of the entity with hash field set to 0
    /// - 0: for new entities not yet created or not yet hashed
    /// - Non-zero: computed hash providing tamper detection
    pub hash: i64,
    
    /// Reference to the current audit log entry for this entity
    /// - None: for new entities not yet created
    /// - Some(uuid): updated on every create/update operation to reference the latest audit log
    /// 
    /// This field, together with `id`, forms the composite primary key in the audit table
    pub audit_log_id: Option<Uuid>,
}

impl Identifiable for NamedModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Auditable for NamedModel {
    fn get_audit_log_id(&self) -> Option<Uuid> {
        self.audit_log_id
    }
}