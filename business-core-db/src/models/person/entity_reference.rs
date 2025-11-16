use heapless::String as HeaplessString;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use std::collections::HashMap;
use crate::{HasPrimaryKey, IdxModelCache, Indexable};
use crate::models::auditable::Auditable;
use crate::models::identifiable::Identifiable;
use crate::models::{Index, IndexAware};

/// Database model for person entity type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "person_entity_type", rename_all = "PascalCase")]
pub enum RelationshipRole {
    Customer,
    Employee,
    Shareholder,
    Director,
    BeneficialOwner,
    Agent,
    Vendor,
    Partner,
    RegulatoryContact,
    EmergencyContact,
    SystemAdmin,
    Other,
}

/// # Documentation
/// - Entity reference table for managing person-to-entity relationships
///
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EntityReferenceModel {

    pub id: Uuid,

    /// # Documentation
    /// - References PersonModel.id
    ///
    /// # Finder Method (use index)
    /// - find_by_person_id
    pub person_id: Uuid,

    /// # Documentation
    /// - Type of entity relationship
    #[serde(serialize_with = "serialize_person_entity_type", deserialize_with = "deserialize_person_entity_type")]
    pub entity_role: RelationshipRole,

    /// # Documentation
    /// - External identifier for the reference (e.g., customer ID, employee ID)
    ///
    /// # Finder Method (use index)
    /// - find_by_reference_external_id_hash
    pub reference_external_id: HeaplessString<50>,

    pub reference_details_l1: Option<HeaplessString<50>>,
    pub reference_details_l2: Option<HeaplessString<50>>,
    pub reference_details_l3: Option<HeaplessString<50>>,

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

impl Identifiable for EntityReferenceModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Auditable for EntityReferenceModel {
    fn get_audit_log_id(&self) -> Option<Uuid> {
        self.audit_log_id
    }
}

// Serialization functions for RelationshipRole
pub fn serialize_person_entity_type<S>(entity_role: &RelationshipRole, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let type_str = match entity_role {
        RelationshipRole::Customer => "customer",
        RelationshipRole::Employee => "employee",
        RelationshipRole::Shareholder => "shareholder",
        RelationshipRole::Director => "director",
        RelationshipRole::BeneficialOwner => "beneficialowner",
        RelationshipRole::Agent => "agent",
        RelationshipRole::Vendor => "vendor",
        RelationshipRole::Partner => "partner",
        RelationshipRole::RegulatoryContact => "regulatorycontact",
        RelationshipRole::EmergencyContact => "emergencycontact",
        RelationshipRole::SystemAdmin => "systemadmin",
        RelationshipRole::Other => "other",
    };
    serializer.serialize_str(type_str)
}

pub fn deserialize_person_entity_type<'de, D>(deserializer: D) -> Result<RelationshipRole, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "customer" => Ok(RelationshipRole::Customer),
        "employee" => Ok(RelationshipRole::Employee),
        "shareholder" => Ok(RelationshipRole::Shareholder),
        "director" => Ok(RelationshipRole::Director),
        "beneficialowner" => Ok(RelationshipRole::BeneficialOwner),
        "agent" => Ok(RelationshipRole::Agent),
        "vendor" => Ok(RelationshipRole::Vendor),
        "partner" => Ok(RelationshipRole::Partner),
        "regulatorycontact" => Ok(RelationshipRole::RegulatoryContact),
        "emergencycontact" => Ok(RelationshipRole::EmergencyContact),
        "systemadmin" => Ok(RelationshipRole::SystemAdmin),
        "other" => Ok(RelationshipRole::Other),
        _ => Err(serde::de::Error::custom(format!("Unknown person entity type: {s}"))),
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EntityReferenceIdxModel {
    pub id: Uuid,
    pub person_id: Uuid,
    pub reference_external_id_hash: i64,
}

impl HasPrimaryKey for EntityReferenceIdxModel {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}

impl IndexAware for EntityReferenceModel {
    type IndexType = EntityReferenceIdxModel;
    
    fn to_index(&self) -> Self::IndexType {
        use crate::utils::hash_as_i64;
        let reference_external_id_hash = hash_as_i64(&self.reference_external_id.as_str()).unwrap_or(0);
        
        EntityReferenceIdxModel {
            id: self.id,
            person_id: self.person_id,
            reference_external_id_hash,
        }
    }
}

impl Identifiable for EntityReferenceIdxModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Index for EntityReferenceIdxModel {}

impl Indexable for EntityReferenceIdxModel {
    fn i64_keys(&self) -> HashMap<String, Option<i64>> {
        let mut keys = HashMap::new();
        keys.insert("reference_external_id_hash".to_string(), Some(self.reference_external_id_hash));
        keys
    }

    fn uuid_keys(&self) -> HashMap<String, Option<Uuid>> {
        let mut keys = HashMap::new();
        keys.insert("person_id".to_string(), Some(self.person_id));
        keys
    }
}

pub type EntityReferenceIdxModelCache = IdxModelCache<EntityReferenceIdxModel>;
