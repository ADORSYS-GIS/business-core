use std::sync::Arc;
use parking_lot::RwLock as ParkingRwLock;
use postgres_unit_of_work::UnitOfWorkSession;
use postgres_index_cache::{CacheNotificationListener, IndexCacheHandler};
use business_core_db::models::product::account_gl_mapping::AccountGlMappingIdxModel;
use business_core_db::models::product::fee_type_gl_mapping::FeeTypeGlMappingIdxModel;
use super::account_gl_mapping_repository::AccountGlMappingRepositoryImpl;
use super::fee_type_gl_mapping_repository::FeeTypeGlMappingRepositoryImpl;

/// Factory for creating product module repositories
///
/// This factory holds all caches for the product module and provides
/// methods to build repositories with the appropriate executor.
/// This should be used as a singleton throughout the application.
pub struct ProductRepoFactory {
    account_gl_mapping_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<AccountGlMappingIdxModel>>>,
    fee_type_gl_mapping_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<FeeTypeGlMappingIdxModel>>>,
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
        }
        
        Arc::new(Self {
            account_gl_mapping_idx_cache,
            fee_type_gl_mapping_idx_cache,
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

    /// Build all product repositories with the given executor
    pub fn build_all_repos(&self, session: &impl UnitOfWorkSession) -> ProductRepositories {
        ProductRepositories {
            account_gl_mapping_repository: self.build_account_gl_mapping_repo(session),
            fee_type_gl_mapping_repository: self.build_fee_type_gl_mapping_repo(session),
        }
    }
}

/// Container for all product module repositories
pub struct ProductRepositories {
    pub account_gl_mapping_repository: Arc<AccountGlMappingRepositoryImpl>,
    pub fee_type_gl_mapping_repository: Arc<FeeTypeGlMappingRepositoryImpl>,
}