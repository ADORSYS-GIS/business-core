use async_trait::async_trait;
use business_core_db::models::person::compliance_status::ComplianceStatusModel;
use business_core_db::repository::load_batch::LoadBatch;
use crate::utils::TryFromRow;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::ComplianceStatusRepositoryImpl;

impl ComplianceStatusRepositoryImpl {
    pub(super) async fn load_batch_impl(
        repo: &ComplianceStatusRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<Option<ComplianceStatusModel>>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        
        let query = r#"SELECT * FROM person_compliance_status WHERE id = ANY($1)"#;
        let rows = {
            let mut tx = repo.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
            sqlx::query(query).bind(ids).fetch_all(&mut **transaction).await?
        };
        
        let mut item_map = std::collections::HashMap::new();
        for row in rows {
            let item = ComplianceStatusModel::try_from_row(&row)?;
            item_map.insert(item.id, item);
        }
        
        let mut result = Vec::with_capacity(ids.len());
        for id in ids {
            result.push(item_map.remove(id));
        }
        Ok(result)
    }
}

#[async_trait]
impl LoadBatch<Postgres, ComplianceStatusModel> for ComplianceStatusRepositoryImpl {
    async fn load_batch(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<Option<ComplianceStatusModel>>, Box<dyn Error + Send + Sync>> {
        Self::load_batch_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::person::compliance_status_repository::test_utils::create_test_compliance_status;
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::load_batch::LoadBatch;

    fn create_test_audit_log() -> business_core_db::models::audit::audit_log::AuditLogModel {
        business_core_db::models::audit::audit_log::AuditLogModel {
            id: uuid::Uuid::new_v4(),
            updated_at: chrono::Utc::now(),
            updated_by_person_id: uuid::Uuid::new_v4(),
        }
    }

    #[tokio::test]
    async fn test_load_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let compliance_status_repo = &ctx.person_repos().compliance_status_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let entity = create_test_compliance_status(uuid::Uuid::new_v4());
        let entity_id = entity.id;
        
        compliance_status_repo.create_batch(vec![entity], Some(audit_log.id)).await?;

        let loaded = compliance_status_repo.load_batch(&[entity_id]).await?;

        assert_eq!(loaded.len(), 1);
        assert!(loaded[0].is_some());
        assert_eq!(loaded[0].as_ref().unwrap().id, entity_id);

        Ok(())
    }
}