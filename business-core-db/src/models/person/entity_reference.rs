use heapless::String as HeaplessString;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use std::collections::HashMap;
use crate::{HasPrimaryKey, IdxModelCache, Indexable};

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
/// # Index
/// 
/// # Audit
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EntityReferenceModel {
    /// # Finder Methods (use index)
    /// - find_by_id
    /// - find_by_ids
    /// - exists_by_id
    pub id: Uuid,

    /// # Documentation
    /// - References PersonModel.id
    /// 
    /// # Finder Method (use index)
    /// - find_ids_by_person_id
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

    pub last_audit_log_id: Option<Uuid>,
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

#[derive(Debug, Clone, FromRow)]
pub struct EntityReferenceIdxModel {

    pub entity_reference_id: Uuid,

    pub person_id: Uuid,

    pub reference_external_id_hash: i64,
}
