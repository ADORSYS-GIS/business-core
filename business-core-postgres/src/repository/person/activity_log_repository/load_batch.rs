use async_trait::async_trait;
use business_core_db::models::person::activity_log::ActivityLogModel;
use business_core_db::repository::load_batch::LoadBatch;
use crate::utils::TryFromRow;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::ActivityLogRepositoryImpl;

impl ActivityLogRepositoryImpl {
    pub(super) async fn load_batch_impl(
        repo: &ActivityLogRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<Option<ActivityLogModel>>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        
        let query = r#"SELECT * FROM person_activity_log WHERE id = ANY($1)"#;
        let rows = {
            let mut tx = repo.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
            sqlx::query(query).bind(ids).fetch_all(&mut **transaction).await?
        };
        
        let mut item_map = std::collections::HashMap::new();
        for row in rows {
            let item = ActivityLogModel::try_from_row(&row)?;
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
impl LoadBatch<Postgres, ActivityLogModel> for ActivityLogRepositoryImpl {
    async fn load_batch(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<Option<ActivityLogModel>>, Box<dyn Error + Send + Sync>> {
        Self::load_batch_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::person::activity_log_repository::test_utils::create_test_activity_log;
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::load_batch::LoadBatch;
    use crate::repository::person::person_repository::test_utils::create_test_person;
    use business_core_db::models::person::person::PersonType;

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
        let person_repo = &ctx.person_repos().person_repository;
        let activity_log_repo = &ctx.person_repos().activity_log_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create a person first
        let person = create_test_person("Test Person", PersonType::Natural);
        let saved_persons = person_repo.create_batch(vec![person], Some(audit_log.id)).await?;
        let person_id = saved_persons[0].id;

        let activity_log = create_test_activity_log(person_id);
        let activity_log_id = activity_log.id;
        
        let _saved = activity_log_repo.create_batch(vec![activity_log], Some(audit_log.id)).await?;

        let loaded = activity_log_repo.load_batch(&[activity_log_id]).await?;

        assert_eq!(loaded.len(), 1);
        assert!(loaded[0].is_some());
        assert_eq!(loaded[0].as_ref().unwrap().id, activity_log_id);

        Ok(())
    }
}