use std::sync::Arc;
use parking_lot::RwLock as ParkingRwLock;
use postgres_unit_of_work::UnitOfWorkSession;
use postgres_index_cache::{CacheNotificationListener, IndexCacheHandler};
use business_core_db::models::person::{
    country::CountryIdxModel,
    country_subdivision::CountrySubdivisionIdxModel,
    locality::LocalityIdxModel,
    location::LocationIdxModel,
    person::PersonIdxModel,
    entity_reference::EntityReferenceIdxModel,
    risk_summary::RiskSummaryIdxModel,
};
use super::{CountryRepositoryImpl, CountrySubdivisionRepositoryImpl, LocalityRepositoryImpl, LocationRepositoryImpl, PersonRepositoryImpl, EntityReferenceRepositoryImpl, RiskSummaryRepositoryImpl, ActivityLogRepositoryImpl};

/// Factory for creating person module repositories
///
/// This factory holds all caches for the person module and provides
/// methods to build repositories with the appropriate executor.
/// This should be used as a singleton throughout the application.
pub struct PersonRepoFactory {
    country_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<CountryIdxModel>>>,
    country_subdivision_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<CountrySubdivisionIdxModel>>>,
    locality_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<LocalityIdxModel>>>,
    location_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<LocationIdxModel>>>,
    person_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<PersonIdxModel>>>,
    entity_reference_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<EntityReferenceIdxModel>>>,
    risk_summary_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<RiskSummaryIdxModel>>>,
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
        
        let locality_idx_cache = Arc::new(ParkingRwLock::new(
            business_core_db::IdxModelCache::new(vec![]).unwrap()
        ));

        let location_idx_cache = Arc::new(ParkingRwLock::new(
            business_core_db::IdxModelCache::new(vec![]).unwrap()
        ));

        let person_idx_cache = Arc::new(ParkingRwLock::new(
            business_core_db::IdxModelCache::new(vec![]).unwrap()
        ));

        let entity_reference_idx_cache = Arc::new(ParkingRwLock::new(
            business_core_db::IdxModelCache::new(vec![]).unwrap()
        ));

        let risk_summary_idx_cache = Arc::new(ParkingRwLock::new(
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
            
            let locality_handler = Arc::new(IndexCacheHandler::new(
                "locality_idx".to_string(),
                locality_idx_cache.clone(),
            ));
            listener.register_handler(locality_handler);

            let location_handler = Arc::new(IndexCacheHandler::new(
                "location_idx".to_string(),
                location_idx_cache.clone(),
            ));
            listener.register_handler(location_handler);

            let person_handler = Arc::new(IndexCacheHandler::new(
                "person_idx".to_string(),
                person_idx_cache.clone(),
            ));
            listener.register_handler(person_handler);

            let entity_reference_handler = Arc::new(IndexCacheHandler::new(
                "entity_reference_idx".to_string(),
                entity_reference_idx_cache.clone(),
            ));
            listener.register_handler(entity_reference_handler);

            let risk_summary_handler = Arc::new(IndexCacheHandler::new(
                "risk_summary_idx".to_string(),
                risk_summary_idx_cache.clone(),
            ));
            listener.register_handler(risk_summary_handler);
        }
        
        Arc::new(Self {
            country_idx_cache,
            country_subdivision_idx_cache,
            locality_idx_cache,
            location_idx_cache,
            person_idx_cache,
            entity_reference_idx_cache,
            risk_summary_idx_cache,
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

    /// Build a LocalityRepository with the given executor
    pub fn build_locality_repo(&self, session: &impl UnitOfWorkSession) -> Arc<LocalityRepositoryImpl> {
        let repo = Arc::new(LocalityRepositoryImpl::new(
            session.executor().clone(),
            self.locality_idx_cache.clone(),
        ));
        session.register_transaction_aware(repo.clone());
        repo
    }

    /// Build a LocationRepository with the given executor
    pub fn build_location_repo(&self, session: &impl UnitOfWorkSession) -> Arc<LocationRepositoryImpl> {
        let repo = Arc::new(LocationRepositoryImpl::new(
            session.executor().clone(),
            self.location_idx_cache.clone(),
        ));
        session.register_transaction_aware(repo.clone());
        repo
    }

    /// Build a PersonRepository with the given executor
    pub fn build_person_repo(&self, session: &impl UnitOfWorkSession) -> Arc<PersonRepositoryImpl> {
        let repo = Arc::new(PersonRepositoryImpl::new(
            session.executor().clone(),
            self.person_idx_cache.clone(),
        ));
        session.register_transaction_aware(repo.clone());
        repo
    }

    /// Build an EntityReferenceRepository with the given executor
    pub fn build_entity_reference_repo(&self, session: &impl UnitOfWorkSession) -> Arc<EntityReferenceRepositoryImpl> {
        let repo = Arc::new(EntityReferenceRepositoryImpl::new(
            session.executor().clone(),
            self.entity_reference_idx_cache.clone(),
        ));
        session.register_transaction_aware(repo.clone());
        repo
    }

    /// Build a RiskSummaryRepository with the given executor
    pub fn build_risk_summary_repo(&self, session: &impl UnitOfWorkSession) -> Arc<RiskSummaryRepositoryImpl> {
        let repo = Arc::new(RiskSummaryRepositoryImpl::new(
            session.executor().clone(),
            self.risk_summary_idx_cache.clone(),
        ));
        session.register_transaction_aware(repo.clone());
        repo
    }

    /// Build an ActivityLogRepository with the given executor
    pub fn build_activity_log_repo(&self, session: &impl UnitOfWorkSession) -> Arc<ActivityLogRepositoryImpl> {
        let repo = Arc::new(ActivityLogRepositoryImpl::new(
            session.executor().clone(),
        ));
        session.register_transaction_aware(repo.clone());
        repo
    }

    /// Build all person repositories with the given executor
    pub fn build_all_repos(&self, session: &impl UnitOfWorkSession) -> PersonRepositories {
        PersonRepositories {
            country_repository: self.build_country_repo(session),
            country_subdivision_repository: self.build_country_subdivision_repo(session),
            locality_repository: self.build_locality_repo(session),
            location_repository: self.build_location_repo(session),
            person_repository: self.build_person_repo(session),
            entity_reference_repository: self.build_entity_reference_repo(session),
            risk_summary_repository: self.build_risk_summary_repo(session),
            activity_log_repository: self.build_activity_log_repo(session),
        }
    }
}

/// Container for all person module repositories
pub struct PersonRepositories {
    pub country_repository: Arc<CountryRepositoryImpl>,
    pub country_subdivision_repository: Arc<CountrySubdivisionRepositoryImpl>,
    pub locality_repository: Arc<LocalityRepositoryImpl>,
    pub location_repository: Arc<LocationRepositoryImpl>,
    pub person_repository: Arc<PersonRepositoryImpl>,
    pub entity_reference_repository: Arc<EntityReferenceRepositoryImpl>,
    pub risk_summary_repository: Arc<RiskSummaryRepositoryImpl>,
    pub activity_log_repository: Arc<ActivityLogRepositoryImpl>,
}