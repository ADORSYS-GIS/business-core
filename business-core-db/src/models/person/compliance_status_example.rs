use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use super::common_enums::KycStatus;

/// Database model for Customer compliance status
/// # Audit
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct ComplianceStatusModel {
    pub id: Uuid,
    pub person_id: Uuid,
    #[serde(serialize_with = "super::common_enums::serialize_kyc_status", deserialize_with = "super::common_enums::deserialize_kyc_status")]
    pub kyc_status: KycStatus,
    pub sanctions_checked: bool,
    pub last_screening_date: Option<DateTime<Utc>>,
}