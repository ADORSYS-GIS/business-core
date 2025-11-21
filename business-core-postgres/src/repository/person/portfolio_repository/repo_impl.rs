use business_core_db::models::person::portfolio::PortfolioModel;
use crate::utils::TryFromRow;
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use async_trait::async_trait;

pub struct PortfolioRepositoryImpl {
    pub executor: Executor,
}

impl PortfolioRepositoryImpl {
    pub fn new(executor: Executor) -> Self {
        Self { executor }
    }
}

impl TryFromRow<PgRow> for PortfolioModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(PortfolioModel {
            id: row.get("id"),
            person_id: row.get("person_id"),
            total_accounts: row.get("total_accounts"),
            total_balance: row.get("total_balance"),
            total_loan_outstanding_main: row.try_get("total_loan_outstanding_main").ok(),
            total_loan_outstanding_grantor: row.try_get("total_loan_outstanding_grantor").ok(),
            risk_score: row.try_get("risk_score").ok(),
            compliance_status: row.get("compliance_status"),
            predecessor_1: row.try_get("predecessor_1").ok(),
            predecessor_2: row.try_get("predecessor_2").ok(),
            predecessor_3: row.try_get("predecessor_3").ok(),
            antecedent_hash: row.get("antecedent_hash"),
            antecedent_audit_log_id: row.get("antecedent_audit_log_id"),
            hash: row.get("hash"),
            audit_log_id: row.try_get("audit_log_id").ok(),
        })
    }
}

#[async_trait]
impl TransactionAware for PortfolioRepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        Ok(())
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        Ok(())
    }
}