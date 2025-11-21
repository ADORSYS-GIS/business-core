use async_trait::async_trait;
use sqlx::Database;
use uuid::Uuid;

use crate::models::index::Index;

/// Generic repository trait for finding index entities by their ID
/// 
/// This trait provides a standard interface for finding index entities from a data store.
/// Any entity that implements the Index trait can be queried using this trait.
/// Returns an Option to handle cases where the entity might not exist.
/// 
/// # Type Parameters
/// * `DB` - The database type (must implement sqlx::Database)
/// * `T` - The index entity type that must implement Index trait
///
/// # Example
/// ```ignore
/// impl<DB: Database> FindIndexById<DB, PersonIdxModel> for PersonRepositoryImpl<DB> {
///     async fn find_index_by_id(&self, id: Uuid) -> Result<Option<PersonIdxModel>, Box<dyn Error + Send + Sync>> {
///         // Implementation
///     }
/// }
/// ```
#[async_trait]
pub trait FindById<DB: Database, T: Index>: Send + Sync {
    /// Find an index entity by its unique identifier
    /// 
    /// # Arguments
    /// * `id` - The UUID of the entity to find
    /// 
    /// # Returns
    /// * `Ok(Some(T))` - The found index entity
    /// * `Ok(None)` - If the entity does not exist
    /// * `Err` - An error if the query could not be executed
    async fn find_by_id(&self, id: Uuid) -> Result<Option<T>, Box<dyn std::error::Error + Send + Sync>>;
}