use std::error::Error;

use business_core_db::models::product::account_gl_mapping::AccountGlMappingModel;
use uuid::Uuid;

use super::repo_impl::AccountGlMappingRepositoryImpl;

impl AccountGlMappingRepositoryImpl {
    pub async fn load_batch_internal(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<AccountGlMappingModel>, Box<dyn Error + Send + Sync>> {
        let mut tx = self.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        let result = sqlx::query_as::<_, AccountGlMappingModel>(
            "SELECT * FROM account_gl_mapping WHERE id = ANY($1)",
        )
        .bind(ids)
        .fetch_all(&mut **transaction)
        .await?;

        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{repository::db_init::DbInitializer, test_helper::TestHelper};

    use super::*;

    #[tokio::test]
    async fn test_load_batch() -> Result<(), Box<dyn Error + Send + Sync>> {
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

        let loaded_items = repo.load_batch(&ids).await?;
        assert_eq!(loaded_items.len(), 5);

        Ok(())
    }
}