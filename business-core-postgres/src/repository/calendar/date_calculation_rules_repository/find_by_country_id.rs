use std::error::Error;

use business_core_db::models::calendar::date_calculation_rules::DateCalculationRulesIdxModel;
use uuid::Uuid;

use super::repo_impl::DateCalculationRulesRepositoryImpl;

impl DateCalculationRulesRepositoryImpl {
    pub async fn find_by_country_id(
        &self,
        country_id: Uuid,
    ) -> Result<Vec<DateCalculationRulesIdxModel>, Box<dyn Error + Send + Sync>> {
        let cache = self.date_calculation_rules_idx_cache.read().await;
        let items = cache.get_by_uuid_index("country_id", &country_id);
        Ok(items)
    }
}