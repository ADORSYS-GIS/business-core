//! Test helper module for transaction-based test isolation
//!
//! This module provides utilities for running tests within database transactions
//! that are automatically rolled back, ensuring perfect test isolation without
//! the need for explicit cleanup operations.

use crate::postgres_repositories::{AuditRepositories, PostgresRepositories};
use sqlx::postgres::PgPoolOptions;
use std::sync::Arc;
use std::time::Duration;

/// Test context that provides a transactional database session
/// 
/// This struct holds audit repositories that will be automatically
/// rolled back when dropped, ensuring test isolation.
pub struct TestContext {
    pub audit_repos: AuditRepositories,
}

impl TestContext {
    /// Get the audit repositories from the context
    pub fn audit_repos(&self) -> &AuditRepositories {
        &self.audit_repos
    }
}

/// Setup a test context with a transactional database session
///
/// This function creates a new database connection pool, starts a transaction,
/// and returns a TestContext that will automatically roll back the transaction
/// when dropped.
///
/// # Example
///
/// ```rust
/// #[tokio::test]
/// async fn test_example() -> Result<(), Box<dyn std::error::Error>> {
///     let ctx = setup_test_context().await?;
///     let audit_log_repo = &ctx.audit_repos().audit_log_repository;
///
///     // Perform test operations...
///     // All changes will be rolled back when ctx is dropped
///
///     Ok(())
/// }
/// ```
pub async fn setup_test_context() -> Result<TestContext, Box<dyn std::error::Error + Send + Sync>> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:password@localhost:5432/business_core_db".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(1)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&database_url)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    let repos = PostgresRepositories::new(Arc::new(pool));
    let audit_repos = repos.create_audit_repositories().await;

    Ok(TestContext { audit_repos })
}

/// Setup a shared PostgresRepositories for tests that need to share state
/// 
/// This function is useful for tests that need to set up data in one transaction
/// and then start a new transaction for the actual test. The returned PostgresRepositories
/// can be used to create multiple repository instances.
#[allow(dead_code)]
pub async fn setup_shared_repos() -> Result<PostgresRepositories, Box<dyn std::error::Error + Send + Sync>> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://user:password@localhost:5432/business_core_db".to_string());

    let pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(Duration::from_secs(30))
        .connect(&database_url)
        .await?;

    sqlx::migrate!().run(&pool).await?;

    Ok(PostgresRepositories::new(Arc::new(pool)))
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;
    use business_core_db::models::audit::AuditLogModel;
    use business_core_db::repository::load::LoadRepository;

    #[tokio::test]
    async fn test_transaction_rollback() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // First, create an audit log in a transaction that will be rolled back
        let test_id = Uuid::new_v4();
        {
            let ctx = setup_test_context().await?;
            let audit_log_repo = &ctx.audit_repos().audit_log_repository;
            
            let audit_log = AuditLogModel {
                id: test_id,
                updated_at: Utc::now(),
                updated_by_person_id: Uuid::new_v4(),
            };
            
            audit_log_repo.create(vec![audit_log]).await?;
            
            // Verify it exists within the transaction
            let loaded = audit_log_repo.load(test_id).await?;
            assert!(loaded.is_some());
        } // Transaction is rolled back here when ctx is dropped
        
        // Now verify the audit log doesn't exist in a new transaction
        {
            let ctx = setup_test_context().await?;
            let audit_log_repo = &ctx.audit_repos().audit_log_repository;
            
            // Should not exist because the previous transaction was rolled back
            let loaded = audit_log_repo.load(test_id).await?;
            assert!(loaded.is_none());
        }
        
        Ok(())
    }
}