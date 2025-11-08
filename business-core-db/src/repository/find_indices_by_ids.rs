use async_trait::async_trait;
use sqlx::Database;
use uuid::Uuid;

use crate::models::index::Index;

/// Generic repository trait for finding multiple index entities by their IDs
/// 
/// This trait provides a standard interface for finding multiple index entities from a data store.
/// Any entity that implements the Index trait can be queried using this trait.
/// Returns a vector of entities that were found (entities that don't exist are not included).
/// 
/// # Type Parameters
/// * `DB` - The database type (must implement sqlx::Database)
/// * `T` - The index entity type that must implement Index trait
///
/// # Example
/// ```ignore
/// impl<DB: Database> FindIndicesByIds<DB, PersonIdxModel> for PersonRepositoryImpl<DB> {
///     async fn find_indices_by_ids(&self, ids: &[Uuid]) -> Result<Vec<PersonIdxModel>, Box<dyn Error + Send + Sync>> {
///         // Implementation
///     }
/// }
/// ```
#[async_trait]
pub trait FindIndicesByIds<DB: Database, T: Index>: Send + Sync {
    /// Find multiple index entities by their unique identifiers
    /// 
    /// # Arguments
    /// * `ids` - A slice of UUIDs to search for
    /// 
    /// # Returns
    /// * `Ok(Vec<T>)` - A vector of found index entities (missing entities are not included)
    /// * `Err` - An error if the query could not be executed
    async fn find_indices_by_ids(&self, ids: &[Uuid]) -> Result<Vec<T>, Box<dyn std::error::Error + Send + Sync>>;
}