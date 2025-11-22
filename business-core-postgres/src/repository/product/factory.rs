use std::sync::Arc;
use parking_lot::RwLock as ParkingRwLock;
use postgres_unit_of_work::UnitOfWorkSession;
use postgres_index_cache::{CacheNotificationListener, IndexCacheHandler, MainModelCacheHandler, MainModelCache, CacheConfig, EvictionPolicy};
use std::time::Duration;
use business_core_db::models::product::account_gl_mapping::{AccountGlMappingIdxModel, AccountGlMappingModel};
use super::account_gl_mapping_repository::AccountGlMappingRepositoryImpl;

/// Factory for creating product module repositories with audit and main cache
pub struct ProductRepoFactory {
    account_gl_mapping_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<AccountGlMappingIdxModel>>>,
    account_gl_mapping_cache: Arc<ParkingRwLock<MainModelCache<AccountGlMappingModel>>>,
}

impl ProductRepoFactory {
    /// Create a new ProductRepoFactory singleton with cache configuration
    ///
    /// Optionally register cache handlers with a notification listener
    pub fn new(listener: Option<&mut CacheNotificationListener>) -> Arc<Self> {
        // Initialize index cache
        let account_gl_mapping_idx_cache = Arc::new(ParkingRwLock::new(
            business_core_db::IdxModelCache::new(vec![]).unwrap()
        ));
        
        // Initialize main cache with configuration
        let cache_config = CacheConfig::new(
            1000,  // Max 1000 entities in cache
            EvictionPolicy::LRU,  // Least Recently Used eviction
        )
        .with_ttl(Duration::from_secs(3600)); // 1 hour TTL
        
        let account_gl_mapping_cache = Arc::new(ParkingRwLock::new(
            MainModelCache::new(cache_config)
        ));
        
        // Register handlers with listener if provided
        if let Some(listener) = listener {
            // Register index cache handler
            let idx_handler = Arc::new(IndexCacheHandler::new(
                "account_gl_mapping_idx".to_string(),
                account_gl_mapping_idx_cache.clone(),
            ));
            listener.register_handler(idx_handler);
            
            // Register main cache handler
            let main_handler = Arc::new(MainModelCacheHandler::new(
                "account_gl_mapping".to_string(),
                account_gl_mapping_cache.clone(),
            ));
            listener.register_handler(main_handler);
        }
        
        Arc::new(Self {
            account_gl_mapping_idx_cache,
            account_gl_mapping_cache,
        })
    }

    /// Build a AccountGlMappingRepository with the given executor
    pub fn build_account_gl_mapping_repo(&self, session: &impl UnitOfWorkSession) -> Arc<AccountGlMappingRepositoryImpl> {
        let repo = Arc::new(AccountGlMappingRepositoryImpl::new(
            session.executor().clone(),
            self.account_gl_mapping_idx_cache.clone(),
            self.account_gl_mapping_cache.clone(),
        ));
        session.register_transaction_aware(repo.clone());
        repo
    }

    /// Build all product repositories with the given executor
    pub fn build_all_repos(&self, session: &impl UnitOfWorkSession) -> ProductRepositories {
        ProductRepositories {
            account_gl_mapping_repository: self.build_account_gl_mapping_repo(session),
        }
    }
}

/// Container for all product module repositories
pub struct ProductRepositories {
    pub account_gl_mapping_repository: Arc<AccountGlMappingRepositoryImpl>,
}