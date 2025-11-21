use business_core_db::models::person::document::DocumentModel;
use crate::utils::{get_optional_heapless_string, TryFromRow};
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use async_trait::async_trait;

pub struct DocumentRepositoryImpl {
    pub executor: Executor,
}

impl DocumentRepositoryImpl {
    pub fn new(executor: Executor) -> Self {
        Self { executor }
    }
}

impl TryFromRow<PgRow> for DocumentModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(DocumentModel {
            id: row.get("id"),
            person_id: row.get("person_id"),
            document_type: get_optional_heapless_string(row, "document_type")?.ok_or("document_type is required")?,
            document_path: get_optional_heapless_string(row, "document_path")?,
            status: row.get("status"),
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
impl TransactionAware for DocumentRepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        Ok(())
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        Ok(())
    }
}