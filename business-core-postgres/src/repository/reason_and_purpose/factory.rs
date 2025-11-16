use std::sync::Arc;
use parking_lot::RwLock as ParkingRwLock;
use postgres_unit_of_work::UnitOfWorkSession;
use postgres_index_cache::{CacheNotificationListener, IndexCacheHandler};
use business_core_db::models::reason_and_purpose::{
    compliance_metadata::ComplianceMetadataIdxModel,
};
use super::{ComplianceMetadataRepositoryImpl};

/// Factory for creating reason_and_purpose module repositories
///
/// This factory holds all caches for the reason_and_purpose module and provides
/// methods to build repositories with the appropriate executor.
/// This should be used as a singleton throughout the application.
pub struct ReasonAndPurposeRepoFactory {
    compliance_metadata_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<ComplianceMetadataIdxModel>>>,
}

impl ReasonAndPurposeRepoFactory {
    /// Create a new ReasonAndPurposeRepoFactory singleton
    ///
    /// Optionally register cache handlers with a notification listener
    pub fn new(listener: Option<&mut CacheNotificationListener>) -> Arc<Self> {
        let compliance_metadata_idx_cache = Arc::new(ParkingRwLock::new(
            business_core_db::IdxModelCache::new(vec![]).unwrap()
        ));
        
        // Register handlers with listener if provided
        if let Some(listener) = listener {
            let handler = Arc::new(IndexCacheHandler::new(
                "compliance_metadata_idx".to_string(),
                compliance_metadata_idx_cache.clone(),
            ));
            listener.register_handler(handler);
        }
        
        Arc::new(Self {
            compliance_metadata_idx_cache,
        })
    }

    /// Build a ComplianceMetadataRepository with the given executor
    pub fn build_compliance_metadata_repo(&self, session: &impl UnitOfWorkSession) -> Arc<ComplianceMetadataRepositoryImpl> {
        let repo = Arc::new(ComplianceMetadataRepositoryImpl::new(
            session.executor().clone(),
            self.compliance_metadata_idx_cache.clone(),
        ));
        session.register_transaction_aware(repo.clone());
        repo
    }

    /// Build all reason_and_purpose repositories with the given executor
    pub fn build_all_repos(&self, session: &impl UnitOfWorkSession) -> ReasonAndPurposeRepositories {
        ReasonAndPurposeRepositories {
            compliance_metadata_repository: self.build_compliance_metadata_repo(session),
        }
    }
}

/// Container for all reason_and_purpose module repositories
pub struct ReasonAndPurposeRepositories {
    pub compliance_metadata_repository: Arc<ComplianceMetadataRepositoryImpl>,
}