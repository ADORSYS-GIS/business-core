use business_core_db::models::product::product::{
    ProductModel, ProductType, PostingFrequency, InterestCalculationMethod,
    MaintenanceFeeFrequency, ProductAccrualFrequency,
};
use rust_decimal::Decimal;
use uuid::Uuid;
use chrono::NaiveDate;

pub fn create_test_product(name: Uuid, account_gl_mapping: Uuid, fee_type_gl_mapping: Uuid) -> ProductModel {
    ProductModel {
        id: Uuid::new_v4(),
        name,
        product_type: ProductType::CASA,
        minimum_balance: Decimal::from(100),
        maximum_balance: Some(Decimal::from(1000000)),
        overdraft_allowed: false,
        overdraft_limit: None,
        interest_calculation_method: InterestCalculationMethod::DailyBalance,
        interest_posting_frequency: PostingFrequency::Monthly,
        dormancy_threshold_days: 365,
        minimum_opening_balance: Decimal::from(100),
        closure_fee: Decimal::from(50),
        maintenance_fee: Some(Decimal::from(10)),
        maintenance_fee_frequency: MaintenanceFeeFrequency::Monthly,
        default_dormancy_days: Some(365),
        default_overdraft_limit: None,
        per_transaction_limit: Some(Decimal::from(50000)),
        daily_transaction_limit: Some(Decimal::from(100000)),
        weekly_transaction_limit: Some(Decimal::from(500000)),
        monthly_transaction_limit: Some(Decimal::from(2000000)),
        overdraft_interest_rate: None,
        accrual_frequency: ProductAccrualFrequency::Daily,
        interest_rate_tier_1: None,
        interest_rate_tier_2: None,
        interest_rate_tier_3: None,
        interest_rate_tier_4: None,
        interest_rate_tier_5: None,
        account_gl_mapping,
        fee_type_gl_mapping,
        is_active: true,
        valid_from: NaiveDate::from_ymd_opt(2024, 1, 1).unwrap(),
        valid_to: None,
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}