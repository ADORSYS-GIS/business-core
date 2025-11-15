use async_trait::async_trait;
use sqlx::Database;
use uuid::Uuid;

use crate::models::identifiable::Identifiable;

/// Generic repository trait for creating multiple entities in a batch
/// 
/// This trait provides a standard interface for batch creating entities in a data store.
/// Any entity that implements the Identifiable trait can be created using this trait.
/// All creates are performed within a single transaction for atomicity.
/// Returns saved items with any generated fields populated.
/// 
/// # Type Parameters
/// * `DB` - The database type (must implement sqlx::Database)
/// * `T` - The entity type that must implement Identifiable trait
///
/// # Example
/// ```ignore
/// impl<DB: Database> CreateBatch<DB, PersonModel> for PersonRepositoryImpl<DB> {
///     async fn create_batch(&self, items: Vec<PersonModel>, audit_log_id: Uuid) -> Result<Vec<PersonModel>, Box<dyn Error + Send + Sync>> {
///         // Implementation
///     }
/// }
/// ```
#[async_trait]
pub trait CreateBatch<DB: Database, T: Identifiable>: Send + Sync {
    /// Save multiple items in a single transaction
    /// 
    /// # Arguments
    /// * `items` - A vector of entities to create
    /// * `audit_log_id` - The optional UUID of the audit log for tracking this operation
    ///
    /// # Returns
    /// * `Ok(Vec<T>)` - A vector of created entities with generated fields populated
    /// * `Err` - An error if the transaction could not be executed
    async fn create_batch(
        &self,
        items: Vec<T>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<T>, Box<dyn std::error::Error + Send + Sync>>;
}