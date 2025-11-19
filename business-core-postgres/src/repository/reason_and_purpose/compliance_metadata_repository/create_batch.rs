use async_trait::async_trait;
use business_core_db::models::reason_and_purpose::compliance_metadata::ComplianceMetadataModel;
use business_core_db::repository::create_batch::CreateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::models::index_aware::IndexAware;

use super::repo_impl::ComplianceMetadataRepositoryImpl;

impl ComplianceMetadataRepositoryImpl {
    pub(super) async fn create_batch_impl(
        repo: &ComplianceMetadataRepositoryImpl,
        items: Vec<ComplianceMetadataModel>,
    ) -> Result<Vec<ComplianceMetadataModel>, Box<dyn Error + Send + Sync>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut saved_items = Vec::new();
        let mut indices = Vec::new();
        
        // Acquire lock once and do all database operations
        {
            let mut tx = repo.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
            
            for item in items {
                // Execute main insert
                sqlx::query(
                    r#"
                    INSERT INTO compliance_metadata (
                        id, regulatory_code, reportable, requires_sar, requires_ctr,
                        retention_years, escalation_required, risk_score_impact, no_tipping_off,
                        jurisdictions1, jurisdictions2, jurisdictions3, jurisdictions4, jurisdictions5
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
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

                // Insert into index table
                let idx = item.to_index();
                sqlx::query(
                    r#"
                    INSERT INTO compliance_metadata_idx (id, regulatory_code_hash)
                    VALUES ($1, $2)
                    "#,
                )
                .bind(idx.id)
                .bind(idx.regulatory_code_hash)
                .execute(&mut **transaction)
                .await?;

                indices.push(idx);
                saved_items.push(item);
            }
        } // Transaction lock released here
        
        // Update cache after releasing transaction lock
        {
            let cache = repo.compliance_metadata_idx_cache.read().await;
            for idx in indices {
                cache.add(idx);
            }
        }

        Ok(saved_items)
    }
}

#[async_trait]
impl CreateBatch<Postgres, ComplianceMetadataModel> for ComplianceMetadataRepositoryImpl {
    async fn create_batch(
        &self,
        items: Vec<ComplianceMetadataModel>,
        _audit_log_id: Option<Uuid>,
    ) -> Result<Vec<ComplianceMetadataModel>, Box<dyn Error + Send + Sync>> {
        Self::create_batch_impl(self, items).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::{random, setup_test_context, setup_test_context_and_listen};
    use business_core_db::models::index_aware::IndexAware;
    use business_core_db::repository::create_batch::CreateBatch;
    use tokio::time::{sleep, Duration};
    use super::super::test_utils::test_utils::create_test_compliance_metadata;

    #[tokio::test]
    async fn test_create_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let compliance_metadata_repo = &ctx.reason_and_purpose_repos().compliance_metadata_repository;

        let mut metadata_items = Vec::new();
        for i in 0..5 {
            let metadata = create_test_compliance_metadata(
                Some(&format!("FATF-R.{i}")),
                true,
                false,
            );
            metadata_items.push(metadata);
        }

        let saved_items = compliance_metadata_repo.create_batch(metadata_items.clone(), None).await?;

        assert_eq!(saved_items.len(), 5);

        for saved_item in &saved_items {
            assert!(saved_item.regulatory_code.is_some());
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let compliance_metadata_repo = &ctx.reason_and_purpose_repos().compliance_metadata_repository;

        let saved_items = compliance_metadata_repo.create_batch(Vec::new(), None).await?;

        assert_eq!(saved_items.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_compliance_metadata_insert_triggers_cache_notification() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        
        // Setup test context with the handler
        let ctx = setup_test_context_and_listen().await?;
        let pool = ctx.pool();

        // Create a test compliance metadata with a unique regulatory code to avoid conflicts
        let unique_code = format!("TEST-{}", random(10));
        let test_metadata = create_test_compliance_metadata(Some(&unique_code), true, false);
        let metadata_idx = test_metadata.to_index();
    
        // Give listener more time to start and establish connection
        sleep(Duration::from_millis(2000)).await;
    
        // First insert the compliance metadata record
        sqlx::query(
            r#"INSERT INTO compliance_metadata (
                id, regulatory_code, reportable, requires_sar, requires_ctr,
                retention_years, escalation_required, risk_score_impact, no_tipping_off,
                jurisdictions1, jurisdictions2, jurisdictions3, jurisdictions4, jurisdictions5
            ) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)"#
        )
            .bind(test_metadata.id)
            .bind(test_metadata.regulatory_code.as_ref().map(|s| s.as_str()))
            .bind(test_metadata.reportable)
            .bind(test_metadata.requires_sar)
            .bind(test_metadata.requires_ctr)
            .bind(test_metadata.retention_years)
            .bind(test_metadata.escalation_required)
            .bind(test_metadata.risk_score_impact)
            .bind(test_metadata.no_tipping_off)
            .bind(test_metadata.jurisdictions1.as_str())
            .bind(test_metadata.jurisdictions2.as_str())
            .bind(test_metadata.jurisdictions3.as_str())
            .bind(test_metadata.jurisdictions4.as_str())
            .bind(test_metadata.jurisdictions5.as_str())
            .execute(&**pool)
            .await
            .expect("Failed to insert compliance_metadata");
    
        // Then insert the compliance metadata index directly into the database using raw SQL
        sqlx::query("INSERT INTO compliance_metadata_idx (id, regulatory_code_hash) VALUES ($1, $2)")
            .bind(metadata_idx.id)
            .bind(metadata_idx.regulatory_code_hash)
            .execute(&**pool)
            .await
            .expect("Failed to insert compliance_metadata index");

        // Give more time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        let compliance_metadata_repo = &ctx.reason_and_purpose_repos().compliance_metadata_repository;

        // Verify the cache was updated via the trigger
        let cache = compliance_metadata_repo.compliance_metadata_idx_cache.read().await;
        assert!(
            cache.contains_primary(&metadata_idx.id),
            "ComplianceMetadata should be in cache after insert"
        );
    
        let cached_metadata = cache.get_by_primary(&metadata_idx.id);
        assert!(cached_metadata.is_some(), "ComplianceMetadata should be retrievable from cache");
        
        // Verify the cached data matches
        let cached_metadata = cached_metadata.unwrap();
        assert_eq!(cached_metadata.id, metadata_idx.id);
        assert_eq!(cached_metadata.regulatory_code_hash, metadata_idx.regulatory_code_hash);
        
        // Drop the read lock before proceeding to allow notification handler to process
        drop(cache);

        // Delete the records from the database, will cascade delete compliance_metadata_idx
        sqlx::query("DELETE FROM compliance_metadata WHERE id = $1")
            .bind(metadata_idx.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete compliance_metadata");

        // Give more time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        // Verify the cache entry was removed
        let cache = compliance_metadata_repo.compliance_metadata_idx_cache.read().await;
        assert!(
            !cache.contains_primary(&metadata_idx.id),
            "ComplianceMetadata should be removed from cache after delete"
        );
        
        Ok(())
    }
}