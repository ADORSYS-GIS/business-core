use chrono::{DateTime, Utc};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlx::FromRow;
use std::str::FromStr;
use uuid::Uuid;
use crate::models::auditable::Auditable;
use crate::models::identifiable::Identifiable;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "kyc_status", rename_all = "PascalCase")]
pub enum KycStatus {
    NotStarted,
    InProgress,
    Pending,
    Complete,
    Approved,
    Rejected,
    RequiresUpdate,
    Failed,
}

impl FromStr for KycStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NotStarted" => Ok(KycStatus::NotStarted),
            "InProgress" => Ok(KycStatus::InProgress),
            "Pending" => Ok(KycStatus::Pending),
            "Complete" => Ok(KycStatus::Complete),
            "Approved" => Ok(KycStatus::Approved),
            "Rejected" => Ok(KycStatus::Rejected),
            "RequiresUpdate" => Ok(KycStatus::RequiresUpdate),
            "Failed" => Ok(KycStatus::Failed),
            _ => Err(()),
        }
    }
}

pub fn serialize_kyc_status<S>(value: &KycStatus, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let value_str = match value {
        KycStatus::NotStarted => "NotStarted",
        KycStatus::InProgress => "InProgress",
        KycStatus::Pending => "Pending",
        KycStatus::Complete => "Complete",
        KycStatus::Approved => "Approved",
        KycStatus::Rejected => "Rejected",
        KycStatus::RequiresUpdate => "RequiresUpdate",
        KycStatus::Failed => "Failed",
    };
    serializer.serialize_str(value_str)
}

pub fn deserialize_kyc_status<'de, D>(deserializer: D) -> Result<KycStatus, D::Error>
where
    D: Deserializer<'de>,
{
    let value_str = String::deserialize(deserializer)?;
    match value_str.as_str() {
        "NotStarted" => Ok(KycStatus::NotStarted),
        "InProgress" => Ok(KycStatus::InProgress),
        "Pending" => Ok(KycStatus::Pending),
        "Complete" => Ok(KycStatus::Complete),
        "Approved" => Ok(KycStatus::Approved),
        "Rejected" => Ok(KycStatus::Rejected),
        "RequiresUpdate" => Ok(KycStatus::RequiresUpdate),
        "Failed" => Ok(KycStatus::Failed),
        _ => Err(serde::de::Error::custom(format!("Invalid KycStatus: {value_str}"))),
    }
}

/// Database model for Person compliance status
/// 
/// This entity tracks KYC and compliance status for persons.
/// It is auditable but not indexable - accessed by ID only.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ComplianceStatusModel {
    pub id: Uuid,
    
    /// The person this compliance status belongs to
    pub person_id: Uuid,
    
    /// Current KYC status
    #[serde(serialize_with = "serialize_kyc_status", deserialize_with = "deserialize_kyc_status")]
    pub kyc_status: KycStatus,
    
    /// Whether sanctions have been checked
    pub sanctions_checked: bool,
    
    /// Date of last screening (optional)
    pub last_screening_date: Option<DateTime<Utc>>,
    
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

impl Identifiable for ComplianceStatusModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Auditable for ComplianceStatusModel {
    fn get_audit_log_id(&self) -> Option<Uuid> {
        self.audit_log_id
    }
}