use serde::{Deserialize, Deserializer, Serialize, Serializer};
use std::str::FromStr;
use uuid::Uuid;

use super::common_enums::RiskRating;

/// Database model for Customer table
/// Audit
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct CustomerModel {
    pub id: Uuid,
    pub person_id: Uuid,
    #[serde(
        serialize_with = "super::common_enums::serialize_risk_rating",
        deserialize_with = "super::common_enums::deserialize_risk_rating"
    )]
    pub risk_rating: RiskRating,
    #[serde(
        serialize_with = "serialize_customer_status",
        deserialize_with = "deserialize_customer_status"
    )]
    pub status: CustomerStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "customer_status", rename_all = "PascalCase")]
pub enum CustomerStatus {
    Active,
    PendingVerification,
    Deceased,
    Dissolved,
    Blacklisted,
}

impl FromStr for CustomerStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Active" => Ok(CustomerStatus::Active),
            "PendingVerification" => Ok(CustomerStatus::PendingVerification),
            "Deceased" => Ok(CustomerStatus::Deceased),
            "Dissolved" => Ok(CustomerStatus::Dissolved),
            "Blacklisted" => Ok(CustomerStatus::Blacklisted),
            _ => Err(()),
        }
    }
}

// ============================================================================
// CUSTOM SERIALIZATION FUNCTIONS
// ============================================================================

fn serialize_customer_status<S>(value: &CustomerStatus, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let value_str = match value {
        CustomerStatus::Active => "Active",
        CustomerStatus::PendingVerification => "PendingVerification",
        CustomerStatus::Deceased => "Deceased",
        CustomerStatus::Dissolved => "Dissolved",
        CustomerStatus::Blacklisted => "Blacklisted",
    };
    serializer.serialize_str(value_str)
}

fn deserialize_customer_status<'de, D>(deserializer: D) -> Result<CustomerStatus, D::Error>
where
    D: Deserializer<'de>,
{
    let value_str = String::deserialize(deserializer)?;
    match value_str.as_str() {
        "Active" => Ok(CustomerStatus::Active),
        "PendingVerification" => Ok(CustomerStatus::PendingVerification),
        "Deceased" => Ok(CustomerStatus::Deceased),
        "Dissolved" => Ok(CustomerStatus::Dissolved),
        "Blacklisted" => Ok(CustomerStatus::Blacklisted),
        _ => Err(serde::de::Error::custom(format!(
            "Invalid CustomerStatus: {value_str}"
        ))),
    }
}