use async_trait::async_trait;
use sqlx::Database;
use uuid::Uuid;

/// Generic repository trait for checking existence of multiple entities by their IDs
/// 
/// This trait provides a standard interface for checking whether multiple entities exist in a data store.
/// Returns a vector of tuples where each tuple contains the UUID and a boolean indicating existence.
/// 
/// # Type Parameters
/// * `DB` - The database type (must implement sqlx::Database)
///
/// # Example
/// ```ignore
/// impl<DB: Database> ExistByIds<DB> for PersonRepositoryImpl<DB> {
///     async fn exist_by_ids(&self, ids: &[Uuid]) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
///         // Implementation
///     }
/// }
/// ```
#[async_trait]
pub trait ExistByIds<DB: Database>: Send + Sync {
    /// Check existence of multiple entities by their unique identifiers
    /// 
    /// # Arguments
    /// * `ids` - A slice of UUIDs to check
    /// 
    /// # Returns
    /// * `Ok(Vec<(Uuid, bool)>)` - A vector of tuples mapping each ID to its existence status
    /// * `Err` - An error if the query could not be executed
    async fn exist_by_ids(&self, ids: &[Uuid]) -> Result<Vec<(Uuid, bool)>, Box<dyn std::error::Error + Send + Sync>>;
}