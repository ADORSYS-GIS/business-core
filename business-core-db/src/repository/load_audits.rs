use async_trait::async_trait;
use sqlx::Database;
use uuid::Uuid;

use crate::models::auditable::Auditable;
use crate::models::audit::audit_hash::AuditHashModel;

/// Generic repository trait for loading audit records for entities
/// 
/// This trait provides a standard interface for loading audit records from a data store.
/// Any entity that implements the Auditable trait can have its audit records loaded using this trait.
/// 
/// # Type Parameters
/// * `DB` - The database type (must implement sqlx::Database)
/// * `T` - The entity type that must implement Auditable trait
///
/// # Example
/// ```ignore
/// impl<DB: Database> LoadAudits<DB, PersonModel> for PersonRepositoryImpl<DB> {
///     async fn load_audits(&self, id: Uuid) -> Result<Vec<(PersonModel, AuditHashModel)>, Box<dyn Error + Send + Sync>> {
///         // Implementation
///     }
/// }
/// ```
#[async_trait]
pub trait LoadAudits<DB: Database, T: Auditable>: Send + Sync {
    /// Load audit records for an entity by its unique identifier
    /// 
    /// # Arguments
    /// * `id` - The UUID of the entity whose audit records should be loaded
    /// 
    /// # Returns
    /// * `Ok(Vec<(T, AuditHashModel)>)` - A vector of tuples containing the entity state and its audit metadata
    /// * `Err` - An error if the audit records could not be loaded
    async fn load_audits(&self, id: Uuid) -> Result<Vec<(T, AuditHashModel)>, Box<dyn std::error::Error + Send + Sync>>;
}