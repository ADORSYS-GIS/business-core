use std::sync::Arc;

use postgres_index_cache::CacheNotificationListener;
use postgres_unit_of_work::UnitOfWorkSession;
use sqlx::PgPool;

use crate::repository::executor::Executor;

use super::{
    account_gl_mapping_repository::repo_impl::AccountGlMappingRepositoryImpl,
    fee_type_gl_mapping_repository::repo_impl::FeeTypeGlMappingRepositoryImpl,
};

/// Factory for creating product module repositories with audit and main cache
pub struct ProductRepoFactory {
    pub fee_type_gl_mapping_repository: Arc<FeeTypeGlMappingRepositoryImpl>,
    pub account_gl_mapping_repository: Arc<AccountGlMappingRepositoryImpl>,
}

impl ProductRepoFactory {
    /// Create a new ProductRepoFactory singleton with cache configuration
    ///
    /// Optionally register cache handlers with a notification listener
    pub fn new(pool: Arc<PgPool>, listener: Option<&mut CacheNotificationListener>) -> Arc<Self> {
        let fee_type_gl_mapping_repository =
            Arc::new(FeeTypeGlMappingRepositoryImpl::new(pool.clone()));
        let account_gl_mapping_repository =
            Arc::new(AccountGlMappingRepositoryImpl::new(pool));

        if let Some(l) = listener {
            l.register_handler(
                "fee_type_gl_mapping_idx",
                Arc::downgrade(&fee_type_gl_mapping_repository.fee_type_gl_mapping_idx_cache)
                    as _,
            );
            l.register_handler(
                "account_gl_mapping_idx",
                Arc::downgrade(&account_gl_mapping_repository.account_gl_mapping_idx_cache) as _,
            );
        }

        Arc::new(Self {
            fee_type_gl_mapping_repository,
            account_gl_mapping_repository,
        })
    }

    /// Build all product repositories with the given executor
    pub fn build_all_repos(&self, session: &impl UnitOfWorkSession) -> ProductRepositories {
        ProductRepositories {
            fee_type_gl_mapping_repository: self.build_fee_type_gl_mapping_repo(session),
            account_gl_mapping_repository: self.build_account_gl_mapping_repo(session),
        }
    }

    pub fn build_fee_type_gl_mapping_repo(
        &self,
        session: &impl UnitOfWorkSession,
    ) -> FeeTypeGlMappingRepositoryImpl {
        let mut repo = (*self.fee_type_gl_mapping_repository).clone();
        repo.executor = Executor::new_with_session(session);
        repo
    }

    pub fn build_account_gl_mapping_repo(
        &self,
        session: &impl UnitOfWorkSession,
    ) -> AccountGlMappingRepositoryImpl {
        let mut repo = (*self.account_gl_mapping_repository).clone();
        repo.executor = Executor::new_with_session(session);
        repo
    }
}

/// Container for all product module repositories
pub struct ProductRepositories {
    pub fee_type_gl_mapping_repository: FeeTypeGlMappingRepositoryImpl,
    pub account_gl_mapping_repository: AccountGlMappingRepositoryImpl,
}