use business_core_db::models::audit::AuditLinkModel;
use super::repo_impl::AuditLinkRepositoryImpl;

impl AuditLinkRepositoryImpl {
    pub(super) async fn create_impl(
        repo: &AuditLinkRepositoryImpl,
        audit_link: &AuditLinkModel,
    ) -> Result<(), sqlx::Error> {
        let query = sqlx::query(
            r#"
            INSERT INTO audit_link (audit_log_id, entity_id, entity_type)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(audit_link.audit_log_id)
        .bind(audit_link.entity_id)
        .bind(audit_link.entity_type);

        let mut tx = repo.executor.tx.lock().await;
        if let Some(transaction) = tx.as_mut() {
            query.execute(&mut **transaction).await?;
        } else {
            return Err(sqlx::Error::Configuration("Transaction has been consumed".into()));
        }

        Ok(())
    }
}