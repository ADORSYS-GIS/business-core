use async_trait::async_trait;
use sqlx::Database;
use uuid::Uuid;

/// Generic repository trait for deleting multiple entities in a batch
/// 
/// This trait provides a standard interface for batch deleting entities from a data store.
/// All deletes are performed within a single transaction for atomicity.
/// Returns the number of items successfully deleted.
/// 
/// # Type Parameters
/// * `DB` - The database type (must implement sqlx::Database)
///
/// # Example
/// ```ignore
/// impl<DB: Database> DeleteBatch<DB> for PersonRepositoryImpl<DB> {
///     async fn delete_batch(&self, ids: &[Uuid], audit_log_id: Uuid) -> Result<usize, Box<dyn Error + Send + Sync>> {
///         // Implementation
///     }
/// }
/// ```
#[async_trait]
pub trait DeleteBatch<DB: Database>: Send + Sync {
    /// Delete multiple items by their IDs in a single transaction
    /// 
    /// # Arguments
    /// * `ids` - A slice of UUIDs of the entities to delete
    /// * `audit_log_id` - The optional UUID of the audit log for tracking this operation
    ///
    /// # Returns
    /// * `Ok(usize)` - The number of items successfully deleted
    /// * `Err` - An error if the transaction could not be executed
    async fn delete_batch(
        &self,
        ids: &[Uuid],
        audit_log_id: Option<Uuid>,
    ) -> Result<usize, Box<dyn std::error::Error + Send + Sync>>;
}