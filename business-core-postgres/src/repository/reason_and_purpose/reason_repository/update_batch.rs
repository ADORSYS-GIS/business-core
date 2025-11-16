use async_trait::async_trait;
use business_core_db::models::reason_and_purpose::reason::ReasonModel;
use business_core_db::models::index_aware::IndexAware;
use business_core_db::repository::update_batch::UpdateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::ReasonRepositoryImpl;

impl ReasonRepositoryImpl {
    pub(super) async fn update_batch_impl(
        &self,
        items: Vec<ReasonModel>,
    ) -> Result<Vec<ReasonModel>, Box<dyn Error + Send + Sync>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut updated_items = Vec::new();
        let mut indices = Vec::new();
        
        // Acquire lock once and do all database operations
        {
            let mut tx = self.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
            
            for item in items {
                // Execute update
                sqlx::query(
                    r#"
                    UPDATE reason
                    SET code = $2, category = $3, context = $4,
                        l1_content = $5, l2_content = $6, l3_content = $7,
                        l1_language_code = $8, l2_language_code = $9, l3_language_code = $10,
                        requires_details = $11, is_active = $12, severity = $13,
                        display_order = $14, compliance_metadata = $15
                    WHERE id = $1
                    "#,
                )
                .bind(item.id)
                .bind(item.code.as_str())
                .bind(item.category)
                .bind(item.context)
                .bind(item.l1_content.as_ref().map(|s| s.as_str()))
                .bind(item.l2_content.as_ref().map(|s| s.as_str()))
                .bind(item.l3_content.as_ref().map(|s| s.as_str()))
                .bind(item.l1_language_code.as_ref().map(|s| s.as_str()))
                .bind(item.l2_language_code.as_ref().map(|s| s.as_str()))
                .bind(item.l3_language_code.as_ref().map(|s| s.as_str()))
                .bind(item.requires_details)
                .bind(item.is_active)
                .bind(item.severity)
                .bind(item.display_order)
                .bind(item.compliance_metadata)
                .execute(&mut **transaction)
                .await?;

                // Update index table
                let idx = item.to_index();
                sqlx::query(
                    r#"
                    UPDATE reason_idx
                    SET code_hash = $2, category_hash = $3, context_hash = $4, compliance_metadata = $5
                    WHERE id = $1
                    "#,
                )
                .bind(idx.id)
                .bind(idx.code_hash)
                .bind(idx.category_hash)
                .bind(idx.context_hash)
                .bind(idx.compliance_metadata)
                .execute(&mut **transaction)
                .await?;

                indices.push((item.id, idx));
                updated_items.push(item);
            }
        } // Transaction lock released here
        
        // Update cache after releasing transaction lock
        {
            let cache = self.reason_idx_cache.read().await;
            for (id, idx) in indices {
                cache.remove(&id);
                cache.add(idx);
            }
        }

        Ok(updated_items)
    }
}

#[async_trait]
impl UpdateBatch<Postgres, ReasonModel> for ReasonRepositoryImpl {
    async fn update_batch(
        &self,
        items: Vec<ReasonModel>,
        _audit_log_id: Option<Uuid>,
    ) -> Result<Vec<ReasonModel>, Box<dyn Error + Send + Sync>> {
        Self::update_batch_impl(self, items).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::update_batch::UpdateBatch;
    use super::super::test_utils::test_utils::create_test_reason;

    #[tokio::test]
    async fn test_update_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let reason_repo = &ctx.reason_and_purpose_repos().reason_repository;

        let mut reasons = Vec::new();
        for i in 0..3 {
            reasons.push(create_test_reason(
                &format!("UPDATE_TEST_{}", i),
                &format!("Update Test Reason {}", i),
            ));
        }

        let saved = reason_repo.create_batch(reasons.clone(), None).await?;
        
        let mut updated_reasons = Vec::new();
        for mut reason in saved {
            reason.display_order = 999;
            updated_reasons.push(reason);
        }

        let updated = reason_repo.update_batch(updated_reasons, None).await?;

        assert_eq!(updated.len(), 3);
        for reason in updated {
            assert_eq!(reason.display_order, 999);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_update_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let reason_repo = &ctx.reason_and_purpose_repos().reason_repository;

        let updated = reason_repo.update_batch(Vec::new(), None).await?;

        assert_eq!(updated.len(), 0);

        Ok(())
    }
}