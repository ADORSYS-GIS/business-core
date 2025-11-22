use business_core_db::repository_error::RepositoryError;
use uuid::Uuid;

use super::repo_impl::AccountGlMappingRepositoryImpl;

impl AccountGlMappingRepositoryImpl {
    pub async fn exist_by_ids_internal(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<Uuid>, RepositoryError> {
        let mut tx = self.executor.tx.lock().await;
        let transaction = tx
            .as_mut()
            .ok_or_else(|| RepositoryError::new("Transaction has been consumed"))?;

        let result = sqlx::query_scalar::<_, Uuid>(
            "SELECT id FROM account_gl_mapping WHERE id = ANY($1)",
        )
        .bind(ids)
        .fetch_all(&mut **transaction)
        .await
        .map_err(|e| RepositoryError::new(&e.to_string()))?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{repository::db_init::DbInitializer, test_helper::TestHelper};

    use super::*;

    #[tokio::test]
    async fn test_exist_by_ids() -> Result<(), Box<dyn Error + Send + Sync>> {
        let db = DbInitializer::init_test_db().await?;
        let helper = TestHelper::new(db.clone());
        let audit_log = helper.create_audit_log().await?;
        let repo = AccountGlMappingRepositoryImpl::new(db.clone());

        let mut items = vec![];
        for i in 0..5 {
            let item = super::super::test_utils::create_test_account_gl_mapping(
                &format!("CUST{}", i),
                Some(&format!("OD{}", i)),
            );
            items.push(item);
        }
        let saved_items = repo.create_batch(items, Some(audit_log.id)).await?;
        let ids: Vec<Uuid> = saved_items.iter().map(|item| item.id).collect();

        let existing_ids = repo.exist_by_ids(&ids).await?;
        assert_eq!(existing_ids.len(), 5);

        Ok(())
    }
}