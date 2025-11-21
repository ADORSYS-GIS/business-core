use business_core_db::models::description::named::NamedModel;
use crate::utils::{get_heapless_string, get_optional_heapless_string, TryFromRow};
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use async_trait::async_trait;

pub struct NamedRepositoryImpl {
    pub executor: Executor,
}

impl NamedRepositoryImpl {
    pub fn new(executor: Executor) -> Self {
        Self { executor }
    }
}

impl TryFromRow<PgRow> for NamedModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(NamedModel {
            id: row.get("id"),
            name_l1: get_heapless_string(row, "name_l1")?,
            name_l2: get_optional_heapless_string(row, "name_l2")?,
            name_l3: get_optional_heapless_string(row, "name_l3")?,
            name_l4: get_optional_heapless_string(row, "name_l4")?,
            description_l1: get_optional_heapless_string(row, "description_l1")?,
            description_l2: get_optional_heapless_string(row, "description_l2")?,
            description_l3: get_optional_heapless_string(row, "description_l3")?,
            description_l4: get_optional_heapless_string(row, "description_l4")?,
            antecedent_hash: row.get("antecedent_hash"),
            antecedent_audit_log_id: row.get("antecedent_audit_log_id"),
            hash: row.get("hash"),
            audit_log_id: row.try_get("audit_log_id").ok(),
        })
    }
}

#[async_trait]
impl TransactionAware for NamedRepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        Ok(())
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        Ok(())
    }
}