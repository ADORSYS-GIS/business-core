use std::sync::Arc;
use parking_lot::RwLock as ParkingRwLock;
use postgres_unit_of_work::UnitOfWorkSession;
use postgres_index_cache::{CacheNotificationListener, IndexCacheHandler, MainModelCacheHandler, MainModelCache, CacheConfig, EvictionPolicy};
use std::time::Duration;
use business_core_db::models::calendar::weekend_days::{WeekendDaysIdxModel, WeekendDaysModel};
use business_core_db::models::calendar::business_day::{BusinessDayIdxModel, BusinessDayModel};
use super::{WeekendDaysRepositoryImpl, BusinessDayRepositoryImpl};

/// Factory for creating calendar module repositories with main cache
pub struct CalendarRepoFactory {
    weekend_days_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<WeekendDaysIdxModel>>>,
    weekend_days_cache: Arc<ParkingRwLock<MainModelCache<WeekendDaysModel>>>,
    business_day_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<BusinessDayIdxModel>>>,
    business_day_cache: Arc<ParkingRwLock<MainModelCache<BusinessDayModel>>>,
}

impl CalendarRepoFactory {
    /// Create a new CalendarRepoFactory singleton with cache configuration
    ///
    /// Optionally register cache handlers with a notification listener
    pub fn new(listener: Option<&mut CacheNotificationListener>) -> Arc<Self> {
        // Initialize index cache
        let weekend_days_idx_cache = Arc::new(ParkingRwLock::new(
            business_core_db::IdxModelCache::new(vec![]).unwrap()
        ));
        
        // Initialize main cache with configuration
        let cache_config = CacheConfig::new(
            1000,  // Max 1000 entities in cache
            EvictionPolicy::LRU,  // Least Recently Used eviction
        )
        .with_ttl(Duration::from_secs(3600)); // 1 hour TTL
        
        let weekend_days_cache = Arc::new(ParkingRwLock::new(
            MainModelCache::new(cache_config)
        ));
        
        // Initialize business_day index cache
        let business_day_idx_cache = Arc::new(ParkingRwLock::new(
            business_core_db::IdxModelCache::new(vec![]).unwrap()
        ));
        
        // Initialize business_day main cache with configuration
        let business_day_cache_config = CacheConfig::new(
            1000,  // Max 1000 entities in cache
            EvictionPolicy::LRU,  // Least Recently Used eviction
        )
        .with_ttl(Duration::from_secs(3600)); // 1 hour TTL
        
        let business_day_cache = Arc::new(ParkingRwLock::new(
            MainModelCache::new(business_day_cache_config)
        ));
        
        // Register handlers with listener if provided
        if let Some(listener) = listener {
            // Register weekend_days index cache handler
            let idx_handler = Arc::new(IndexCacheHandler::new(
                "calendar_weekend_days_idx".to_string(),
                weekend_days_idx_cache.clone(),
            ));
            listener.register_handler(idx_handler);
            
            // Register weekend_days main cache handler
            let main_handler = Arc::new(MainModelCacheHandler::new(
                "calendar_weekend_days".to_string(),  // Note: main table name, not _idx
                weekend_days_cache.clone(),
            ));
            listener.register_handler(main_handler);
            
            // Register business_day index cache handler
            let business_day_idx_handler = Arc::new(IndexCacheHandler::new(
                "calendar_business_day_idx".to_string(),
                business_day_idx_cache.clone(),
            ));
            listener.register_handler(business_day_idx_handler);
            
            // Register business_day main cache handler
            let business_day_main_handler = Arc::new(MainModelCacheHandler::new(
                "calendar_business_day".to_string(),  // Note: main table name, not _idx
                business_day_cache.clone(),
            ));
            listener.register_handler(business_day_main_handler);
        }
        
        Arc::new(Self {
            weekend_days_idx_cache,
            weekend_days_cache,
            business_day_idx_cache,
            business_day_cache,
        })
    }

    /// Build a WeekendDaysRepository with the given executor
    pub fn build_weekend_days_repo(&self, session: &impl UnitOfWorkSession) -> Arc<WeekendDaysRepositoryImpl> {
        let repo = Arc::new(WeekendDaysRepositoryImpl::new(
            session.executor().clone(),
            self.weekend_days_idx_cache.clone(),
            self.weekend_days_cache.clone(),
        ));
        session.register_transaction_aware(repo.clone());
        repo
    }

    /// Build a BusinessDayRepository with the given executor
    pub fn build_business_day_repo(&self, session: &impl UnitOfWorkSession) -> Arc<BusinessDayRepositoryImpl> {
        let repo = Arc::new(BusinessDayRepositoryImpl::new(
            session.executor().clone(),
            self.business_day_idx_cache.clone(),
            self.business_day_cache.clone(),
        ));
        session.register_transaction_aware(repo.clone());
        repo
    }

    /// Build all calendar repositories with the given executor
    pub fn build_all_repos(&self, session: &impl UnitOfWorkSession) -> CalendarRepositories {
        CalendarRepositories {
            weekend_days_repository: self.build_weekend_days_repo(session),
            business_day_repository: self.build_business_day_repo(session),
        }
    }
}

/// Container for all calendar module repositories
pub struct CalendarRepositories {
    pub weekend_days_repository: Arc<WeekendDaysRepositoryImpl>,
    pub business_day_repository: Arc<BusinessDayRepositoryImpl>,
}