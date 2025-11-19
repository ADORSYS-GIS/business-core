use async_trait::async_trait;
use business_core_db::repository::exist_by_ids::ExistByIds;
use sqlx::{Postgres, Row};
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::ActivityLogRepositoryImpl;

impl ActivityLogRepositoryImpl {
    pub(super) async fn exist_by_ids_impl(
        repo: &ActivityLogRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        
        let query = r#"SELECT id FROM person_activity_log WHERE id = ANY($1)"#;
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
impl ExistByIds<Postgres> for ActivityLogRepositoryImpl {
    async fn exist_by_ids(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        Self::exist_by_ids_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::person::activity_log_repository::test_utils::create_test_activity_log;
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::exist_by_ids::ExistByIds;
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
    async fn test_exist_by_ids() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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
        let non_existent_id = uuid::Uuid::new_v4();
        
        activity_log_repo.create_batch(vec![activity_log], Some(audit_log.id)).await?;

        let results = activity_log_repo.exist_by_ids(&[activity_log_id, non_existent_id]).await?;

        assert_eq!(results.len(), 2);
        assert_eq!(results[0], (activity_log_id, true));
        assert_eq!(results[1], (non_existent_id, false));

        Ok(())
    }
}