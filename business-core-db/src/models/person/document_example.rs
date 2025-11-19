use heapless::String as HeaplessString;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;
use uuid::Uuid;

/// Database model for Customer documents
/// # Audit
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct DocumentModel {
    pub id: Uuid,
    pub person_id: Uuid,
    pub document_type: HeaplessString<50>,
    pub document_path: Option<HeaplessString<500>>,
    #[serde(
        serialize_with = "serialize_document_status",
        deserialize_with = "deserialize_document_status"
    )]
    pub status: DocumentStatus,
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