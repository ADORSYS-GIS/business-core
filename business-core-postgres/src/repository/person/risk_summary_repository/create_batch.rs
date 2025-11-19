use async_trait::async_trait;
use business_core_db::models::person::risk_summary::RiskSummaryModel;
use business_core_db::repository::create_batch::CreateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::models::index_aware::IndexAware;

use super::repo_impl::RiskSummaryRepositoryImpl;

impl RiskSummaryRepositoryImpl {
    pub(super) async fn create_batch_impl(
        repo: &RiskSummaryRepositoryImpl,
        items: Vec<RiskSummaryModel>,
    ) -> Result<Vec<RiskSummaryModel>, Box<dyn Error + Send + Sync>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut saved_items = Vec::new();
        let mut indices = Vec::new();
        
        // Acquire lock once and do all database operations
        let mut tx = repo.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        
        for item in items {
            // Execute main insert
            sqlx::query(
                r#"
                INSERT INTO risk_summary (id, person_id, current_rating, last_assessment_date, flags_01, flags_02, flags_03, flags_04, flags_05)
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                "#,
            )
            .bind(item.id)
            .bind(item.person_id)
            .bind(item.current_rating)
            .bind(item.last_assessment_date)
            .bind(item.flags_01.as_str())
            .bind(item.flags_02.as_str())
            .bind(item.flags_03.as_str())
            .bind(item.flags_04.as_str())
            .bind(item.flags_05.as_str())
            .execute(&mut **transaction)
            .await?;

            // Insert into index table
            let idx = item.to_index();
            sqlx::query(
                r#"
                INSERT INTO risk_summary_idx (id, person_id)
                VALUES ($1, $2)
                "#,
            )
            .bind(idx.id)
            .bind(idx.person_id)
            .execute(&mut **transaction)
            .await?;

            indices.push(idx);
            saved_items.push(item);
        }
        
        // Release transaction lock before updating cache
        drop(tx);
        
        // Update cache after releasing transaction lock
        {
            let cache = repo.risk_summary_idx_cache.read().await;
            for idx in indices {
                cache.add(idx);
            }
        }

        Ok(saved_items)
    }
}

#[async_trait]
impl CreateBatch<Postgres, RiskSummaryModel> for RiskSummaryRepositoryImpl {
    async fn create_batch(
        &self,
        items: Vec<RiskSummaryModel>,
        _audit_log_id: Option<Uuid>,
    ) -> Result<Vec<RiskSummaryModel>, Box<dyn Error + Send + Sync>> {
        Self::create_batch_impl(self, items).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::{setup_test_context, setup_test_context_and_listen};
    use business_core_db::models::index_aware::IndexAware;
    use business_core_db::repository::create_batch::CreateBatch;
    use tokio::time::{sleep, Duration};
    use super::super::test_utils::test_utils::{create_test_risk_summary, create_test_person};
    use crate::repository::person::test_utils::create_test_audit_log;

    #[tokio::test]
    async fn test_create_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;
        let risk_summary_repo = &ctx.person_repos().risk_summary_repository;

        // Create audit log
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create person
        let person = create_test_person();
        person_repo.create_batch(vec![person.clone()], Some(audit_log.id)).await?;

        // Create risk summaries
        let risk_summary1 = create_test_risk_summary(person.id);
        let risk_summary2 = create_test_risk_summary(person.id);

        let saved = risk_summary_repo.create_batch(vec![risk_summary1.clone(), risk_summary2.clone()], Some(audit_log.id)).await?;

        assert_eq!(saved.len(), 2);
        assert_eq!(saved[0].id, risk_summary1.id);
        assert_eq!(saved[1].id, risk_summary2.id);

        Ok(())
    }

    #[tokio::test]
    async fn test_create_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let risk_summary_repo = &ctx.person_repos().risk_summary_repository;

        let saved = risk_summary_repo.create_batch(vec![], None).await?;

