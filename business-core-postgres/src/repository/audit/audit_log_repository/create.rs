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
    use crate::repository::person::test_utils::create_test_audit_log;
        use crate::test_helper::setup_test_context;
    
        #[tokio::test]
        async fn test_create_audit_log() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
            let ctx = setup_test_context().await?;
            let audit_log_repo = &ctx.audit_repos().audit_log_repository;
    
            let audit_log_model = create_test_audit_log();
        let result = audit_log_repo.create(&audit_log_model).await;

        assert!(result.is_ok());
        let created = result.unwrap();
        assert_eq!(created.id, audit_log_model.id);

        Ok(())
    }
}