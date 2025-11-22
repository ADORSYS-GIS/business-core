use business_core_db::repository_error::RepositoryError;
use uuid::Uuid;

use super::repo_impl::FeeTypeGlMappingRepositoryImpl;

impl FeeTypeGlMappingRepositoryImpl {
    pub async fn exist_by_ids_internal(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<Uuid>, RepositoryError> {
        let mut tx = self.executor.tx.lock().await;
        let transaction = tx
            .as_mut()
            .ok_or_else(|| RepositoryError::TransactionConsumed)?;
        let res = sqlx::query_scalar(
            r#"
            SELECT id FROM fee_type_gl_mapping WHERE id = ANY($1)
            "#,
        )
        .bind(ids)
        .fetch_all(&mut **transaction)
        .await
        .map_err(|e| e.into());

        res
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{repository::db_init::DbInitializer, test_helper::TestHelper};
    use business_core_db::models::product::fee_type_gl_mapping::FeeType;
    use uuid::Uuid;

    use super::*;

    #[tokio::test]
    async fn test_exist_by_ids() -> Result<(), Box<dyn Error + Send + Sync>> {
        let db = DbInitializer::init_test_db().await?;
        let helper = TestHelper::new(db.clone());
        let audit_log = helper.create_audit_log().await?;

        let mut items = vec![];
        for i in 0..5 {
            let item = business_core_db::models::product::fee_type_gl_mapping::FeeTypeGlMappingModel {
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
        let existing_ids = repo.exist_by_ids(&ids).await?;

        assert_eq!(existing_ids.len(), 5);

        Ok(())
    }
}