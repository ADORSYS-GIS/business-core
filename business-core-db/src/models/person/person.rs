use heapless::String as HeaplessString;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use crate::models::auditable::Auditable;
use crate::models::identifiable::Identifiable;
use std::collections::HashMap;
use crate::{HasPrimaryKey, IdxModelCache, Indexable};
use crate::models::{Index, IndexAware};

/// Database model for person type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "person_type", rename_all = "PascalCase")]
pub enum PersonType {
    Natural,
    Legal,
    System,
    Integration,
    Unknown,
}

impl std::fmt::Display for PersonType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PersonType::Natural => write!(f, "Natural"),
            PersonType::Legal => write!(f, "Legal"),
            PersonType::System => write!(f, "System"),
            PersonType::Integration => write!(f, "Integration"),
            PersonType::Unknown => write!(f, "Unknown"),
        }
    }
}

/// Database model for Person
/// Represents a person throughout the system for business audit and tracking purposes
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PersonModel {
    pub id: Uuid,
    
    #[serde(serialize_with = "serialize_person_type", deserialize_with = "deserialize_person_type")]
    pub person_type: PersonType,
    
    pub display_name: HeaplessString<100>,

    /// External identifier (e.g., employee ID, badge number, system ID)
    pub external_identifier: Option<HeaplessString<50>>,

    pub entity_reference_count: i32,
    
    /// References PersonModel.id for organizational hierarchy
    pub organization_person_id: Option<Uuid>,
    
    /// Encoded type and value of up to 5 messaging methods (`type:value`)
    pub messaging_info1: Option<HeaplessString<50>>,
    pub messaging_info2: Option<HeaplessString<50>>,
    pub messaging_info3: Option<HeaplessString<50>>,
    pub messaging_info4: Option<HeaplessString<50>>,
    pub messaging_info5: Option<HeaplessString<50>>,
    
    /// Department within organization
    pub department: Option<HeaplessString<50>>,

    /// References LocationModel.id for person's location
    pub location_id: Option<Uuid>,
    
    /// References PersonModel.id for duplicate tracking
    pub duplicate_of_person_id: Option<Uuid>,

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

impl Identifiable for PersonModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Auditable for PersonModel {
    fn get_audit_log_id(&self) -> Option<Uuid> {
        self.audit_log_id
    }
}

/// Index model for Person
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PersonIdxModel {
    pub id: Uuid,
    pub external_identifier_hash: Option<i64>,
    pub organization_person_id: Option<Uuid>,
    pub duplicate_of_person_id: Option<Uuid>,
}

impl HasPrimaryKey for PersonIdxModel {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}

impl IndexAware for PersonModel {
    type IndexType = PersonIdxModel;
    
    fn to_index(&self) -> Self::IndexType {
        let external_identifier_hash = self.external_identifier.as_ref().map(|ext_id| {
            use std::hash::{Hash, Hasher};
            use twox_hash::XxHash64;
            let mut hasher = XxHash64::with_seed(0);
            ext_id.as_str().hash(&mut hasher);
            hasher.finish() as i64
        });

        PersonIdxModel {
            id: self.id,
            external_identifier_hash,
            organization_person_id: self.organization_person_id,
            duplicate_of_person_id: self.duplicate_of_person_id,
        }
    }
}

impl Identifiable for PersonIdxModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Index for PersonIdxModel {}

impl Indexable for PersonIdxModel {
    fn i64_keys(&self) -> HashMap<String, Option<i64>> {
        let mut keys = HashMap::new();
        keys.insert(
            "external_identifier_hash".to_string(),
            self.external_identifier_hash,
        );
        keys
    }

    fn uuid_keys(&self) -> HashMap<String, Option<Uuid>> {
        let mut keys = HashMap::new();
        keys.insert(
            "organization_person_id".to_string(),
            self.organization_person_id,
        );
        keys.insert(
            "duplicate_of_person_id".to_string(),
            self.duplicate_of_person_id,
        );
        keys
    }
}

pub type PersonIdxModelCache = IdxModelCache<PersonIdxModel>;

// Serialization functions for PersonType
fn serialize_person_type<S>(person_type: &PersonType, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    let type_str = match person_type {
        PersonType::Natural => "Natural",
        PersonType::Legal => "Legal",
        PersonType::System => "System",
        PersonType::Integration => "Integration",
        PersonType::Unknown => "Unknown",
    };
    serializer.serialize_str(type_str)
}

fn deserialize_person_type<'de, D>(deserializer: D) -> Result<PersonType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "Natural" => Ok(PersonType::Natural),
        "Legal" => Ok(PersonType::Legal),
        "System" => Ok(PersonType::System),
        "Integration" => Ok(PersonType::Integration),
        "Unknown" => Ok(PersonType::Unknown),
        _ => Err(serde::de::Error::custom(format!("Unknown person type: {s}"))),
    }
}