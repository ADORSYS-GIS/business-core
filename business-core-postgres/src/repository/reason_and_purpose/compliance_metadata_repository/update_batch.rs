use async_trait::async_trait;
use business_core_db::models::reason_and_purpose::compliance_metadata::ComplianceMetadataModel;
use business_core_db::models::index_aware::IndexAware;
use business_core_db::repository::update_batch::UpdateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::ComplianceMetadataRepositoryImpl;

impl ComplianceMetadataRepositoryImpl {
    pub(super) async fn update_batch_impl(
        &self,
        items: Vec<ComplianceMetadataModel>,
    ) -> Result<Vec<ComplianceMetadataModel>, Box<dyn Error + Send + Sync>> {
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
                    UPDATE compliance_metadata
                    SET regulatory_code = $2, reportable = $3, requires_sar = $4, requires_ctr = $5,
                        retention_years = $6, escalation_required = $7, risk_score_impact = $8, no_tipping_off = $9,
                        jurisdictions1 = $10, jurisdictions2 = $11, jurisdictions3 = $12, jurisdictions4 = $13, jurisdictions5 = $14
                    WHERE id = $1
                    "#,
                )
                .bind(item.id)
                .bind(item.regulatory_code.as_ref().map(|s| s.as_str()))
                .bind(item.reportable)
                .bind(item.requires_sar)
                .bind(item.requires_ctr)
                .bind(item.retention_years)
                .bind(item.escalation_required)
                .bind(item.risk_score_impact)
                .bind(item.no_tipping_off)
                .bind(item.jurisdictions1.as_str())
                .bind(item.jurisdictions2.as_str())
                .bind(item.jurisdictions3.as_str())
                .bind(item.jurisdictions4.as_str())
                .bind(item.jurisdictions5.as_str())
                .execute(&mut **transaction)
                .await?;
                
                indices.push((item.id, item.to_index()));
                updated_items.push(item);
            }
        } // Transaction lock released here
        
        // Update cache after releasing transaction lock
        {
            let cache = self.compliance_metadata_idx_cache.read().await;
            for (id, idx) in indices {
                cache.remove(&id);
                cache.add(idx);
            }
        }

        Ok(updated_items)
    }
}

#[async_trait]
impl UpdateBatch<Postgres, ComplianceMetadataModel> for ComplianceMetadataRepositoryImpl {
    async fn update_batch(
        &self,
        items: Vec<ComplianceMetadataModel>,
        _audit_log_id: Option<Uuid>,
    ) -> Result<Vec<ComplianceMetadataModel>, Box<dyn Error + Send + Sync>> {
        Self::update_batch_impl(self, items).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::update_batch::UpdateBatch;
    use heapless::String as HeaplessString;
    use super::super::test_utils::test_utils::create_test_compliance_metadata;

    #[tokio::test]
    async fn test_update_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let compliance_metadata_repo = &ctx.reason_and_purpose_repos().compliance_metadata_repository;

        let mut metadata_items = Vec::new();
        for i in 0..3 {
            let metadata = create_test_compliance_metadata(
                Some(&format!("UPDATE-{}", i)),
                true,
                false,
            );
            metadata_items.push(metadata);
        }

        let saved_items = compliance_metadata_repo.create_batch(metadata_items.clone(), None).await?;

        // Update the items
        let mut updated_items = Vec::new();
        for item in saved_items {
            let mut updated = item.clone();
            updated.reportable = false;
            updated.requires_sar = true;
            updated.regulatory_code = Some(HeaplessString::try_from("UPDATED").unwrap());
            updated_items.push(updated);
        }

        let result = compliance_metadata_repo.update_batch(updated_items.clone(), None).await?;
        assert_eq!(result.len(), 3);

        for updated_item in result {
            assert_eq!(updated_item.reportable, false);
            assert_eq!(updated_item.requires_sar, true);
            assert_eq!(updated_item.regulatory_code.as_ref().unwrap().as_str(), "UPDATED");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_update_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let compliance_metadata_repo = &ctx.reason_and_purpose_repos().compliance_metadata_repository;

        let result = compliance_metadata_repo.update_batch(Vec::new(), None).await?;

        assert_eq!(result.len(), 0);

        Ok(())
    }
}