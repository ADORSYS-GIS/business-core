use async_trait::async_trait;
use business_core_db::models::reason_and_purpose::reason::ReasonModel;
use business_core_db::repository::load_batch::LoadBatch;
use crate::utils::TryFromRow;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::ReasonRepositoryImpl;

impl ReasonRepositoryImpl {
    pub(super) async fn load_batch_impl(
        repo: &ReasonRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<Option<ReasonModel>>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        
        let query = r#"SELECT * FROM reason WHERE id = ANY($1)"#;
        let rows = {
            let mut tx = repo.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
            sqlx::query(query).bind(ids).fetch_all(&mut **transaction).await?
        };
        
        let mut item_map = std::collections::HashMap::new();
        for row in rows {
            let item = ReasonModel::try_from_row(&row)?;
            item_map.insert(item.id, item);
        }
        
        let mut result = Vec::with_capacity(ids.len());
        for id in ids {
            result.push(item_map.remove(id));
        }
        Ok(result)
    }
}

#[async_trait]
impl LoadBatch<Postgres, ReasonModel> for ReasonRepositoryImpl {
    async fn load_batch(&self, ids: &[Uuid]) -> Result<Vec<Option<ReasonModel>>, Box<dyn Error + Send + Sync>> {
        Self::load_batch_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::load_batch::LoadBatch;
    use super::super::test_utils::test_utils::create_test_reason;

    #[tokio::test]
    async fn test_load_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let reason_repo = &ctx.reason_and_purpose_repos().reason_repository;

        let mut reasons = Vec::new();
        for i in 0..3 {
            reasons.push(create_test_reason(
                &format!("LOAD_TEST_{i}"),
                &format!("Load Test Reason {i}"),
            ));
        }

        let saved = reason_repo.create_batch(reasons.clone(), None).await?;
        let ids: Vec<_> = saved.iter().map(|r| r.id).collect();

        let loaded = reason_repo.load_batch(&ids).await?;

        assert_eq!(loaded.len(), 3);
        for item in loaded {
            assert!(item.is_some());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_load_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let reason_repo = &ctx.reason_and_purpose_repos().reason_repository;

        let loaded = reason_repo.load_batch(&[]).await?;

        assert_eq!(loaded.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_load_batch_non_existent() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let reason_repo = &ctx.reason_and_purpose_repos().reason_repository;

        let non_existent_id = uuid::Uuid::new_v4();
        let loaded = reason_repo.load_batch(&[non_existent_id]).await?;

        assert_eq!(loaded.len(), 1);
        assert!(loaded[0].is_none());

        Ok(())
    }
}