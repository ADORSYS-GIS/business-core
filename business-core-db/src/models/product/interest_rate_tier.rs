use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InterestRateTierModel {
    pub id: Uuid,
    pub name: Uuid,
    pub minimum_balance: Decimal,
    pub maximum_balance: Option<Decimal>,
    pub interest_rate: Decimal,
}