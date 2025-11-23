use async_trait::async_trait;
use business_core_db::models::{
    audit::{AuditEntityType, AuditLinkModel},
};
use business_core_db::repository::{delete_batch::DeleteBatch, load_batch::LoadBatch};
use business_core_db::utils::hash_as_i64;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::InterestRateTierRepositoryImpl;

impl InterestRateTierRepositoryImpl {
    pub(super) async fn delete_batch_impl(
        repo: &InterestRateTierRepositoryImpl,
        ids: &[Uuid],
        audit_log_id: Option<Uuid>,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for InterestRateTierModel")?;
        if ids.is_empty() {
            return Ok(0);
        }

        // 1. Load the full entities to be deleted to get their final state for auditing.
        //    This is necessary because the delete operation only receives IDs.
        let entities_to_delete = repo.load_batch(ids).await?;
        let mut deleted_count = 0;

        // Acquire lock once and do all database operations
        {
            let mut tx = repo.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;

            for entity_opt in &entities_to_delete {
                if let Some(entity) = entity_opt {
                    // 2. Create a final audit record before deletion. This follows a similar
                    //    pattern to an update.
                    let mut final_audit_entity = entity.clone();

                    // Set antecedent fields from the entity's current state
                    final_audit_entity.antecedent_hash = entity.hash;
                    final_audit_entity.antecedent_audit_log_id = entity
                        .audit_log_id
                        .ok_or("Entity must have audit_log_id for deletion")?;

                    // Set the new audit_log_id for this final "delete" event
                    final_audit_entity.audit_log_id = Some(audit_log_id);
                    final_audit_entity.hash = 0; // Set to 0 before final hashing

                    // Compute the final hash for the audit record
                    let final_hash = hash_as_i64(&final_audit_entity)?;
                    final_audit_entity.hash = final_hash;

                // 3. Build the audit insert query for the final state snapshot
                sqlx::query(
                    r#"
                    INSERT INTO interest_rate_tier_audit
                    (id, name, minimum_balance, maximum_balance, interest_rate, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                    "#,
                )
                    .bind(final_audit_entity.id)
                    .bind(final_audit_entity.name)
                    .bind(final_audit_entity.minimum_balance)
                    .bind(final_audit_entity.maximum_balance)
                    .bind(final_audit_entity.interest_rate)
                    .bind(final_audit_entity.antecedent_hash)
                    .bind(final_audit_entity.antecedent_audit_log_id)
                    .bind(final_audit_entity.hash)
                    .bind(final_audit_entity.audit_log_id)
                .execute(&mut **transaction)
                .await?;

                // 4. Build the entity delete query. The corresponding index record
                //    will be deleted automatically via `ON DELETE CASCADE`.
                    // 4. Build the entity delete query. The corresponding index record
                    //    will be deleted automatically via `ON DELETE CASCADE`.
                    let result = sqlx::query(
                        r#"
                        DELETE FROM interest_rate_tier WHERE id = $1
                        "#,
                    )
                    .bind(entity.id)
                    .execute(&mut **transaction)
                    .await?;

                    deleted_count += result.rows_affected() as usize;

                    // Create audit link for the deleted entity
                    let audit_link = AuditLinkModel {
                        audit_log_id,
                        entity_id: entity.id,
                        entity_type: AuditEntityType::InterestRateTier,
                    };
                    sqlx::query(
                        r#"
                        INSERT INTO audit_link (audit_log_id, entity_id, entity_type)
                        VALUES ($1, $2, $3)
                        "#,
                    )
                    .bind(audit_link.audit_log_id)
                    .bind(audit_link.entity_id)
                    .bind(audit_link.entity_type)
                    .execute(&mut **transaction)
                    .await?;
                }
            }
        } // Transaction lock released here

        // 6. Remove the entities from the cache
        {
            let cache = repo.interest_rate_tier_idx_cache.write().await;
            for entity_opt in &entities_to_delete {
                if let Some(entity) = entity_opt {
                    cache.remove(&entity.id);
                }
            }
        }

        Ok(deleted_count)
    }
}

#[async_trait]
impl DeleteBatch<Postgres> for InterestRateTierRepositoryImpl {
    async fn delete_batch(
        &self,
        ids: &[Uuid],
        audit_log_id: Option<Uuid>,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Self::delete_batch_impl(self, ids, audit_log_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::product::interest_rate_tier_repository::test_utils::create_test_interest_rate_tier;
    use crate::test_helper::setup_test_context;
    use business_core_db::{
        repository::{create_batch::CreateBatch, delete_batch::DeleteBatch},
    };
    use crate::repository::person::test_utils::create_test_audit_log;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_delete_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let interest_rate_tier_repo = &ctx.product_repos().interest_rate_tier_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let mut interest_rate_tiers = Vec::new();
        for _ in 0..3 {
            let interest_rate_tier = create_test_interest_rate_tier();
            interest_rate_tiers.push(interest_rate_tier);
        }

        let saved = interest_rate_tier_repo
            .create_batch(interest_rate_tiers, Some(audit_log.id))
            .await?;

        let ids: Vec<Uuid> = saved.iter().map(|s| s.id).collect();
        
        let delete_audit_log = create_test_audit_log();
        audit_log_repo.create(&delete_audit_log).await?;
        let deleted_count = interest_rate_tier_repo
            .delete_batch(&ids, Some(delete_audit_log.id))
            .await?;

        assert_eq!(deleted_count, 3);

        Ok(())
    }
}