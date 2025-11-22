use async_trait::async_trait;
use business_core_db::models::product::account_gl_mapping::AccountGlMappingIdxModel;
use business_core_db::repository::find_by_i64::FindByI64;
use std::error::Error;
use crate::repository::product::account_gl_mapping_repository::repo_impl::AccountGlMappingRepositoryImpl;

#[async_trait]
impl FindByI64<AccountGlMappingIdxModel> for AccountGlMappingRepositoryImpl {
    async fn find_by_i64(
        &self,
        hash: i64,
    ) -> Result<Vec<AccountGlMappingIdxModel>, Box<dyn Error + Send + Sync>> {
        let cache = self.account_gl_mapping_idx_cache.read().await;
        let result = cache.get_by_i64("customer_account_code_hash", hash);
        Ok(result)
    }
}