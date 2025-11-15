use heapless::String as HeaplessString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use crate::models::auditable::Auditable;
use crate::models::identifiable::Identifiable;
use std::collections::HashMap;
use crate::{HasPrimaryKey, IdxModelCache, Indexable};
use crate::models::{Index, IndexAware};

/// Database model for location type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "location_type", rename_all = "PascalCase")]
pub enum LocationType {
    Residential,
    Business,
    Mailing,
    Temporary,
    Branch,
    Community,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LocationModel {
    pub id: Uuid,

    /// - Structured location components - 4 street lines
    pub street_line1: HeaplessString<50>,
    pub street_line2: Option<HeaplessString<50>>,
    pub street_line3: Option<HeaplessString<50>>,
    pub street_line4: Option<HeaplessString<50>>,

    pub locality_id: Uuid,
    pub postal_code: Option<HeaplessString<20>>,

    /// Geographical coordinates (decimal degrees)
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
    pub accuracy_meters: Option<f32>,
    
    #[serde(serialize_with = "serialize_location_type", deserialize_with = "deserialize_location_type")]
    pub location_type: LocationType,

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
    /// This field, together with `id`, forms the composite primary key in the audit table
    pub audit_log_id: Option<Uuid>,
}

impl Identifiable for LocationModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Auditable for LocationModel {
    fn get_audit_log_id(&self) -> Option<Uuid> {
        self.audit_log_id
    }
}


#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LocationIdxModel {
    pub id: Uuid,
    pub locality_id: Uuid,
}

impl HasPrimaryKey for LocationIdxModel {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}

impl IndexAware for LocationModel {
    type IndexType = LocationIdxModel;
    
    fn to_index(&self) -> Self::IndexType {
        LocationIdxModel {
            id: self.id,
            locality_id: self.locality_id,
        }
    }
}

impl Identifiable for LocationIdxModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Index for LocationIdxModel {}

impl Indexable for LocationIdxModel {
    fn i64_keys(&self) -> HashMap<String, Option<i64>> {
        HashMap::new()
    }

    fn uuid_keys(&self) -> HashMap<String, Option<Uuid>> {
        let mut keys = HashMap::new();
        keys.insert("locality_id".to_string(), Some(self.locality_id));
        keys
    }
}

pub type LocationIdxModelCache = IdxModelCache<LocationIdxModel>;


fn serialize_location_type<S>(location_type: &LocationType, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(match location_type {
        LocationType::Residential => "Residential",
        LocationType::Business => "Business",
        LocationType::Mailing => "Mailing",
        LocationType::Temporary => "Temporary",
        LocationType::Branch => "Branch",
        LocationType::Community => "Community",
        LocationType::Other => "Other",
    })
}

fn deserialize_location_type<'de, D>(deserializer: D) -> Result<LocationType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "Residential" => Ok(LocationType::Residential),
        "Business" => Ok(LocationType::Business),
        "Mailing" => Ok(LocationType::Mailing),
        "Temporary" => Ok(LocationType::Temporary),
        "Branch" => Ok(LocationType::Branch),
        "Community" => Ok(LocationType::Community),
        "Other" => Ok(LocationType::Other),
        _ => Err(serde::de::Error::custom(format!("Unknown location type: {}", s))),
    }
}
