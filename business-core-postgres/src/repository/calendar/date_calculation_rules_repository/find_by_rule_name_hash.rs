use std::error::Error;

use business_core_db::models::calendar::date_calculation_rules::DateCalculationRulesIdxModel;

use super::repo_impl::DateCalculationRulesRepositoryImpl;

impl DateCalculationRulesRepositoryImpl {
    pub async fn find_by_rule_name_hash(
        &self,
        rule_name_hash: i64,
    ) -> Result<Vec<DateCalculationRulesIdxModel>, Box<dyn Error + Send + Sync>> {
        let cache = self.date_calculation_rules_idx_cache.read().await;
        let items = cache.get_by_i64_index("rule_name_hash", &rule_name_hash);
        Ok(items)
    }
}