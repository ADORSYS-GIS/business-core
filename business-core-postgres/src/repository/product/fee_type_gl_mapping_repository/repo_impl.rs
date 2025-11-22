use async_trait::async_trait;
use business_core_db::models::product::fee_type_gl_mapping::{
    FeeTypeGlMappingIdxModel, FeeTypeGlMappingModel,
};
use business_core_db::repository::product::fee_type_gl_mapping::FeeTypeGlMappingRepository;
use business_core_db::repository_error::RepositoryError;
use business_core_db::search::pageable::Page;
use business_core_db::search::pageable::PageRequest;
use cache::Cache;
use common_utils::fp::Fp;
use sqlx::PgPool;
use std::error::Error;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::repository::executor::Executor;

#[derive(Clone)]
pub struct FeeTypeGlMappingRepositoryImpl {
    pub executor: Executor,
    pub fee_type_gl_mapping_idx_cache: Arc<RwLock<Cache<Uuid, FeeTypeGlMappingIdxModel>>>,
}

impl FeeTypeGlMappingRepositoryImpl {
    pub fn new(pool: Arc<PgPool>) -> Self {
        Self {
            executor: Executor::new(pool),
            fee_type_gl_mapping_idx_cache: Arc::new(RwLock::new(Cache::new(None))),
        }
    }
}

#[async_trait]
impl Fp for FeeTypeGlMappingRepositoryImpl {}

#[async_trait]
impl FeeTypeGlMappingRepository for FeeTypeGlMappingRepositoryImpl {
    async fn create_batch(
        &self,
        items: Vec<FeeTypeGlMappingModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<FeeTypeGlMappingModel>, Box<dyn Error + Send + Sync>> {
        self.create_batch_internal(items, audit_log_id).await
    }

    async fn load_batch(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<FeeTypeGlMappingModel>, Box<dyn Error + Send + Sync>> {
        self.load_batch_internal(ids).await
    }

    async fn load_audits(
        &self,
        id: Uuid,
        page_req: PageRequest,
    ) -> Result<Page<FeeTypeGlMappingModel>, Box<dyn Error + Send + Sync>> {
        self.load_audits_internal(id, page_req).await
    }

    async fn update_batch(
        &self,
        items: Vec<FeeTypeGlMappingModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<FeeTypeGlMappingModel>, Box<dyn Error + Send + Sync>> {
        self.update_batch_internal(items, audit_log_id).await
    }

    async fn delete_batch(
        &self,
        ids: &[Uuid],
        audit_log_id: Option<Uuid>,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        self.delete_batch_internal(ids, audit_log_id).await
    }

    async fn exist_by_ids(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<Uuid>, RepositoryError> {
        self.exist_by_ids_internal(ids).await
    }
}