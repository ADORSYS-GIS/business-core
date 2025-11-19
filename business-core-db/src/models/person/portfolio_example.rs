use chrono::{DateTime, Utc};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Database model for Customer Portfolio summary
/// # Audit
#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct PortfolioModel {
    pub id: Uuid,
    pub person_id: Uuid,
    pub total_accounts: i64,
    pub total_balance: Decimal,
    pub total_loan_outstanding_main: Option<Decimal>,
    pub total_loan_outstanding_grantor: Option<Decimal>,
    pub risk_score: Option<Decimal>,
    pub compliance_status: Uuid,
}