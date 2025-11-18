use business_core_db::models::calendar::date_calculation_rules::{DateCalculationRulesIdxModel, DateCalculationRulesModel, DateRulePurpose, DateShiftRule};
use crate::utils::{get_heapless_string, get_optional_heapless_string, TryFromRow};
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use postgres_index_cache::{TransactionAwareIdxModelCache, TransactionAwareMainModelCache};
use parking_lot::RwLock as ParkingRwLock;
use tokio::sync::RwLock;
use std::sync::Arc;
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use async_trait::async_trait;

pub struct DateCalculationRulesRepositoryImpl {
    pub executor: Executor,
    pub date_calculation_rules_idx_cache: Arc<RwLock<TransactionAwareIdxModelCache<DateCalculationRulesIdxModel>>>,
    pub date_calculation_rules_cache: Arc<RwLock<TransactionAwareMainModelCache<DateCalculationRulesModel>>>,
}

impl DateCalculationRulesRepositoryImpl {
    pub fn new(
        executor: Executor,
        date_calculation_rules_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<DateCalculationRulesIdxModel>>>,
        date_calculation_rules_cache: Arc<ParkingRwLock<postgres_index_cache::MainModelCache<DateCalculationRulesModel>>>,
    ) -> Self {
        Self {
            executor,
            date_calculation_rules_idx_cache: Arc::new(RwLock::new(TransactionAwareIdxModelCache::new(
                date_calculation_rules_idx_cache,
            ))),
            date_calculation_rules_cache: Arc::new(RwLock::new(TransactionAwareMainModelCache::new(
                date_calculation_rules_cache,
            ))),
        }
    }

    pub async fn load_all_date_calculation_rules_idx(
        executor: &Executor,
    ) -> Result<Vec<DateCalculationRulesIdxModel>, sqlx::Error> {
        let query = sqlx::query("SELECT * FROM calendar_date_calculation_rules_idx");
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
            idx_models.push(DateCalculationRulesIdxModel::try_from_row(&row).map_err(sqlx::Error::Decode)?);
        }
        Ok(idx_models)
    }
}

#[async_trait]
impl TransactionAware for DateCalculationRulesRepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        self.date_calculation_rules_idx_cache.read().await.on_commit().await?;
        self.date_calculation_rules_cache.read().await.on_commit().await?;
        Ok(())
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        self.date_calculation_rules_idx_cache.read().await.on_rollback().await?;
        self.date_calculation_rules_cache.read().await.on_rollback().await?;
        Ok(())
    }
}

impl TryFromRow<PgRow> for DateCalculationRulesModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(DateCalculationRulesModel {
            id: row.get("id"),
            country_id: row.get("country_id"),
            country_subdivision_id: row.get("country_subdivision_id"),
            rule_name: get_heapless_string(row, "rule_name")?,
            rule_purpose: row.get("rule_purpose"),
            default_shift_rule: row.get("default_shift_rule"),
            weekend_days_id: row.get("weekend_days_id"),
            priority: row.get("priority"),
            is_active: row.get("is_active"),
            effective_date: row.get("effective_date"),
            expiry_date: row.get("expiry_date"),
        })
    }
}

impl TryFromRow<PgRow> for DateCalculationRulesIdxModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(DateCalculationRulesIdxModel {
            id: row.get("id"),
            country_id: row.get("country_id"),
            country_subdivision_id: row.get("country_subdivision_id"),
            rule_name_hash: row.get("rule_name_hash"),
        })
    }
}