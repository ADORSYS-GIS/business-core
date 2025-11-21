use std::sync::Arc;
use postgres_unit_of_work::UnitOfWorkSession;
use super::NamedRepositoryImpl;

/// Factory for creating description module repositories (without caching)
pub struct DescriptionRepoFactory {
    // No cache fields needed
}

impl DescriptionRepoFactory {
    /// Create a new DescriptionRepoFactory singleton
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }

    /// Build a NamedRepository with the given executor
    pub fn build_named_repo(&self, session: &impl UnitOfWorkSession) -> Arc<NamedRepositoryImpl> {
        let repo = Arc::new(NamedRepositoryImpl::new(
            session.executor().clone(),
        ));
        session.register_transaction_aware(repo.clone());
        repo
    }

    /// Build all description repositories with the given executor
    pub fn build_all_repos(&self, session: &impl UnitOfWorkSession) -> DescriptionRepositories {
        DescriptionRepositories {
            named_repository: self.build_named_repo(session),
        }
    }
}

/// Container for all description module repositories
pub struct DescriptionRepositories {
    pub named_repository: Arc<NamedRepositoryImpl>,
}