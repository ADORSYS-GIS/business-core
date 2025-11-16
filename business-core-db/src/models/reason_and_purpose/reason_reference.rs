use heapless::String as HeaplessString;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use crate::models::auditable::Auditable;
use crate::models::identifiable::Identifiable;
use crate::models::audit::entity_type::EntityType;

/// # Documentation
/// Reason Reference links a reason to an entity in the system.
/// There is no custom finder on this table. Navigation always originates from the 
/// entity referenced here.
/// 
/// This entity is auditable but not indexable - accessed by ID only.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReasonReferenceModel {
    pub id: Uuid,

    /// Reference to the reason being applied
    pub reason_id: Uuid,

    /// The entity to which the reason applies
    pub entity_id: Uuid,

    /// Additional contextual details about this reason application
    pub additional_details: Option<HeaplessString<200>>,

    /// The type of entity being referenced
    #[serde(serialize_with = "serialize_entity_type", deserialize_with = "deserialize_entity_type")]
    pub entity_type: EntityType,

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

impl Identifiable for ReasonReferenceModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Auditable for ReasonReferenceModel {
    fn get_audit_log_id(&self) -> Option<Uuid> {
        self.audit_log_id
    }
}

fn serialize_entity_type<S>(entity_type: &EntityType, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(match entity_type {
        EntityType::Location => "Location",
        EntityType::Person => "Person",
        EntityType::EntityReference => "EntityReference",
        EntityType::ReasonReference => "ReasonReference",
    })
}

fn deserialize_entity_type<'de, D>(deserializer: D) -> Result<EntityType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "Location" => Ok(EntityType::Location),
        "Person" => Ok(EntityType::Person),
        "EntityReference" => Ok(EntityType::EntityReference),
        "ReasonReference" => Ok(EntityType::ReasonReference),
        _ => Err(serde::de::Error::custom(format!("Unknown entity type: {s}"))),
    }
}