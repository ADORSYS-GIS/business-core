//! Test helper module for transaction-based test isolation
//!
//! This module provides utilities for running tests within database transactions
//! that are automatically rolled back, ensuring perfect test isolation without
//! the need for explicit cleanup operations.

use sqlx::{PgPool, postgres::PgPoolOptions};
use std::sync::Arc;
use std::time::Duration;
use postgres_index_cache::CacheNotificationListener;
use postgres_unit_of_work::{PostgresUnitOfWork, UnitOfWork};
use tokio::sync::OnceCell;

use crate::repository::{audit::AuditRepositories, person::PersonRepositories, reason_and_purpose::ReasonAndPurposeRepositories, calendar::CalendarRepositories, description::DescriptionRepositories, product::ProductRepositories};

// Flag to track if DB initialization has been done
static DB_INITIALIZED: OnceCell<()> = OnceCell::const_new();

/// Initialize database and get a test pool with specified number of connections
///
/// This function uses a special 2-connection pool for DB initialization (only on first call),
/// then returns a new pool with the specified number of connections for each invocation.
async fn get_or_init_test_pool_with_size(max_connections: u32) -> Result<Arc<PgPool>, Box<dyn std::error::Error + Send + Sync>> {
    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5433/business_core_db".to_string());

    // Initialize DB only on first call using a dedicated 2-connection pool
    DB_INITIALIZED.get_or_try_init(|| async {
        let init_pool = PgPoolOptions::new()
            .max_connections(2)
            .min_connections(1)
            .acquire_timeout(Duration::from_secs(10))
            .connect(&database_url)
            .await?;

        // Init DB
        crate::repository::db_init::cleanup_database(&init_pool).await?;
        postgres_index_cache::cleanup_cache_triggers(&init_pool).await?;
        postgres_index_cache::init_cache_triggers(&init_pool).await?;
        crate::repository::db_init::init_database(&init_pool).await?;

        init_pool.close().await;
        Ok::<(), Box<dyn std::error::Error + Send + Sync>>(())
    }).await?;

    // Create and return a new pool with specified connections for each call
    let pool = PgPoolOptions::new()
        .max_connections(max_connections)
        .min_connections(1)
        .acquire_timeout(Duration::from_secs(10))
        .idle_timeout(Some(Duration::from_secs(20)))
        .max_lifetime(Some(Duration::from_secs(1800)))
        .after_connect(|conn, _meta| {
            Box::pin(async move {
                // Set statement timeout to prevent hung queries
                sqlx::query("SET statement_timeout = '5s'")
                    .execute(conn)
                    .await?;
                Ok(())
            })
        })
        .connect(&database_url)
        .await?;

    Ok(Arc::new(pool))
}

/// Initialize database and get a test pool with 2 connections
///
/// This function uses a special 2-connection pool for DB initialization (only on first call),
/// then returns a new 2-connection pool for each invocation.
async fn get_or_init_test_pool() -> Result<Arc<PgPool>, Box<dyn std::error::Error + Send + Sync>> {
    get_or_init_test_pool_with_size(2).await
}

/// Test context that provides a transactional database session
///
/// This struct holds audit and person repositories that will be automatically
/// rolled back when dropped, ensuring test isolation.
pub struct TestContext {
    pub audit_repos: AuditRepositories,
    pub person_repos: PersonRepositories,
    pub reason_and_purpose_repos: ReasonAndPurposeRepositories,
    pub calendar_repos: CalendarRepositories,
    pub description_repos: DescriptionRepositories,
    pub product_repos: ProductRepositories,
    pub pool: Arc<PgPool>,
    listener_handle: Option<tokio::task::JoinHandle<()>>,
}

impl TestContext {
    /// Get the audit repositories from the context
    pub fn audit_repos(&self) -> &AuditRepositories {
        &self.audit_repos
    }

    /// Get the person repositories from the context
    pub fn person_repos(&self) -> &PersonRepositories {
        &self.person_repos
    }

    /// Get the reason_and_purpose repositories from the context
    pub fn reason_and_purpose_repos(&self) -> &ReasonAndPurposeRepositories {
        &self.reason_and_purpose_repos
    }

    /// Get the calendar repositories from the context
    pub fn calendar_repos(&self) -> &CalendarRepositories {
        &self.calendar_repos
    }

    /// Get the description repositories from the context
    pub fn description_repos(&self) -> &DescriptionRepositories {
        &self.description_repos
    }

    /// Get the product repositories from the context
    pub fn product_repos(&self) -> &ProductRepositories {
        &self.product_repos
    }

    /// Get the pool from the context
    pub fn pool(&self) -> &Arc<PgPool> {
        &self.pool
    }
}
impl Drop for TestContext {
    fn drop(&mut self) {
        if let Some(handle) = self.listener_handle.take() {
            handle.abort();
        }
    }
}

