use sqlx::PgPool;
use std::sync::Arc;
use postgres_unit_of_work::Executor;
use postgres_index_cache::{CacheNotificationListener, IndexCacheHandler};

use crate::repository::audit::audit_log_repository::AuditLogRepositoryImpl;

pub struct PostgresRepositories {
    pool: Arc<PgPool>,
}

impl PostgresRepositories {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self { pool }
    }

    /// Create all repositories sharing a single transaction
    pub async fn create_all_repositories(&self, listener: Option<&mut CacheNotificationListener>) -> (AuditRepositories, PersonRepositories) {
        let tx = self
            .pool
            .begin()
            .await
            .expect("Failed to begin transaction");
        let executor = Executor::new(tx);

        // Create audit repositories with shared executor
        let audit_log_repository = Arc::new(AuditLogRepositoryImpl::new(executor.clone()));
        let audit_repos = AuditRepositories {
            audit_log_repository,
        };

        // Create person repositories with shared executor
        let country_idx_cache = Arc::new(parking_lot::RwLock::new(
            business_core_db::IdxModelCache::new(vec![]).unwrap()
        ));
        
        let country_subdivision_idx_cache = Arc::new(parking_lot::RwLock::new(
            business_core_db::IdxModelCache::new(vec![]).unwrap()
        ));
        
        // Register handler with listener if provided
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
        
        let country_repository = Arc::new(crate::repository::person::CountryRepositoryImpl::new(
            executor.clone(),
            country_idx_cache,
        ));
        
        let country_subdivision_repository = Arc::new(crate::repository::person::CountrySubdivisionRepositoryImpl::new(
            executor.clone(),
            country_subdivision_idx_cache,
        ));
        
        let person_repos = PersonRepositories {
            country_repository,
            country_subdivision_repository,
        };

        (audit_repos, person_repos)
    }
}

pub struct AuditRepositories {
    pub audit_log_repository: Arc<AuditLogRepositoryImpl>,
}

pub struct PersonRepositories {
    pub country_repository: Arc<crate::repository::person::CountryRepositoryImpl>,
    pub country_subdivision_repository: Arc<crate::repository::person::CountrySubdivisionRepositoryImpl>,
}