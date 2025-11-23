use business_core_db::models::product::interest_rate_tier::InterestRateTierModel;
use rust_decimal::Decimal;
use uuid::Uuid;

pub fn create_test_interest_rate_tier() -> InterestRateTierModel {
    InterestRateTierModel {
        id: Uuid::new_v4(),
        name: Uuid::new_v4(),
        minimum_balance: Decimal::new(100, 2), // 1.00
        maximum_balance: Some(Decimal::new(1000, 2)), // 10.00
        interest_rate: Decimal::new(5, 2), // 0.05
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}