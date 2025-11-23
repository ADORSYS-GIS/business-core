use std::sync::Arc;
use parking_lot::RwLock as ParkingRwLock;
use postgres_unit_of_work::UnitOfWorkSession;
use postgres_index_cache::{CacheNotificationListener, IndexCacheHandler};
use business_core_db::models::product::account_gl_mapping::AccountGlMappingIdxModel;
use business_core_db::models::product::fee_type_gl_mapping::FeeTypeGlMappingIdxModel;
use business_core_db::models::product::interest_rate_tier::InterestRateTierIdxModel;
use business_core_db::models::product::product::ProductIdxModel;
use super::account_gl_mapping_repository::AccountGlMappingRepositoryImpl;
use super::fee_type_gl_mapping_repository::FeeTypeGlMappingRepositoryImpl;
use super::interest_rate_tier_repository::InterestRateTierRepositoryImpl;
use super::product_repository::ProductRepositoryImpl;

/// Factory for creating product module repositories
///
/// This factory holds all caches for the product module and provides
/// methods to build repositories with the appropriate executor.
/// This should be used as a singleton throughout the application.
pub struct ProductRepoFactory {
    account_gl_mapping_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<AccountGlMappingIdxModel>>>,
    fee_type_gl_mapping_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<FeeTypeGlMappingIdxModel>>>,
    interest_rate_tier_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<InterestRateTierIdxModel>>>,
    product_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<ProductIdxModel>>>,
}

impl ProductRepoFactory {
    /// Create a new ProductRepoFactory singleton
    ///
    /// Optionally register cache handlers with a notification listener
    pub fn new(listener: Option<&mut CacheNotificationListener>) -> Arc<Self> {
        let account_gl_mapping_idx_cache = Arc::new(ParkingRwLock::new(
            business_core_db::IdxModelCache::new(vec![]).unwrap()
        ));
        
        let fee_type_gl_mapping_idx_cache = Arc::new(ParkingRwLock::new(
            business_core_db::IdxModelCache::new(vec![]).unwrap()
        ));

        let interest_rate_tier_idx_cache = Arc::new(ParkingRwLock::new(
            business_core_db::IdxModelCache::new(vec![]).unwrap()
        ));

        let product_idx_cache = Arc::new(ParkingRwLock::new(
            business_core_db::IdxModelCache::new(vec![]).unwrap()
        ));
        
        // Register handlers with listener if provided
        if let Some(listener) = listener {
            let account_handler = Arc::new(IndexCacheHandler::new(
                "account_gl_mapping_idx".to_string(),
                account_gl_mapping_idx_cache.clone(),
            ));
            listener.register_handler(account_handler);
            
            let fee_type_handler = Arc::new(IndexCacheHandler::new(
                "fee_type_gl_mapping_idx".to_string(),
                fee_type_gl_mapping_idx_cache.clone(),
            ));
            listener.register_handler(fee_type_handler);

            let interest_rate_tier_handler = Arc::new(IndexCacheHandler::new(
                "interest_rate_tier_idx".to_string(),
                interest_rate_tier_idx_cache.clone(),
            ));
            listener.register_handler(interest_rate_tier_handler);

            let product_handler = Arc::new(IndexCacheHandler::new(
                "product_idx".to_string(),
                product_idx_cache.clone(),
            ));
            listener.register_handler(product_handler);
        }
        
        Arc::new(Self {
            account_gl_mapping_idx_cache,
            fee_type_gl_mapping_idx_cache,
            interest_rate_tier_idx_cache,
            product_idx_cache,
        })
    }

    /// Build an AccountGlMappingRepository with the given executor
    pub fn build_account_gl_mapping_repo(&self, session: &impl UnitOfWorkSession) -> Arc<AccountGlMappingRepositoryImpl> {
        let repo = Arc::new(AccountGlMappingRepositoryImpl::new(
            session.executor().clone(),
            self.account_gl_mapping_idx_cache.clone(),
        ));
        session.register_transaction_aware(repo.clone());
        repo
    }

    /// Build a FeeTypeGlMappingRepository with the given executor
    pub fn build_fee_type_gl_mapping_repo(&self, session: &impl UnitOfWorkSession) -> Arc<FeeTypeGlMappingRepositoryImpl> {
        let repo = Arc::new(FeeTypeGlMappingRepositoryImpl::new(
            session.executor().clone(),
            self.fee_type_gl_mapping_idx_cache.clone(),
        ));
        session.register_transaction_aware(repo.clone());
        repo
    }

    /// Build a InterestRateTierRepository with the given executor
    pub fn build_interest_rate_tier_repo(&self, session: &impl UnitOfWorkSession) -> Arc<InterestRateTierRepositoryImpl> {
        let repo = Arc::new(InterestRateTierRepositoryImpl::new(
            session.executor().clone(),
            self.interest_rate_tier_idx_cache.clone(),
        ));
        session.register_transaction_aware(repo.clone());
        repo
    }

    /// Build a ProductRepository with the given executor
    pub fn build_product_repo(&self, session: &impl UnitOfWorkSession) -> Arc<ProductRepositoryImpl> {
        let repo = Arc::new(ProductRepositoryImpl::new(
            session.executor().clone(),
            self.product_idx_cache.clone(),
        ));
        session.register_transaction_aware(repo.clone());
        repo
    }

    /// Build all product repositories with the given executor
    pub fn build_all_repos(&self, session: &impl UnitOfWorkSession) -> ProductRepositories {
        ProductRepositories {
            account_gl_mapping_repository: self.build_account_gl_mapping_repo(session),
            fee_type_gl_mapping_repository: self.build_fee_type_gl_mapping_repo(session),
            interest_rate_tier_repository: self.build_interest_rate_tier_repo(session),
            product_repository: self.build_product_repo(session),
        }
    }
}

/// Container for all product module repositories
pub struct ProductRepositories {
    pub account_gl_mapping_repository: Arc<AccountGlMappingRepositoryImpl>,
    pub fee_type_gl_mapping_repository: Arc<FeeTypeGlMappingRepositoryImpl>,
    pub interest_rate_tier_repository: Arc<InterestRateTierRepositoryImpl>,
    pub product_repository: Arc<ProductRepositoryImpl>,
}