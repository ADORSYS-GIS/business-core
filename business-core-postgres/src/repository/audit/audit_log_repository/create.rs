use business_core_db::models::audit::AuditLogModel;

use super::repo_impl::AuditLogRepositoryImpl;

impl AuditLogRepositoryImpl {
    pub(super) async fn create_impl(
        repo: &AuditLogRepositoryImpl,
        audit_log: &AuditLogModel,
    ) -> Result<AuditLogModel, Box<dyn std::error::Error + Send + Sync>> {
        let query = sqlx::query(
            r#"
            INSERT INTO audit_log (id, updated_at, updated_by_person_id)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(audit_log.id)
        .bind(audit_log.updated_at)
        .bind(audit_log.updated_by_person_id);

        // Execute query using the new executor structure
        let mut tx = repo.executor.tx.lock().await;
        if let Some(transaction) = tx.as_mut() {
            query.execute(&mut **transaction).await?;
        } else {
            return Err("Transaction has been consumed".into());
        }

        Ok(audit_log.clone())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::models::audit::AuditLogModel;
    use chrono::Utc;
    use uuid::Uuid;

    fn new_test_audit_log() -> AuditLogModel {
        AuditLogModel {
            id: Uuid::new_v4(),
            updated_at: Utc::now(),
            updated_by_person_id: Uuid::new_v4(),
        }
    }

    #[tokio::test]
    async fn test_create_audit_log() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;

        let audit_log_model = new_test_audit_log();
        let result = audit_log_repo.create(vec![audit_log_model.clone()]).await;

        assert!(result.is_ok());
        let created_logs = result.unwrap();
        assert_eq!(created_logs.len(), 1);
        assert_eq!(created_logs[0].id, audit_log_model.id);

        Ok(())
    }
}