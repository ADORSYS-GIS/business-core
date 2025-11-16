use async_trait::async_trait;
use business_core_db::models::person::entity_reference::EntityReferenceModel;
use business_core_db::repository::load_batch::LoadBatch;
use crate::utils::TryFromRow;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::EntityReferenceRepositoryImpl;

impl EntityReferenceRepositoryImpl {
    pub(super) async fn load_batch_impl(
        repo: &EntityReferenceRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<Option<EntityReferenceModel>>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        
        let query = r#"SELECT * FROM entity_reference WHERE id = ANY($1)"#;
        let rows = {
            let mut tx = repo.executor.tx.lock().await;
            if let Some(transaction) = tx.as_mut() {
                sqlx::query(query).bind(ids).fetch_all(&mut **transaction).await?
            } else {
                return Err("Transaction has been consumed".into());
            }
        };
        
        let mut item_map = std::collections::HashMap::new();
        for row in rows {
            let item = EntityReferenceModel::try_from_row(&row)?;
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
impl LoadBatch<Postgres, EntityReferenceModel> for EntityReferenceRepositoryImpl {
    async fn load_batch(&self, ids: &[Uuid]) -> Result<Vec<Option<EntityReferenceModel>>, Box<dyn Error + Send + Sync>> {
        Self::load_batch_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::person::entity_reference_repository::test_utils::create_test_entity_reference;
    use crate::repository::person::test_utils::{create_test_audit_log, create_test_person};
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::load_batch::LoadBatch;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_load_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;
        let entity_reference_repo = &ctx.person_repos().entity_reference_repository;

        let person = create_test_person("Alice Smith");
        let person_id = person.id;
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        person_repo.create_batch(vec![person], Some(audit_log.id)).await?;

        let mut entity_references = Vec::new();
        for i in 0..3 {
            let entity_reference = create_test_entity_reference(person_id, &format!("LOAD-{}", i));
            entity_references.push(entity_reference);
        }

        let saved = entity_reference_repo.create_batch(entity_references.clone(), Some(audit_log.id)).await?;

        let ids: Vec<Uuid> = saved.iter().map(|s| s.id).collect();
        let loaded = entity_reference_repo.load_batch(&ids).await?;

        assert_eq!(loaded.len(), 3);
        for item in loaded {
            assert!(item.is_some());
            let entity_reference = item.unwrap();
            assert_eq!(entity_reference.person_id, person_id);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_load_batch_with_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;
        let entity_reference_repo = &ctx.person_repos().entity_reference_repository;

        let person = create_test_person("Bob Jones");
        let person_id = person.id;
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        person_repo.create_batch(vec![person], Some(audit_log.id)).await?;

        let entity_reference = create_test_entity_reference(person_id, "LOAD-TEST");

        let saved = entity_reference_repo.create_batch(vec![entity_reference], Some(audit_log.id)).await?;

        let mut ids = vec![saved[0].id];
        ids.push(Uuid::new_v4()); // Add non-existing ID

        let loaded = entity_reference_repo.load_batch(&ids).await?;

        assert_eq!(loaded.len(), 2);
        assert!(loaded[0].is_some());
        assert!(loaded[1].is_none());

        Ok(())
    }
}