/// Setup a test context with a transactional database session (without listener)
///
/// This function creates a new database connection pool, starts a transaction,
/// and returns a TestContext that will automatically roll back the transaction
/// when dropped. This version does NOT start the cache notification listener,
/// making it suitable for standard tests that don't need cache synchronization.
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
    let pool = get_or_init_test_pool().await?;
    
    // Create a unit of work and begin a transaction session
    let uow = PostgresUnitOfWork::new(pool.clone());
    let session = uow.begin().await?;
    
    // Create factories with listener for cache synchronization
    let audit_factory = crate::repository::audit::AuditRepoFactory::new();
    let person_factory = crate::repository::person::PersonRepoFactory::new(None);
    let reason_and_purpose_factory = crate::repository::reason_and_purpose::ReasonAndPurposeRepoFactory::new(None);
    let calendar_factory = crate::repository::calendar::CalendarRepoFactory::new(None);
    let description_factory = crate::repository::description::DescriptionRepoFactory::new(None);
    let product_factory = crate::repository::product::ProductRepoFactory::new(None);
    
    // Build repositories using the session executor
    let audit_repos = audit_factory.build_all_repos(&session);
    let person_repos = person_factory.build_all_repos(&session);
    let reason_and_purpose_repos = reason_and_purpose_factory.build_all_repos(&session);
    let calendar_repos = calendar_factory.build_all_repos(&session);
    let description_repos = description_factory.build_all_repos(&session);
    let product_repos = product_factory.build_all_repos(&session);

    Ok(TestContext {
        audit_repos,
        person_repos,
        reason_and_purpose_repos,
        calendar_repos,
        description_repos,
        product_repos,
        pool,
        listener_handle: None,
    })
}

/// Setup a test context with a transactional database session AND start the cache notification listener
///
/// This function uses the shared test pool, starts a transaction,
/// and returns a TestContext that will automatically roll back the transaction
/// when dropped. This version DOES start the cache notification listener in the background,
/// making it suitable for tests that need to verify cache synchronization behavior.
///
pub async fn setup_test_context_and_listen() -> Result<TestContext, Box<dyn std::error::Error + Send + Sync>> {
    // Use 4 connections: 1 for transaction, 1 for listener, 2 for raw queries
    let pool = get_or_init_test_pool_with_size(10).await?;
    
    // Create a unit of work and begin a transaction session
    let uow = PostgresUnitOfWork::new(pool.clone());
    let session = uow.begin().await?;
        
    // Create listener for cache notifications
    let mut listener = CacheNotificationListener::new();
    
    // Create factories with listener for cache synchronization
    let audit_factory = crate::repository::audit::AuditRepoFactory::new();
    let person_factory = crate::repository::person::PersonRepoFactory::new(Some(&mut listener));
    let reason_and_purpose_factory = crate::repository::reason_and_purpose::ReasonAndPurposeRepoFactory::new(Some(&mut listener));
    let calendar_factory = crate::repository::calendar::CalendarRepoFactory::new(Some(&mut listener));
    let description_factory = crate::repository::description::DescriptionRepoFactory::new(Some(&mut listener));
    let product_factory = crate::repository::product::ProductRepoFactory::new(Some(&mut listener));
    
    // Build repositories using the session executor
    let audit_repos = audit_factory.build_all_repos(&session);
    let person_repos = person_factory.build_all_repos(&session);
    let reason_and_purpose_repos = reason_and_purpose_factory.build_all_repos(&session);
    let calendar_repos = calendar_factory.build_all_repos(&session);
    let description_repos = description_factory.build_all_repos(&session);
    let product_repos = product_factory.build_all_repos(&session);
    
    // Start listening to notifications in background
    let pool_clone = pool.clone();
    let listen_handle = tokio::spawn(async move {
        // The listener will run until aborted
        let _ = listener.listen(&pool_clone).await;
    });

    Ok(TestContext {
        audit_repos,
        person_repos,
        reason_and_purpose_repos,
        calendar_repos,
        description_repos,
        product_repos,
        pool,
        listener_handle: Some(listen_handle),
    })
}
use rand::{distributions::Alphanumeric, Rng};

pub fn random(n: usize) -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(n)
        .map(char::from)
        .collect()
}


#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;
    use business_core_db::models::audit::AuditLogModel;
    use business_core_db::repository::load::Load;

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
            
            audit_log_repo.create(&audit_log).await?;
            
            // Verify it exists within the transaction
            match audit_log_repo.load(test_id).await {
                Ok(loaded) => {
                    assert_eq!(loaded.id, test_id);
                }
                Err(e) => {
                    panic!("Expected audit log to exist within transaction, but got error: {e}");
                }
            }
        } // Transaction is rolled back here when ctx is dropped
        
        // Now verify the audit log doesn't exist in a new transaction
        {
            let ctx = setup_test_context().await?;
            let audit_log_repo = &ctx.audit_repos().audit_log_repository;
            
            // Should not exist because the previous transaction was rolled back
            match audit_log_repo.load(test_id).await {
                Ok(_) => {
                    panic!("Expected audit log to not exist after rollback, but it was found");
                }
                Err(_) => {
                    // Expected: entity should not be found
                }
            }
        }
        
        Ok(())
    }
}