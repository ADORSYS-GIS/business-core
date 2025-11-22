use std::error::Error;

use business_core_db::models::product::fee_type_gl_mapping::FeeTypeGlMappingModel;
use uuid::Uuid;

use super::repo_impl::FeeTypeGlMappingRepositoryImpl;

impl FeeTypeGlMappingRepositoryImpl {
    pub async fn load_batch_internal(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<FeeTypeGlMappingModel>, Box<dyn Error + Send + Sync>> {
        let mut tx = self.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        let res = sqlx::query_as::<_, FeeTypeGlMappingModel>(
            r#"
            SELECT * FROM fee_type_gl_mapping WHERE id = ANY($1)
            "#,
        )
        .bind(ids)
        .fetch_all(&mut **transaction)
        .await?;

        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{repository::db_init::DbInitializer, test_helper::TestHelper};
    use business_core_db::models::product::fee_type_gl_mapping::FeeType;

    use super::*;

    #[tokio::test]
    async fn test_load_batch() -> Result<(), Box<dyn Error + Send + Sync>> {
        let db = DbInitializer::init_test_db().await?;
        let helper = TestHelper::new(db.clone());
        let audit_log = helper.create_audit_log().await?;

        let mut items = vec![];
        for i in 0..5 {
            let item = FeeTypeGlMappingModel {
                id: Uuid::new_v4(),
                fee_type: FeeType::InterestExpense,
                gl_code: format!("GLCODE{}", i).try_into().unwrap(),
                antecedent_hash: 0,
                antecedent_audit_log_id: Uuid::nil(),
                hash: 0,
                audit_log_id: None,
            };
            items.push(item);
        }
        let repo = FeeTypeGlMappingRepositoryImpl::new(db.clone());
        let saved_items = repo.create_batch(items, Some(audit_log.id)).await?;
        let ids: Vec<Uuid> = saved_items.iter().map(|item| item.id).collect();
        let loaded_items = repo.load_batch(&ids).await?;

        assert_eq!(loaded_items.len(), 5);

        Ok(())
    }
}