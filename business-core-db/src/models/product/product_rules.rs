use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Frequency for interest posting
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PostingFrequency {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Annually,
}

/// Frequency for interest accrual
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProductAccrualFrequency {
    Daily,
    BusinessDaysOnly,
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductRules {
    pub minimum_balance: Decimal,
    pub maximum_balance: Option<Decimal>,
    pub daily_transaction_limit: Option<Decimal>,
    pub monthly_transaction_limit: Option<Decimal>,
    pub overdraft_allowed: bool,
    pub overdraft_limit: Option<Decimal>,
    pub interest_calculation_method: heapless::String<50>,
    pub interest_posting_frequency: PostingFrequency,
    pub dormancy_threshold_days: i32,
    pub minimum_opening_balance: Decimal,
    pub closure_fee: Decimal,
    pub maintenance_fee: Option<Decimal>,
    pub maintenance_fee_frequency: Option<heapless::String<50>>,
    pub default_dormancy_days: Option<i32>,
    pub default_overdraft_limit: Option<Decimal>,
    pub per_transaction_limit: Option<Decimal>,
    pub overdraft_interest_rate: Option<Decimal>,
    pub accrual_frequency: ProductAccrualFrequency,
}