        assert_eq!(saved.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_risk_summary_insert_triggers_cache_notification() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        
        // Setup test context with the notification listener
        let ctx = setup_test_context_and_listen().await?;
        let pool = ctx.pool();

        // Create a test person first
        let person = create_test_person();
        sqlx::query("INSERT INTO person (id, person_type, risk_rating, status, display_name, external_identifier, id_type, id_number, entity_reference_count, organization_person_id, messaging_info1, messaging_info2, messaging_info3, messaging_info4, messaging_info5, department, location_id, duplicate_of_person_id, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22)")
            .bind(person.id)
            .bind(person.person_type)
            .bind(person.risk_rating)
            .bind(person.status)
            .bind(person.display_name.as_str())
            .bind(person.external_identifier.as_ref().map(|s| s.as_str()))
            .bind(person.id_type)
            .bind(person.id_number.as_str())
            .bind(person.entity_reference_count)
            .bind(person.organization_person_id)
            .bind(person.messaging_info1.as_ref().map(|s| s.as_str()))
            .bind(person.messaging_info2.as_ref().map(|s| s.as_str()))
            .bind(person.messaging_info3.as_ref().map(|s| s.as_str()))
            .bind(person.messaging_info4.as_ref().map(|s| s.as_str()))
            .bind(person.messaging_info5.as_ref().map(|s| s.as_str()))
            .bind(person.department.as_ref().map(|s| s.as_str()))
            .bind(person.location_id)
            .bind(person.duplicate_of_person_id)
            .bind(person.antecedent_hash)
            .bind(person.antecedent_audit_log_id)
            .bind(person.hash)
            .bind(person.audit_log_id)
            .execute(&**pool)
            .await
            .expect("Failed to insert person");

        // Create a test risk summary
        let test_risk_summary = create_test_risk_summary(person.id);
        let risk_summary_idx = test_risk_summary.to_index();
    
        // Give listener time to start and establish connection
        sleep(Duration::from_millis(2000)).await;
    
        // Insert the risk summary record directly into database
        sqlx::query("INSERT INTO risk_summary (id, person_id, current_rating, last_assessment_date, flags_01, flags_02, flags_03, flags_04, flags_05) VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)")
            .bind(test_risk_summary.id)
            .bind(test_risk_summary.person_id)
            .bind(test_risk_summary.current_rating)
            .bind(test_risk_summary.last_assessment_date)
            .bind(test_risk_summary.flags_01.as_str())
            .bind(test_risk_summary.flags_02.as_str())
            .bind(test_risk_summary.flags_03.as_str())
            .bind(test_risk_summary.flags_04.as_str())
            .bind(test_risk_summary.flags_05.as_str())
            .execute(&**pool)
            .await
            .expect("Failed to insert risk_summary");
    
        // Insert the index record directly into database (triggers notification)
        sqlx::query("INSERT INTO risk_summary_idx (id, person_id) VALUES ($1, $2)")
            .bind(risk_summary_idx.id)
            .bind(risk_summary_idx.person_id)
            .execute(&**pool)
            .await
            .expect("Failed to insert risk_summary index");

        // Give time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        let risk_summary_repo = &ctx.person_repos().risk_summary_repository;

        // Verify the cache was updated via the trigger
        let cache = risk_summary_repo.risk_summary_idx_cache.read().await;
        assert!(
            cache.contains_primary(&risk_summary_idx.id),
            "RiskSummary should be in cache after insert"
        );
    
        let cached_risk_summary = cache.get_by_primary(&risk_summary_idx.id);
        assert!(cached_risk_summary.is_some(), "RiskSummary should be retrievable from cache");
        
        // Verify the cached data matches
        let cached_risk_summary = cached_risk_summary.unwrap();
        assert_eq!(cached_risk_summary.id, risk_summary_idx.id);
        assert_eq!(cached_risk_summary.person_id, risk_summary_idx.person_id);
        
        // Drop the read lock before proceeding
        drop(cache);

        // Delete the record from database (triggers notification)
        sqlx::query("DELETE FROM risk_summary WHERE id = $1")
            .bind(risk_summary_idx.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete risk_summary");

        // Give time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        // Verify the cache entry was removed
        let cache = risk_summary_repo.risk_summary_idx_cache.read().await;
        assert!(
            !cache.contains_primary(&risk_summary_idx.id),
            "RiskSummary should be removed from cache after delete"
        );
        
        Ok(())
    }
}