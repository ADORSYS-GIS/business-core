use std::sync::Arc;
use parking_lot::RwLock as ParkingRwLock;
use postgres_unit_of_work::UnitOfWorkSession;
use postgres_index_cache::{CacheNotificationListener, IndexCacheHandler};
use business_core_db::models::person::{
    country::CountryIdxModel,
    country_subdivision::CountrySubdivisionIdxModel,
};
use super::{CountryRepositoryImpl, CountrySubdivisionRepositoryImpl};

/// Factory for creating person module repositories
/// 
/// This factory holds all caches for the person module and provides
/// methods to build repositories with the appropriate executor.
/// This should be used as a singleton throughout the application.
pub struct PersonRepoFactory {
    country_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<CountryIdxModel>>>,
    country_subdivision_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<CountrySubdivisionIdxModel>>>,
}

impl PersonRepoFactory {
    /// Create a new PersonRepoFactory singleton
    /// 
    /// Optionally register cache handlers with a notification listener
    pub fn new(listener: Option<&mut CacheNotificationListener>) -> Arc<Self> {
        let country_idx_cache = Arc::new(ParkingRwLock::new(
            business_core_db::IdxModelCache::new(vec![]).unwrap()
        ));
        
        let country_subdivision_idx_cache = Arc::new(ParkingRwLock::new(
            business_core_db::IdxModelCache::new(vec![]).unwrap()
        ));
        
        // Register handlers with listener if provided
        if let Some(listener) = listener {
            let handler = Arc::new(IndexCacheHandler::new(
                "country_idx".to_string(),
                country_idx_cache.clone(),
            ));
            listener.register_handler(handler);
            
            let subdivision_handler = Arc::new(IndexCacheHandler::new(
                "country_subdivision_idx".to_string(),
                country_subdivision_idx_cache.clone(),
            ));
            listener.register_handler(subdivision_handler);
        }
        
        Arc::new(Self {
            country_idx_cache,
            country_subdivision_idx_cache,
        })
    }

    /// Build a CountryRepository with the given executor
    pub fn build_country_repo(&self, session: &impl UnitOfWorkSession) -> Arc<CountryRepositoryImpl> {
        let repo = Arc::new(CountryRepositoryImpl::new(
            session.executor().clone(),
            self.country_idx_cache.clone(),
        ));
        session.register_transaction_aware(repo.clone());
        repo
    }

    /// Build a CountrySubdivisionRepository with the given executor
    pub fn build_country_subdivision_repo(&self, session: &impl UnitOfWorkSession) -> Arc<CountrySubdivisionRepositoryImpl> {
        let repo = Arc::new(CountrySubdivisionRepositoryImpl::new(
            session.executor().clone(),
            self.country_subdivision_idx_cache.clone(),
        ));
        session.register_transaction_aware(repo.clone());
        repo
    }

    /// Build all person repositories with the given executor
    pub fn build_all_repos(&self, session: &impl UnitOfWorkSession) -> PersonRepositories {
        PersonRepositories {
            country_repository: self.build_country_repo(session),
            country_subdivision_repository: self.build_country_subdivision_repo(session),
        }
    }
}

/// Container for all person module repositories
pub struct PersonRepositories {
    pub country_repository: Arc<CountryRepositoryImpl>,
    pub country_subdivision_repository: Arc<CountrySubdivisionRepositoryImpl>,
}