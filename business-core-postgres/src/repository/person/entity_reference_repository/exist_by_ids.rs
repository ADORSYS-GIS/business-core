use async_trait::async_trait;
use business_core_db::repository::exist_by_ids::ExistByIds;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::EntityReferenceRepositoryImpl;

impl EntityReferenceRepositoryImpl {
    pub(super) async fn exist_by_ids_impl(
        repo: &EntityReferenceRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        let mut result = Vec::new();
        let cache = repo.entity_reference_idx_cache.read().await;
        for &id in ids {
            result.push((id, cache.contains_primary(&id)));
        }
        Ok(result)
    }
}

#[async_trait]
impl ExistByIds<Postgres> for EntityReferenceRepositoryImpl {
    async fn exist_by_ids(&self, ids: &[Uuid]) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        Self::exist_by_ids_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::person::entity_reference_repository::test_utils::create_test_entity_reference;
    use crate::repository::person::test_utils::{create_test_audit_log, create_test_person};
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::exist_by_ids::ExistByIds;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_exist_by_ids() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;
        let entity_reference_repo = &ctx.person_repos().entity_reference_repository;

        let person = create_test_person("Frank Garcia");
        let person_id = person.id;
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        person_repo.create_batch(vec![person], Some(audit_log.id)).await?;

        let entity_reference = create_test_entity_reference(person_id, "EXIST-TEST");

        let saved = entity_reference_repo.create_batch(vec![entity_reference], Some(audit_log.id)).await?;

        let existing_id = saved[0].id;
        let non_existing_id = Uuid::new_v4();

        let result = entity_reference_repo.exist_by_ids(&[existing_id, non_existing_id]).await?;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], (existing_id, true));
        assert_eq!(result[1], (non_existing_id, false));

        Ok(())
    }

    #[tokio::test]
    async fn test_custom_finder_methods() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;
        let entity_reference_repo = &ctx.person_repos().entity_reference_repository;

        let person = create_test_person("Grace Martinez");
        let person_id = person.id;
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        person_repo.create_batch(vec![person], Some(audit_log.id)).await?;

        let mut entity_references = Vec::new();
        for i in 0..3 {
            let entity_reference = create_test_entity_reference(person_id, &format!("FINDER-{}", i));
            entity_references.push(entity_reference);
        }

        let saved = entity_reference_repo.create_batch(entity_references.clone(), Some(audit_log.id)).await?;

        // Test find_ids_by_person_id
        let ids_by_person = entity_reference_repo.find_ids_by_person_id(person_id).await?;
        assert_eq!(ids_by_person.len(), 3);

        // Test find_ids_by_reference_external_id_hash
        let first_ref_hash = business_core_db::utils::hash_as_i64(&saved[0].reference_external_id.as_str()).unwrap();
        let ids_by_hash = entity_reference_repo.find_ids_by_reference_external_id_hash(first_ref_hash).await?;
        assert_eq!(ids_by_hash.len(), 1);
        assert_eq!(ids_by_hash[0], saved[0].id);

        Ok(())
    }
}