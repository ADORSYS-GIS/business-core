use async_trait::async_trait;
use sqlx::Database;
use uuid::Uuid;

use crate::models::identifiable::Identifiable;

/// Generic repository trait for updating multiple entities in a batch
/// 
/// This trait provides a standard interface for batch updating entities in a data store.
/// Any entity that implements the Identifiable trait can be updated using this trait.
/// All updates are performed within a single transaction for atomicity.
/// Only updates items that have changed (based on hash comparison).
/// 
/// # Type Parameters
/// * `DB` - The database type (must implement sqlx::Database)
/// * `T` - The entity type that must implement Identifiable trait
///
/// # Example
/// ```ignore
/// impl<DB: Database> UpdateBatch<DB, PersonModel> for PersonRepositoryImpl<DB> {
///     async fn update_batch(&self, items: Vec<PersonModel>, audit_log_id: Uuid) -> Result<Vec<PersonModel>, Box<dyn Error + Send + Sync>> {
///         // Implementation
///     }
/// }
/// ```
#[async_trait]
pub trait UpdateBatch<DB: Database, T: Identifiable>: Send + Sync {
    /// Update multiple items in a single transaction
    /// 
    /// # Arguments
    /// * `items` - A vector of entities to update
    /// * `audit_log_id` - The UUID of the audit log for tracking this operation
    /// 
    /// # Returns
    /// * `Ok(Vec<T>)` - A vector of updated entities
    /// * `Err` - An error if the transaction could not be executed
    async fn update_batch(
        &self,
        items: Vec<T>,
        audit_log_id: Uuid,
    ) -> Result<Vec<T>, Box<dyn std::error::Error + Send + Sync>>;
}