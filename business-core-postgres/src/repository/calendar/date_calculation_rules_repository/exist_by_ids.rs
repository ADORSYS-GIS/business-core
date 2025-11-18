use business_core_db::models::calendar::date_calculation_rules::DateCalculationRulesModel;
use business_core_db::repository::exist_by_ids::ExistByIds;
use super::repo_impl::DateCalculationRulesRepositoryImpl;
use async_trait::async_trait;
use std::error::Error;
use uuid::Uuid;
use sqlx::Postgres;

#[async_trait]
impl ExistByIds<sqlx::Postgres> for DateCalculationRulesRepositoryImpl {
    async fn exist_by_ids(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        Self::exist_by_ids_impl(self, ids).await
    }
}

impl DateCalculationRulesRepositoryImpl {
    pub(super) async fn exist_by_ids_impl(
        repo: &DateCalculationRulesRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        // Use index cache to check existence
        let idx_cache = repo.date_calculation_rules_idx_cache.read().await;
        let mut result = Vec::new();
        for &id in ids {
            result.push((id, idx_cache.contains_primary(&id)));
        }

        Ok(result)
    }
}