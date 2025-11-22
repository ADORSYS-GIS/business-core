use std::sync::Arc;
use postgres_unit_of_work::UnitOfWorkSession;
use postgres_index_cache::CacheNotificationListener;

/// Factory for creating product module repositories with audit and main cache
pub struct ProductRepoFactory {}

impl ProductRepoFactory {
    /// Create a new ProductRepoFactory singleton with cache configuration
    ///
    /// Optionally register cache handlers with a notification listener
    pub fn new(_listener: Option<&mut CacheNotificationListener>) -> Arc<Self> {
        Arc::new(Self {})
    }

    /// Build all product repositories with the given executor
    pub fn build_all_repos(&self, _session: &impl UnitOfWorkSession) -> ProductRepositories {
        ProductRepositories {}
    }
}

/// Container for all product module repositories
pub struct ProductRepositories {}