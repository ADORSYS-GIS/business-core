use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "risk_rating", rename_all = "PascalCase")]
pub enum RiskRating {
    Low,
    Medium,
    High,
    Blacklisted,
}

impl std::fmt::Display for RiskRating {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RiskRating::Low => write!(f, "Low"),
            RiskRating::Medium => write!(f, "Medium"),
            RiskRating::High => write!(f, "High"),
            RiskRating::Blacklisted => write!(f, "Blacklisted"),
        }
    }
}

impl FromStr for RiskRating {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Low" => Ok(RiskRating::Low),
            "Medium" => Ok(RiskRating::Medium),
            "High" => Ok(RiskRating::High),
            "Blacklisted" => Ok(RiskRating::Blacklisted),
            _ => Err(()),
        }
    }
}

pub fn serialize_risk_rating<S>(value: &RiskRating, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let value_str = match value {
        RiskRating::Low => "Low",
        RiskRating::Medium => "Medium",
        RiskRating::High => "High",
        RiskRating::Blacklisted => "Blacklisted",
    };
    serializer.serialize_str(value_str)
}

pub fn deserialize_risk_rating<'de, D>(deserializer: D) -> Result<RiskRating, D::Error>
where
    D: Deserializer<'de>,
{
    let value_str = String::deserialize(deserializer)?;
    match value_str.as_str() {
        "Low" => Ok(RiskRating::Low),
        "Medium" => Ok(RiskRating::Medium),
        "High" => Ok(RiskRating::High),
        "Blacklisted" => Ok(RiskRating::Blacklisted),
        _ => Err(serde::de::Error::custom(format!("Invalid RiskRating: {value_str}"))),
    }
}


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