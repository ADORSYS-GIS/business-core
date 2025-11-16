use business_core_db::models::reason_and_purpose::compliance_metadata::{ComplianceMetadataIdxModel, ComplianceMetadataModel};
use crate::utils::{get_heapless_string, get_optional_heapless_string, TryFromRow};
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use postgres_index_cache::TransactionAwareIdxModelCache;
use parking_lot::RwLock as ParkingRwLock;
use tokio::sync::RwLock;
use std::sync::Arc;
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use async_trait::async_trait;

pub struct ComplianceMetadataRepositoryImpl {
    pub executor: Executor,
    pub compliance_metadata_idx_cache: Arc<RwLock<TransactionAwareIdxModelCache<ComplianceMetadataIdxModel>>>,
}

impl ComplianceMetadataRepositoryImpl {
    pub fn new(
        executor: Executor,
        compliance_metadata_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<ComplianceMetadataIdxModel>>>,
    ) -> Self {
        Self {
            executor,
            compliance_metadata_idx_cache: Arc::new(RwLock::new(TransactionAwareIdxModelCache::new(
                compliance_metadata_idx_cache,
            ))),
        }
    }

    pub async fn load_all_compliance_metadata_idx(
        executor: &Executor,
    ) -> Result<Vec<ComplianceMetadataIdxModel>, sqlx::Error> {
        let query = sqlx::query("SELECT * FROM compliance_metadata_idx");
        let rows = {
            let mut tx = executor.tx.lock().await;
            if let Some(transaction) = tx.as_mut() {
                query.fetch_all(&mut **transaction).await?
            } else {
                return Err(sqlx::Error::PoolTimedOut);
            }
        };
        
        let mut idx_models = Vec::with_capacity(rows.len());
        for row in rows {
            idx_models.push(ComplianceMetadataIdxModel::try_from_row(&row).map_err(sqlx::Error::Decode)?);
        }
        Ok(idx_models)
    }
}

#[async_trait]
impl TransactionAware for ComplianceMetadataRepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        self.compliance_metadata_idx_cache.read().await.on_commit().await
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        self.compliance_metadata_idx_cache.read().await.on_rollback().await
    }
}

impl TryFromRow<PgRow> for ComplianceMetadataModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(ComplianceMetadataModel {
            id: row.get("id"),
            regulatory_code: get_optional_heapless_string(row, "regulatory_code")?,
            reportable: row.get("reportable"),
            requires_sar: row.get("requires_sar"),
            requires_ctr: row.get("requires_ctr"),
            retention_years: row.get("retention_years"),
            escalation_required: row.get("escalation_required"),
            risk_score_impact: row.try_get("risk_score_impact").ok(),
            no_tipping_off: row.get("no_tipping_off"),
            jurisdictions1: get_heapless_string(row, "jurisdictions1")?,
            jurisdictions2: get_heapless_string(row, "jurisdictions2")?,
            jurisdictions3: get_heapless_string(row, "jurisdictions3")?,
            jurisdictions4: get_heapless_string(row, "jurisdictions4")?,
            jurisdictions5: get_heapless_string(row, "jurisdictions5")?,
        })
    }
}

impl TryFromRow<PgRow> for ComplianceMetadataIdxModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(ComplianceMetadataIdxModel {
            id: row.get("compliance_metadata_id"),
            regulatory_code_hash: row.try_get("regulatory_code_hash").ok(),
        })
    }
}