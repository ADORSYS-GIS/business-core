use sqlx::PgPool;
use std::sync::Arc;
use postgres_unit_of_work::Executor;

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
}

pub struct AuditRepositories {
    pub audit_log_repository: Arc<AuditLogRepositoryImpl>,
}