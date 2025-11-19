use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use super::common_enums::KycStatus;

/// Database model for Customer Portfolio summary
/// # Audit
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct CustomerPortfolioModel {
    pub id: Uuid,
    pub customer_id: Uuid,
    pub total_accounts: i64,
    pub total_balance: Decimal,
    pub total_loan_outstanding: Option<Decimal>,
    pub last_activity_date: Option<DateTime<Utc>>,
    pub risk_score: Option<Decimal>,
    #[serde(serialize_with = "super::common_enums::serialize_kyc_status", deserialize_with = "super::common_enums::deserialize_kyc_status")]
    pub kyc_status: KycStatus,
    pub sanctions_checked: bool,
    pub last_screening_date: Option<DateTime<Utc>>,
}