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
/// Defines the various roles a person can have in relation to the bank or other persons
/// - Qualifying persons that are in relationship with the bank (like customer, employee, etc.)
/// - Qualifying persons in relation to other persons (guarantor, guardian, etc.)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "person_entity_type", rename_all = "PascalCase")]
pub enum RelationshipRole {
    /// Customer of the bank
    Customer,
    /// Employee of the bank or organization
    Employee,
    /// Shareholder of a business entity
    Shareholder,
    /// Director of a company
    Director,
    /// Ultimate beneficial owner - natural person who ultimately owns/controls the business
    UltimateBeneficialOwner,
    /// Agent representing the person or entity
    Agent,
    /// Vendor providing services
    Vendor,
    /// Partner in a business relationship
    Partner,
    /// Contact person for regulatory purposes
    RegulatoryContact,
    /// Emergency contact person (no financial authority)
    EmergencyContact,
    /// System administrator with high-level access
    SystemAdmin,
    /// Person who guarantees to pay a borrower's debt if they default
    Guarantor,
    /// Legal guardian or tutor for minor accounts
    LegalGuardian,
    /// Person with power of attorney to act on behalf of the customer
    PowerOfAttorney,
    /// Designated beneficiary to receive assets upon death of account holder
    Beneficiary,
    /// Authorized to conduct transactions on behalf of the account holder
    AuthorizedSignatory,
    /// Person with significant responsibility to control, manage, or direct the entity
    ControllingPerson,
    /// Person with delegated access (often temporary or specific authority)
    Delegate,
    /// Administrator with high-level access to manage accounts and users
    Administrator,
    /// Other relationship type not covered by specific categories
    Other,
}

/// Database model for customer relationship status enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "customer_relationship_status", rename_all = "PascalCase")]
pub enum RelationshipStatus {
    /// Relationship is currently active
    Active,
    /// Relationship is inactive
    Inactive,
    /// Relationship is pending approval
    Pending,
    /// Relationship has been terminated
    Terminated,
    /// Unknown status
    Unknown,
}

impl std::fmt::Display for RelationshipStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RelationshipStatus::Active => write!(f, "Active"),
            RelationshipStatus::Inactive => write!(f, "Inactive"),
            RelationshipStatus::Pending => write!(f, "Pending"),
            RelationshipStatus::Terminated => write!(f, "Terminated"),
            RelationshipStatus::Unknown => write!(f, "Unknown"),
        }
    }
}

/// # Documentation
/// - Entity reference table for managing person-to-entity relationships
///
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct EntityReferenceModel {

    pub id: Uuid,

    /// # Documentation
    /// - References PersonModel.id
    /// - Can be the customer of the bank
    /// - For individual: can be the guarantor, guardian, beneficiary, of the related_person
    /// - For business: the director, UBO, authorized signatory, etc... of the related_person
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

    /// The person ID of the individual related to the primary person
    pub related_person_id: Option<Uuid>,

    /// The date when this relationship becomes effective
    pub start_date: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Optional date when this relationship ends
    pub end_date: Option<chrono::DateTime<chrono::Utc>>,
    
    /// Status of the relationship
    #[serde(
        serialize_with = "serialize_relationship_status",
        deserialize_with = "deserialize_relationship_status"
    )]
    pub status: Option<RelationshipStatus>,

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
        RelationshipRole::UltimateBeneficialOwner => "ultimatebeneficialowner",
        RelationshipRole::Agent => "agent",
        RelationshipRole::Vendor => "vendor",
        RelationshipRole::Partner => "partner",
        RelationshipRole::RegulatoryContact => "regulatorycontact",
        RelationshipRole::EmergencyContact => "emergencycontact",
        RelationshipRole::SystemAdmin => "systemadmin",
        RelationshipRole::Guarantor => "guarantor",
        RelationshipRole::LegalGuardian => "legalguardian",
        RelationshipRole::PowerOfAttorney => "powerofattorney",
        RelationshipRole::Beneficiary => "beneficiary",
        RelationshipRole::AuthorizedSignatory => "authorizedsignatory",
        RelationshipRole::ControllingPerson => "controllingperson",
        RelationshipRole::Delegate => "delegate",
        RelationshipRole::Administrator => "administrator",
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
        "ultimatebeneficialowner" => Ok(RelationshipRole::UltimateBeneficialOwner),
        "agent" => Ok(RelationshipRole::Agent),
        "vendor" => Ok(RelationshipRole::Vendor),
        "partner" => Ok(RelationshipRole::Partner),
        "regulatorycontact" => Ok(RelationshipRole::RegulatoryContact),
        "emergencycontact" => Ok(RelationshipRole::EmergencyContact),
        "systemadmin" => Ok(RelationshipRole::SystemAdmin),
        "guarantor" => Ok(RelationshipRole::Guarantor),
        "legalguardian" => Ok(RelationshipRole::LegalGuardian),
        "powerofattorney" => Ok(RelationshipRole::PowerOfAttorney),
        "beneficiary" => Ok(RelationshipRole::Beneficiary),
        "authorizedsignatory" => Ok(RelationshipRole::AuthorizedSignatory),
        "controllingperson" => Ok(RelationshipRole::ControllingPerson),
        "delegate" => Ok(RelationshipRole::Delegate),
        "administrator" => Ok(RelationshipRole::Administrator),
        "other" => Ok(RelationshipRole::Other),
        _ => Err(serde::de::Error::custom(format!("Unknown person entity type: {s}"))),
    }
}

// Serialization functions for RelationshipStatus
pub fn serialize_relationship_status<S>(status: &Option<RelationshipStatus>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match status {
        Some(s) => serializer.serialize_str(&s.to_string()),
        None => serializer.serialize_none(),
    }
}

pub fn deserialize_relationship_status<'de, D>(deserializer: D) -> Result<Option<RelationshipStatus>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt: Option<String> = Option::deserialize(deserializer)?;
    match opt {
        Some(value_str) => {
            let status = match value_str.as_str() {
                "Active" => RelationshipStatus::Active,
                "Inactive" => RelationshipStatus::Inactive,
                "Pending" => RelationshipStatus::Pending,
                "Terminated" => RelationshipStatus::Terminated,
                "Unknown" => RelationshipStatus::Unknown,
                _ => return Err(serde::de::Error::custom(format!("Invalid RelationshipStatus: {value_str}"))),
            };
            Ok(Some(status))
        }
        None => Ok(None),
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
