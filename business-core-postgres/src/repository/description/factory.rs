use std::sync::Arc;
use parking_lot::RwLock as ParkingRwLock;
use postgres_unit_of_work::UnitOfWorkSession;
use postgres_index_cache::{CacheNotificationListener, IndexCacheHandler};
use business_core_db::models::description::named::NamedIdxModel;
use super::NamedRepositoryImpl;

/// Factory for creating description module repositories
///
/// This factory holds all caches for the description module and provides
/// methods to build repositories with the appropriate executor.
/// This should be used as a singleton throughout the application.
pub struct DescriptionRepoFactory {
    named_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<NamedIdxModel>>>,
}

impl DescriptionRepoFactory {
    /// Create a new DescriptionRepoFactory singleton
    ///
    /// Optionally register cache handlers with a notification listener
    pub fn new(listener: Option<&mut CacheNotificationListener>) -> Arc<Self> {
        let named_idx_cache = Arc::new(ParkingRwLock::new(
            business_core_db::IdxModelCache::new(vec![]).unwrap()
        ));
        
        // Register handlers with listener if provided
        if let Some(listener) = listener {
            let handler = Arc::new(IndexCacheHandler::new(
                "named_idx".to_string(),
                named_idx_cache.clone(),
            ));
            listener.register_handler(handler);
        }
        
        Arc::new(Self {
            named_idx_cache,
        })
    }

    /// Build a NamedRepository with the given executor
    pub fn build_named_repo(&self, session: &impl UnitOfWorkSession) -> Arc<NamedRepositoryImpl> {
        let repo = Arc::new(NamedRepositoryImpl::new(
            session.executor().clone(),
            self.named_idx_cache.clone(),
        ));
        session.register_transaction_aware(repo.clone());
        repo
    }

    /// Build all description repositories with the given executor
    pub fn build_all_repos(&self, session: &impl UnitOfWorkSession) -> DescriptionRepositories {
        DescriptionRepositories {
            named_repository: self.build_named_repo(session),
        }
    }
}

/// Container for all description module repositories
pub struct DescriptionRepositories {
    pub named_repository: Arc<NamedRepositoryImpl>,
}