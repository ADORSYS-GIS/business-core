use async_trait::async_trait;
use sqlx::Database;
use uuid::Uuid;

use crate::models::identifiable::Identifiable;

/// Generic repository trait for loading entities by their ID
/// 
/// This trait provides a standard interface for loading entities from a data store.
/// Any entity that implements the Identifiable trait can be loaded using this trait.
/// 
/// # Type Parameters
/// * `DB` - The database type (must implement sqlx::Database)
/// * `T` - The entity type that must implement Identifiable trait
///
/// # Example
/// ```ignore
/// impl<DB: Database> Load<DB, PersonModel> for PersonRepositoryImpl<DB> {
///     async fn load(&self, id: Uuid) -> Result<PersonModel, Box<dyn Error + Send + Sync>> {
///         // Implementation
///     }
/// }
/// ```
#[async_trait]
pub trait Load<DB: Database, T: Identifiable>: Send + Sync {
    /// Load an entity by its unique identifier
    /// 
    /// # Arguments
    /// * `id` - The UUID of the entity to load
    /// 
    /// # Returns
    /// * `Ok(T)` - The loaded entity
    /// * `Err` - An error if the entity could not be loaded
    async fn load(&self, id: Uuid) -> Result<T, Box<dyn std::error::Error + Send + Sync>>;
}