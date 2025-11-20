use business_core_db::models::person::compliance_status::ComplianceStatusModel;
use crate::utils::TryFromRow;
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use async_trait::async_trait;

pub struct ComplianceStatusRepositoryImpl {
    pub executor: Executor,
}

impl ComplianceStatusRepositoryImpl {
    pub fn new(executor: Executor) -> Self {
        Self { executor }
    }
}

impl TryFromRow<PgRow> for ComplianceStatusModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(ComplianceStatusModel {
            id: row.get("id"),
            person_id: row.get("person_id"),
            kyc_status: row.get("kyc_status"),
            sanctions_checked: row.get("sanctions_checked"),
            last_screening_date: row.try_get("last_screening_date").ok(),
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
impl TransactionAware for ComplianceStatusRepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        Ok(())
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        Ok(())
    }
}