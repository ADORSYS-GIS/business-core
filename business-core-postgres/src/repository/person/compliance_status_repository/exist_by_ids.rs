use async_trait::async_trait;
use business_core_db::repository::exist_by_ids::ExistByIds;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use sqlx::Row;

use super::repo_impl::ComplianceStatusRepositoryImpl;

impl ComplianceStatusRepositoryImpl {
    pub(super) async fn exist_by_ids_impl(
        repo: &ComplianceStatusRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        
        let query = r#"SELECT id FROM person_compliance_status WHERE id = ANY($1)"#;
        let rows = {
            let mut tx = repo.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
            sqlx::query(query).bind(ids).fetch_all(&mut **transaction).await?
        };
        
        let existing_ids: std::collections::HashSet<Uuid> = rows
            .iter()
            .map(|row| row.get("id"))
            .collect();
        
        let mut result = Vec::new();
        for &id in ids {
            result.push((id, existing_ids.contains(&id)));
        }
        Ok(result)
    }
}

#[async_trait]
impl ExistByIds<Postgres> for ComplianceStatusRepositoryImpl {
    async fn exist_by_ids(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        Self::exist_by_ids_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::person::compliance_status_repository::test_utils::create_test_compliance_status;
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::exist_by_ids::ExistByIds;

    fn create_test_audit_log() -> business_core_db::models::audit::audit_log::AuditLogModel {
        business_core_db::models::audit::audit_log::AuditLogModel {
            id: uuid::Uuid::new_v4(),
            updated_at: chrono::Utc::now(),
            updated_by_person_id: uuid::Uuid::new_v4(),
        }
    }

    #[tokio::test]
    async fn test_exist_by_ids() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let compliance_status_repo = &ctx.person_repos().compliance_status_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let entity = create_test_compliance_status(uuid::Uuid::new_v4());
        let entity_id = entity.id;
        let non_existent_id = uuid::Uuid::new_v4();
        
        compliance_status_repo.create_batch(vec![entity], Some(audit_log.id)).await?;

        let results = compliance_status_repo.exist_by_ids(&[entity_id, non_existent_id]).await?;

        assert_eq!(results.len(), 2);
        assert_eq!(results[0], (entity_id, true));
        assert_eq!(results[1], (non_existent_id, false));

        Ok(())
    }
}