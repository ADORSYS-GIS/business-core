use async_trait::async_trait;
use business_core_db::models::reason_and_purpose::reason_reference::ReasonReferenceModel;
use business_core_db::repository::load_batch::LoadBatch;
use crate::utils::TryFromRow;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::ReasonReferenceRepositoryImpl;

impl ReasonReferenceRepositoryImpl {
    pub(super) async fn load_batch_impl(
        repo: &ReasonReferenceRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<Option<ReasonReferenceModel>>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        
        let query = r#"SELECT * FROM reason_reference WHERE id = ANY($1)"#;
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
            let item = ReasonReferenceModel::try_from_row(&row)?;
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
impl LoadBatch<Postgres, ReasonReferenceModel> for ReasonReferenceRepositoryImpl {
    async fn load_batch(&self, ids: &[Uuid]) -> Result<Vec<Option<ReasonReferenceModel>>, Box<dyn Error + Send + Sync>> {
        Self::load_batch_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::reason_and_purpose::reason_reference_repository::test_utils::create_test_reason_reference;
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::load_batch::LoadBatch;
    use crate::repository::reason_and_purpose::compliance_metadata_repository::test_utils::test_utils::create_test_compliance_metadata;
    use crate::repository::reason_and_purpose::reason_repository::test_utils::test_utils::create_test_reason_with_compliance_metadata;
    use uuid::Uuid;

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
        let compliance_metadata_repo = &ctx.reason_and_purpose_repos().compliance_metadata_repository;
        let reason_repo = &ctx.reason_and_purpose_repos().reason_repository;
        let reason_reference_repo = &ctx.reason_and_purpose_repos().reason_reference_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create compliance metadata and reason first
        let compliance_metadata = create_test_compliance_metadata(Some("GDPR-002"), true, false);
        compliance_metadata_repo.create_batch(vec![compliance_metadata.clone()], Some(audit_log.id)).await?;

        let reason = create_test_reason_with_compliance_metadata("DATA_STORAGE", "Data Storage Reason", Some(compliance_metadata.id));
        let saved_reasons = reason_repo.create_batch(vec![reason], Some(audit_log.id)).await?;
        let reason_id = saved_reasons[0].id;

        let mut reason_references = Vec::new();
        for _ in 0..3 {
            let reason_reference = create_test_reason_reference(reason_id, Uuid::new_v4());
            reason_references.push(reason_reference);
        }

        let saved = reason_reference_repo.create_batch(reason_references, Some(audit_log.id)).await?;

        let ids: Vec<Uuid> = saved.iter().map(|s| s.id).collect();
        let loaded = reason_reference_repo.load_batch(&ids).await?;

        assert_eq!(loaded.len(), 3);
        for item in loaded {
            assert!(item.is_some());
            let reason_reference = item.unwrap();
            assert_eq!(reason_reference.reason_id, reason_id);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_load_batch_with_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let compliance_metadata_repo = &ctx.reason_and_purpose_repos().compliance_metadata_repository;
        let reason_repo = &ctx.reason_and_purpose_repos().reason_repository;
        let reason_reference_repo = &ctx.reason_and_purpose_repos().reason_reference_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create compliance metadata and reason first
        let compliance_metadata = create_test_compliance_metadata(Some("GDPR-003"), true, false);
        compliance_metadata_repo.create_batch(vec![compliance_metadata.clone()], Some(audit_log.id)).await?;

        let reason = create_test_reason_with_compliance_metadata("DATA_TRANSFER", "Data Transfer Reason", Some(compliance_metadata.id));
        let saved_reasons = reason_repo.create_batch(vec![reason], Some(audit_log.id)).await?;
        let reason_id = saved_reasons[0].id;

        let reason_reference = create_test_reason_reference(reason_id, Uuid::new_v4());
        let saved = reason_reference_repo.create_batch(vec![reason_reference], Some(audit_log.id)).await?;

        let mut ids = vec![saved[0].id];
        ids.push(Uuid::new_v4()); // Add non-existing ID

        let loaded = reason_reference_repo.load_batch(&ids).await?;

        assert_eq!(loaded.len(), 2);
        assert!(loaded[0].is_some());
        assert!(loaded[1].is_none());

        Ok(())
    }
}