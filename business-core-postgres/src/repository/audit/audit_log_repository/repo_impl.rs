use async_trait::async_trait;
use business_core_db::{
    models::audit::AuditLogModel,
    repository::{load::Load, load_batch::LoadBatch},
};
use sqlx::Postgres;
use uuid::Uuid;
use postgres_unit_of_work::Executor;

pub struct AuditLogRepositoryImpl {
    pub(crate) executor: Executor,
}

impl AuditLogRepositoryImpl {
    pub fn new(executor: Executor) -> Self {
        Self { executor }
    }

    pub async fn create(&self, audit_log: &AuditLogModel) -> Result<AuditLogModel, Box<dyn std::error::Error + Send + Sync>> {
        Self::create_impl(self, audit_log).await
    }
}

#[async_trait]
impl Load<Postgres, AuditLogModel> for AuditLogRepositoryImpl {
    async fn load(&self, id: Uuid) -> Result<AuditLogModel, Box<dyn std::error::Error + Send + Sync>> {
        let results = self.load_batch(&[id]).await?;
        results.into_iter().next().flatten()
            .ok_or_else(|| "Entity not found".into())
    }
}

#[async_trait]
impl LoadBatch<Postgres, AuditLogModel> for AuditLogRepositoryImpl {
    async fn load_batch(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<Option<AuditLogModel>>, Box<dyn std::error::Error + Send + Sync>> {
        super::load_batch::load_batch_impl(&self.executor, ids).await
    }
}