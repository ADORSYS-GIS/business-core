use business_core_db::models::audit::AuditLinkModel;
use postgres_unit_of_work::Executor;
use uuid::Uuid;

pub struct AuditLinkRepositoryImpl {
    pub(crate) executor: Executor,
}

impl AuditLinkRepositoryImpl {
    pub fn new(executor: Executor) -> Self {
        Self { executor }
    }

    pub async fn create(
        &self,
        audit_link: &AuditLinkModel,
    ) -> Result<(), sqlx::Error> {
        Self::create_impl(self, audit_link).await
    }

    pub async fn find_by_audit_log_id(
        &self,
        audit_log_id: Uuid,
    ) -> Result<Vec<AuditLinkModel>, sqlx::Error> {
        Self::find_by_audit_log_id_impl(self, audit_log_id).await
    }
}