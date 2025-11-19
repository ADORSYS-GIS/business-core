use async_trait::async_trait;
use business_core_db::models::reason_and_purpose::compliance_metadata::ComplianceMetadataModel;
use business_core_db::repository::load_batch::LoadBatch;
use crate::utils::TryFromRow;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::ComplianceMetadataRepositoryImpl;

impl ComplianceMetadataRepositoryImpl {
    pub(super) async fn load_batch_impl(
        repo: &ComplianceMetadataRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<Option<ComplianceMetadataModel>>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        
        let query = r#"SELECT * FROM compliance_metadata WHERE id = ANY($1)"#;
        let rows = {
            let mut tx = repo.executor.tx.lock().await;
            if let Some(transaction) = tx.as_mut() {
                sqlx::query(query).bind(ids).fetch_all(&mut **transaction).await?
            } else {
                return Err("Transaction has been consumed".into());
            }
        };
        
        let mut item_map = std::collections::HashMap::new();
        for row in rows {
            let item = ComplianceMetadataModel::try_from_row(&row)?;
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
impl LoadBatch<Postgres, ComplianceMetadataModel> for ComplianceMetadataRepositoryImpl {
    async fn load_batch(&self, ids: &[Uuid]) -> Result<Vec<Option<ComplianceMetadataModel>>, Box<dyn Error + Send + Sync>> {
        Self::load_batch_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::load_batch::LoadBatch;
    use uuid::Uuid;
    use super::super::test_utils::test_utils::create_test_compliance_metadata;

    #[tokio::test]
    async fn test_load_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let compliance_metadata_repo = &ctx.reason_and_purpose_repos().compliance_metadata_repository;

        let mut metadata_items = Vec::new();
        for i in 0..5 {
            let metadata = create_test_compliance_metadata(
                Some(&format!("LOAD-{i}")),
                true,
                false,
            );
            metadata_items.push(metadata);
        }

        let saved_items = compliance_metadata_repo.create_batch(metadata_items.clone(), None).await?;
        let ids: Vec<Uuid> = saved_items.iter().map(|m| m.id).collect();

        let loaded_items = compliance_metadata_repo.load_batch(&ids).await?;
        assert_eq!(loaded_items.len(), 5);
        
        for item_opt in loaded_items {
            assert!(item_opt.is_some());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_load_batch_with_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let compliance_metadata_repo = &ctx.reason_and_purpose_repos().compliance_metadata_repository;

        let metadata = create_test_compliance_metadata(Some("LOAD-TEST"), true, false);
        let saved = compliance_metadata_repo.create_batch(vec![metadata.clone()], None).await?;

        let non_existent_id = Uuid::new_v4();
        let ids = vec![saved[0].id, non_existent_id];

        let loaded_items = compliance_metadata_repo.load_batch(&ids).await?;
        assert_eq!(loaded_items.len(), 2);
        assert!(loaded_items[0].is_some());
        assert!(loaded_items[1].is_none());

        Ok(())
    }
}