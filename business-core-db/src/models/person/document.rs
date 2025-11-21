use heapless::String as HeaplessString;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlx::FromRow;
use std::str::FromStr;
use uuid::Uuid;
use crate::models::auditable::Auditable;
use crate::models::identifiable::Identifiable;

/// # Documentation
/// Database model for Customer documents
/// 
/// Tracks document information for a person including:
/// - Document type (e.g., passport, driver's license)
/// - Document storage path
/// - Document verification status
///
/// This entity is auditable but not indexable - accessed by ID only.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DocumentModel {
    pub id: Uuid,
    
    /// Reference to the person who owns this document
    pub person_id: Uuid,
    
    /// Type of document (e.g., "Passport", "ID Card", "Driver License")
    pub document_type: HeaplessString<50>,
    
    /// Storage path or URL to the document
    pub document_path: Option<HeaplessString<500>>,
    
    /// Current status of the document
    #[serde(
        serialize_with = "serialize_document_status",
        deserialize_with = "deserialize_document_status"
    )]
    pub status: DocumentStatus,
    
    /// First predecessor reference (nullable)
    pub predecessor_1: Option<Uuid>,
    
    /// Second predecessor reference (nullable)
    pub predecessor_2: Option<Uuid>,
    
    /// Third predecessor reference (nullable)
    pub predecessor_3: Option<Uuid>,
    
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

impl Identifiable for DocumentModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Auditable for DocumentModel {
    fn get_audit_log_id(&self) -> Option<Uuid> {
        self.audit_log_id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "document_status", rename_all = "PascalCase")]
pub enum DocumentStatus {
    Uploaded,
    Verified,
    Rejected,
    Expired,
}

impl std::fmt::Display for DocumentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentStatus::Uploaded => write!(f, "Uploaded"),
            DocumentStatus::Verified => write!(f, "Verified"),
            DocumentStatus::Rejected => write!(f, "Rejected"),
            DocumentStatus::Expired => write!(f, "Expired"),
        }
    }
}

impl FromStr for DocumentStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Uploaded" => Ok(DocumentStatus::Uploaded),
            "Verified" => Ok(DocumentStatus::Verified),
            "Rejected" => Ok(DocumentStatus::Rejected),
            "Expired" => Ok(DocumentStatus::Expired),
            _ => Err(()),
        }
    }
}

fn serialize_document_status<S>(value: &DocumentStatus, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let value_str = match value {
        DocumentStatus::Uploaded => "Uploaded",
        DocumentStatus::Verified => "Verified",
        DocumentStatus::Rejected => "Rejected",
        DocumentStatus::Expired => "Expired",
    };
    serializer.serialize_str(value_str)
}

fn deserialize_document_status<'de, D>(deserializer: D) -> Result<DocumentStatus, D::Error>
where
    D: Deserializer<'de>,
{
    let value_str = String::deserialize(deserializer)?;
    match value_str.as_str() {
        "Uploaded" => Ok(DocumentStatus::Uploaded),
        "Verified" => Ok(DocumentStatus::Verified),
        "Rejected" => Ok(DocumentStatus::Rejected),
        "Expired" => Ok(DocumentStatus::Expired),
        _ => Err(serde::de::Error::custom(format!(
            "Invalid DocumentStatus: {value_str}"
        ))),
    }
}