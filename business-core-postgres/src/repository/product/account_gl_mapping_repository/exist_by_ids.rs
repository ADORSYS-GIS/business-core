use async_trait::async_trait;
use business_core_db::repository::exist_by_ids::ExistByIds;
use std::error::Error;
use uuid::Uuid;
use crate::repository::product::account_gl_mapping_repository::repo_impl::AccountGlMappingRepositoryImpl;

#[async_trait]
impl ExistByIds for AccountGlMappingRepositoryImpl {
    async fn exist_by_ids(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<Uuid>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }

        let query = "SELECT id FROM account_gl_mapping WHERE id = ANY($1)";
        let rows = {
            let mut tx = self.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
            sqlx::query(query).bind(ids).fetch_all(&mut **transaction).await?
        };

        let existing_ids = rows.into_iter().map(|row| sqlx::Row::get(&row, "id")).collect();
        Ok(existing_ids)
    }
}