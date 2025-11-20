use business_core_db::models::person::portfolio::PortfolioModel;
use rust_decimal::Decimal;
use uuid::Uuid;

pub fn create_test_portfolio() -> PortfolioModel {
    PortfolioModel {
        id: Uuid::new_v4(),
        person_id: Uuid::new_v4(),
        total_accounts: 3,
        total_balance: Decimal::from(10000),
        total_loan_outstanding_main: Some(Decimal::from(5000)),
        total_loan_outstanding_grantor: Some(Decimal::from(2000)),
        risk_score: Some(Decimal::new(750, 2)), // 7.50
        compliance_status: Uuid::new_v4(),
        predecessor_1: None,
        predecessor_2: None,
        predecessor_3: None,
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}