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

    pub async fn create_audit_repositories(&self) -> AuditRepositories {
        let tx = self
            .pool
            .begin()
            .await
            .expect("Failed to begin transaction");
        let executor = Executor::new(tx);

        let audit_log_repository = Arc::new(AuditLogRepositoryImpl::new(executor.clone()));

        AuditRepositories {
            audit_log_repository,
        }
    }

    pub async fn create_person_repositories(&self, listener: Option<&mut CacheNotificationListener>) -> PersonRepositories {
        let tx = self
            .pool
            .begin()
            .await
            .expect("Failed to begin transaction");
        let executor = Executor::new(tx);

        let country_idx_cache = Arc::new(parking_lot::RwLock::new(
            business_core_db::IdxModelCache::new(vec![]).unwrap()
        ));
        
        // Register handler with listener if provided
        if let Some(listener) = listener {
            let handler = Arc::new(IndexCacheHandler::new(
                "country_idx".to_string(),
                country_idx_cache.clone(),
            ));
            listener.register_handler(handler);
        }
        
        let country_repository = Arc::new(crate::repository::person::CountryRepositoryImpl::new(
            executor.clone(),
            country_idx_cache,
        ));

        PersonRepositories {
            country_repository,
        }
    }
}

pub struct AuditRepositories {
    pub audit_log_repository: Arc<AuditLogRepositoryImpl>,
}

pub struct PersonRepositories {
    pub country_repository: Arc<crate::repository::person::CountryRepositoryImpl>,
}