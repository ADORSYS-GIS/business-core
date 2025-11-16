use std::sync::Arc;
use postgres_unit_of_work::UnitOfWorkSession;
use super::{
    audit_link_repository::AuditLinkRepositoryImpl,
    audit_log_repository::AuditLogRepositoryImpl,
};

/// Factory for creating audit module repositories
///
/// This factory holds all caches for the audit module and provides
/// methods to build repositories with the appropriate executor.
/// This should be used as a singleton throughout the application.
#[derive(Default)]
pub struct AuditRepoFactory {
    // Currently no caches needed for audit module
}

impl AuditRepoFactory {
    /// Create a new AuditRepoFactory singleton
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }

    /// Build an AuditLogRepository with the given executor
    pub fn build_audit_log_repo(&self, session: &impl UnitOfWorkSession) -> Arc<AuditLogRepositoryImpl> {
        Arc::new(AuditLogRepositoryImpl::new(session.executor().clone()))
    }

    /// Build an AuditLinkRepository with the given executor
    pub fn build_audit_link_repo(&self, session: &impl UnitOfWorkSession) -> Arc<AuditLinkRepositoryImpl> {
        Arc::new(AuditLinkRepositoryImpl::new(session.executor().clone()))
    }

    /// Build all audit repositories with the given executor
    pub fn build_all_repos(&self, session: &impl UnitOfWorkSession) -> AuditRepositories {
        AuditRepositories {
            audit_log_repository: self.build_audit_log_repo(session),
            audit_link_repository: self.build_audit_link_repo(session),
        }
    }
}

/// Container for all audit module repositories
pub struct AuditRepositories {
    pub audit_log_repository: Arc<AuditLogRepositoryImpl>,
    pub audit_link_repository: Arc<AuditLinkRepositoryImpl>,
}