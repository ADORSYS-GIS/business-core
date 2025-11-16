use business_core_db::models::reason_and_purpose::reason_reference::ReasonReferenceModel;
use crate::utils::{get_optional_heapless_string, TryFromRow};
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use async_trait::async_trait;

pub struct ReasonReferenceRepositoryImpl {
    pub executor: Executor,
}

impl ReasonReferenceRepositoryImpl {
    pub fn new(executor: Executor) -> Self {
        Self { executor }
    }
}

impl TryFromRow<PgRow> for ReasonReferenceModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(ReasonReferenceModel {
            id: row.get("id"),
            reason_id: row.get("reason_id"),
            entity_id: row.get("entity_id"),
            additional_details: get_optional_heapless_string(row, "additional_details")?,
            entity_type: row.get("entity_type"),
            antecedent_hash: row.get("antecedent_hash"),
            antecedent_audit_log_id: row.get("antecedent_audit_log_id"),
            hash: row.get("hash"),
            audit_log_id: row.try_get("audit_log_id").ok(),
        })
    }
}

#[async_trait]
impl TransactionAware for ReasonReferenceRepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        Ok(())
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        Ok(())
    }
}