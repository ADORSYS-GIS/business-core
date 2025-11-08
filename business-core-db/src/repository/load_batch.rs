use async_trait::async_trait;
use sqlx::Database;
use uuid::Uuid;

use crate::models::identifiable::Identifiable;

/// Generic repository trait for loading multiple entities by their IDs
/// 
/// This trait provides a standard interface for batch loading entities from a data store.
/// Any entity that implements the Identifiable trait can be loaded using this trait.
/// Returns items in the same order as the provided IDs.
/// Missing items are represented as None in the result.
/// 
/// # Type Parameters
/// * `DB` - The database type (must implement sqlx::Database)
/// * `T` - The entity type that must implement Identifiable trait
///
/// # Example
/// ```ignore
/// impl<DB: Database> LoadBatch<DB, PersonModel> for PersonRepositoryImpl<DB> {
///     async fn load_batch(&self, ids: &[Uuid]) -> Result<Vec<Option<PersonModel>>, Box<dyn Error + Send + Sync>> {
///         // Implementation
///     }
/// }
/// ```
#[async_trait]
pub trait LoadBatch<DB: Database, T: Identifiable>: Send + Sync {
    /// Load multiple entities by their unique identifiers
    /// 
    /// # Arguments
    /// * `ids` - A slice of UUIDs of the entities to load
    /// 
    /// # Returns
    /// * `Ok(Vec<Option<T>>)` - A vector of optional entities in the same order as the provided IDs
    ///   - `Some(T)` for entities that exist
    ///   - `None` for entities that do not exist
    /// * `Err` - An error if the query could not be executed
    async fn load_batch(&self, ids: &[Uuid]) -> Result<Vec<Option<T>>, Box<dyn std::error::Error + Send + Sync>>;
}