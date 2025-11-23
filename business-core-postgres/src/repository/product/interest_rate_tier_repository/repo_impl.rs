use async_trait::async_trait;
use parking_lot::RwLock as ParkingRwLock;
use postgres_index_cache::TransactionAwareIdxModelCache;
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct InterestRateTierRepositoryImpl {
    pub(super) executor: Executor,
    pub(super) interest_rate_tier_idx_cache:
        Arc<RwLock<TransactionAwareIdxModelCache<business_core_db::models::product::interest_rate_tier::InterestRateTierIdxModel>>>,
}

impl InterestRateTierRepositoryImpl {
    pub fn new(
        executor: Executor,
        interest_rate_tier_idx_cache: Arc<
            ParkingRwLock<
                business_core_db::IdxModelCache<
                    business_core_db::models::product::interest_rate_tier::InterestRateTierIdxModel,
                >,
            >,
        >,
    ) -> Self {
        Self {
            executor,
            interest_rate_tier_idx_cache: Arc::new(RwLock::new(
                TransactionAwareIdxModelCache::new(interest_rate_tier_idx_cache),
            )),
        }
    }
}

#[async_trait]
impl TransactionAware for InterestRateTierRepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        self.interest_rate_tier_idx_cache
            .read()
            .await
            .on_commit()
            .await
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        self.interest_rate_tier_idx_cache
            .read()
            .await
            .on_rollback()
            .await
    }
}