use async_trait::async_trait;
use business_core_db::repository::exist_by_ids::ExistByIds;
use sqlx::{Postgres, Row};
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::ReasonReferenceRepositoryImpl;

impl ReasonReferenceRepositoryImpl {
    pub(super) async fn exist_by_ids_impl(
        repo: &ReasonReferenceRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        
        let query = r#"SELECT id FROM reason_reference WHERE id = ANY($1)"#;
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
impl ExistByIds<Postgres> for ReasonReferenceRepositoryImpl {
    async fn exist_by_ids(&self, ids: &[Uuid]) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        Self::exist_by_ids_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::reason_and_purpose::reason_reference_repository::test_utils::create_test_reason_reference;
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::exist_by_ids::ExistByIds;
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
    async fn test_exist_by_ids() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let compliance_metadata_repo = &ctx.reason_and_purpose_repos().compliance_metadata_repository;
        let reason_repo = &ctx.reason_and_purpose_repos().reason_repository;
        let reason_reference_repo = &ctx.reason_and_purpose_repos().reason_reference_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create compliance metadata and reason first
        let compliance_metadata = create_test_compliance_metadata(Some("GDPR-007"), true, false);
        compliance_metadata_repo.create_batch(vec![compliance_metadata.clone()], Some(audit_log.id)).await?;

        let reason = create_test_reason_with_compliance_metadata("DATA_RETENTION", "Data Retention Reason", Some(compliance_metadata.id));
        let saved_reasons = reason_repo.create_batch(vec![reason], Some(audit_log.id)).await?;
        let reason_id = saved_reasons[0].id;

        let reason_reference = create_test_reason_reference(reason_id, Uuid::new_v4());
        let saved = reason_reference_repo.create_batch(vec![reason_reference], Some(audit_log.id)).await?;

        let existing_id = saved[0].id;
        let non_existing_id = Uuid::new_v4();

        let result = reason_reference_repo.exist_by_ids(&[existing_id, non_existing_id]).await?;

        assert_eq!(result.len(), 2);
        assert_eq!(result[0], (existing_id, true));
        assert_eq!(result[1], (non_existing_id, false));

        Ok(())
    }
}