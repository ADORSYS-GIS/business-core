use async_trait::async_trait;
use business_core_db::models::reason_and_purpose::reason::ReasonModel;
use business_core_db::repository::create_batch::CreateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::models::index_aware::IndexAware;

use super::repo_impl::ReasonRepositoryImpl;

impl ReasonRepositoryImpl {
    pub(super) async fn create_batch_impl(
        repo: &ReasonRepositoryImpl,
        items: Vec<ReasonModel>,
    ) -> Result<Vec<ReasonModel>, Box<dyn Error + Send + Sync>> {
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
                    INSERT INTO reason (
                        id, code, category, context,
                        l1_content, l2_content, l3_content,
                        l1_language_code, l2_language_code, l3_language_code,
                        requires_details, is_active, severity, display_order,
                        compliance_metadata
                    )
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
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

                // Insert into index table
                let idx = item.to_index();
                sqlx::query(
                    r#"
                    INSERT INTO reason_idx (
                        id, code_hash, category_hash, context_hash, compliance_metadata
                    )
                    VALUES ($1, $2, $3, $4, $5)
                    "#,
                )
                .bind(idx.id)
                .bind(idx.code_hash)
                .bind(idx.category_hash)
                .bind(idx.context_hash)
                .bind(idx.compliance_metadata)
                .execute(&mut **transaction)
                .await?;

                indices.push(idx);
                saved_items.push(item);
            }
        } // Transaction lock released here
        
        // Update cache after releasing transaction lock
        {
            let cache = repo.reason_idx_cache.read().await;
            for idx in indices {
                cache.add(idx);
            }
        }

        Ok(saved_items)
    }
}

#[async_trait]
impl CreateBatch<Postgres, ReasonModel> for ReasonRepositoryImpl {
    async fn create_batch(
        &self,
        items: Vec<ReasonModel>,
        _audit_log_id: Option<Uuid>,
    ) -> Result<Vec<ReasonModel>, Box<dyn Error + Send + Sync>> {
        Self::create_batch_impl(self, items).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::{setup_test_context, setup_test_context_and_listen};
    use business_core_db::models::index_aware::IndexAware;
    use business_core_db::repository::create_batch::CreateBatch;
    use tokio::time::{sleep, Duration};
    use super::super::test_utils::test_utils::create_test_reason;

    #[tokio::test]
    async fn test_create_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let reason_repo = &ctx.reason_and_purpose_repos().reason_repository;

        let mut reasons = Vec::new();
        for i in 0..5 {
            let reason = create_test_reason(
                &format!("TEST_CODE_{i}"),
                &format!("Test Reason {i}"),
            );
            reasons.push(reason);
        }

        let saved_reasons = reason_repo.create_batch(reasons.clone(), None).await?;

        assert_eq!(saved_reasons.len(), 5);

        for saved_reason in &saved_reasons {
            assert!(saved_reason.code.as_str().starts_with("TEST_CODE_"));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let reason_repo = &ctx.reason_and_purpose_repos().reason_repository;

        let saved_reasons = reason_repo.create_batch(Vec::new(), None).await?;

        assert_eq!(saved_reasons.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_reason_insert_triggers_cache_notification() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        
        // Setup test context with the handler
        let ctx = setup_test_context_and_listen().await?;
        let pool = ctx.pool();

        // Create a test reason with a unique code to avoid conflicts
        let unique_code = {
            let uuid = uuid::Uuid::new_v4();
            format!("TEST_{uuid}")
        };
        let test_reason = create_test_reason(&unique_code, "Test Reason");
        let reason_idx = test_reason.to_index();
    
        // Give listener more time to start and establish connection
        sleep(Duration::from_millis(2000)).await;
    
        // First insert the reason record
        sqlx::query(
            r#"
            INSERT INTO reason (
                id, code, category, context,
                l1_content, l2_content, l3_content,
                l1_language_code, l2_language_code, l3_language_code,
                requires_details, is_active, severity, display_order,
                compliance_metadata
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
            "#
        )
        .bind(test_reason.id)
        .bind(test_reason.code.as_str())
        .bind(test_reason.category)
        .bind(test_reason.context)
        .bind(test_reason.l1_content.as_ref().map(|s| s.as_str()))
        .bind(test_reason.l2_content.as_ref().map(|s| s.as_str()))
        .bind(test_reason.l3_content.as_ref().map(|s| s.as_str()))
        .bind(test_reason.l1_language_code.as_ref().map(|s| s.as_str()))
        .bind(test_reason.l2_language_code.as_ref().map(|s| s.as_str()))
        .bind(test_reason.l3_language_code.as_ref().map(|s| s.as_str()))
        .bind(test_reason.requires_details)
        .bind(test_reason.is_active)
        .bind(test_reason.severity)
        .bind(test_reason.display_order)
        .bind(test_reason.compliance_metadata)
        .execute(&**pool)
        .await
        .expect("Failed to insert reason");
    
        // Then insert the reason index directly into the database using raw SQL
        sqlx::query("INSERT INTO reason_idx (id, code_hash, category_hash, context_hash, compliance_metadata) VALUES ($1, $2, $3, $4, $5)")
            .bind(reason_idx.id)
            .bind(reason_idx.code_hash)
            .bind(reason_idx.category_hash)
            .bind(reason_idx.context_hash)
            .bind(reason_idx.compliance_metadata)
            .execute(&**pool)
            .await
            .expect("Failed to insert reason index");

        // Give more time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        let reason_repo = &ctx.reason_and_purpose_repos().reason_repository;

        // Verify the cache was updated via the trigger
        let cache = reason_repo.reason_idx_cache.read().await;
        assert!(
            cache.contains_primary(&reason_idx.id),
            "Reason should be in cache after insert"
        );
    
        let cached_reason = cache.get_by_primary(&reason_idx.id);
        assert!(cached_reason.is_some(), "Reason should be retrievable from cache");
        
        // Verify the cached data matches
        let cached_reason = cached_reason.unwrap();
        assert_eq!(cached_reason.id, reason_idx.id);
        assert_eq!(cached_reason.code_hash, reason_idx.code_hash);
        assert_eq!(cached_reason.category_hash, reason_idx.category_hash);
        assert_eq!(cached_reason.context_hash, reason_idx.context_hash);
        
        // Drop the read lock before proceeding to allow notification handler to process
        drop(cache);

        // Delete the records from the database, will cascade delete reason_idx
        sqlx::query("DELETE FROM reason WHERE id = $1")
            .bind(reason_idx.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete reason");

        // Give more time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        // Verify the cache entry was removed
        let cache = reason_repo.reason_idx_cache.read().await;
        assert!(
            !cache.contains_primary(&reason_idx.id),
            "Reason should be removed from cache after delete"
        );
        
        Ok(())
    }
}