use business_core_db::models::audit::AuditLinkModel;
use uuid::Uuid;
use super::repo_impl::AuditLinkRepositoryImpl;

impl AuditLinkRepositoryImpl {
    pub(super) async fn find_by_audit_log_id_impl(
        repo: &AuditLinkRepositoryImpl,
        audit_log_id: Uuid,
    ) -> Result<Vec<AuditLinkModel>, sqlx::Error> {
        let query = sqlx::query_as::<_, AuditLinkModel>(
            r#"
            SELECT audit_log_id, entity_id, entity_type
            FROM audit_link
            WHERE audit_log_id = $1
            "#,
        )
        .bind(audit_log_id);

        let mut tx = repo.executor.tx.lock().await;
        if let Some(transaction) = tx.as_mut() {
            query.fetch_all(&mut **transaction).await
        } else {
            Err(sqlx::Error::Configuration("Transaction has been consumed".into()))
        }
    }
}