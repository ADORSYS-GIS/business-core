use async_trait::async_trait;
use sqlx::Database;
use uuid::Uuid;

use crate::models::auditable::Auditable;
use crate::repository::pagination::{Page, PageRequest};

/// Generic repository trait for loading audit records for entities with pagination
///
/// This trait provides a standard interface for loading audit records from a data store with pagination support.
/// Any entity that implements the Auditable trait can have its audit records loaded using this trait.
///
/// # Type Parameters
/// * `DB` - The database type (must implement sqlx::Database)
/// * `T` - The entity type that must implement Auditable trait
///
/// # Example
/// ```ignore
/// use business_core_db::repository::pagination::PageRequest;
///
/// impl<DB: Database> LoadAudits<DB, PersonModel> for PersonRepositoryImpl<DB> {
///     async fn load_audits(&self, id: Uuid, page: PageRequest) -> Result<Page<PersonModel>, Box<dyn Error + Send + Sync>> {
///         // Implementation
///     }
/// }
///
/// // Usage:
/// let page = repo.load_audits(person_id, PageRequest::new(20, 0)).await?;
/// println!("Page {} of {}", page.page_number(), page.total_pages());
/// ```
#[async_trait]
pub trait LoadAudits<DB: Database, T: Auditable>: Send + Sync {
    /// Load paginated audit records for an entity by its unique identifier
    ///
    /// # Arguments
    /// * `id` - The UUID of the entity whose audit records should be loaded
    /// * `page` - The pagination parameters (limit and offset)
    ///
    /// # Returns
    /// * `Ok(Page<T>)` - A page containing the entity state
    /// * `Err` - An error if the audit records could not be loaded
    async fn load_audits(&self, id: Uuid, page: PageRequest) -> Result<Page<T>, Box<dyn std::error::Error + Send + Sync>>;
}