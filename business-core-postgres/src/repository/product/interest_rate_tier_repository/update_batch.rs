use async_trait::async_trait;
use business_core_db::models::{
    audit::{AuditEntityType, AuditLinkModel},
    index_aware::IndexAware,
    product::interest_rate_tier::InterestRateTierModel,
};
use business_core_db::repository::update_batch::UpdateBatch;
use business_core_db::utils::hash_as_i64;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::InterestRateTierRepositoryImpl;

impl InterestRateTierRepositoryImpl {
    pub(super) async fn update_batch_impl(
        repo: &InterestRateTierRepositoryImpl,
        items: Vec<InterestRateTierModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<InterestRateTierModel>, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for InterestRateTierModel")?;
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut saved_items = Vec::new();
        let mut indices_to_remove = Vec::new();
        let mut indices_to_add = Vec::new();

        // Acquire lock once and do all database operations
        {
            let mut tx = repo.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;

            for mut item in items {
                // 1. Save current hash and audit_log_id for antecedent tracking
                let previous_hash = item.hash;
                let previous_audit_log_id = item
                    .audit_log_id
                    .ok_or("Entity must have audit_log_id for update")?;

                // 2. Check if entity has actually changed by recomputing hash
                let mut entity_for_hashing = item.clone();
                entity_for_hashing.hash = 0;

                // Compute hash of entity_for_hashing
                let computed_hash = hash_as_i64(&entity_for_hashing)?;

                // 3. Only proceed with update if entity has changed
                if computed_hash == previous_hash {
                    // No changes detected, return entity as-is
                    saved_items.push(item);
                    continue;
                }

                // The antecedent hash and audit log ID are now part of the entity itself.
                item.antecedent_hash = previous_hash;
                item.antecedent_audit_log_id = previous_audit_log_id;

                // 4. Entity has changed, update with new hash and audit_log_id
                // The hash used for the change check is not the final hash.
                // The entity must be re-hashed with the antecedent fields and new audit_log_id.
                item.audit_log_id = Some(audit_log_id);
                item.hash = 0; // Set to 0 before final hashing

                // Compute final hash for storage
                let new_computed_hash = hash_as_i64(&item)?;
                item.hash = new_computed_hash;

                // 5. Build audit insert query (includes all entity fields)
                sqlx::query(
                    r#"
                    INSERT INTO interest_rate_tier_audit
                    (id, name, minimum_balance, maximum_balance, interest_rate, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9)
                    "#,
                )
                .bind(item.id)
                .bind(item.name)
                .bind(item.minimum_balance)
                .bind(item.maximum_balance)
                .bind(item.interest_rate)
                .bind(item.antecedent_hash)
                .bind(item.antecedent_audit_log_id)
                .bind(item.hash)
                .bind(item.audit_log_id)
                .execute(&mut **transaction)
                .await?;

                // Build entity update query
                let rows_affected = sqlx::query(
                    r#"
                    UPDATE interest_rate_tier SET
                    name = $2,
                    minimum_balance = $3,
                    maximum_balance = $4,
                    interest_rate = $5,
                    antecedent_hash = $6,
                    antecedent_audit_log_id = $7,
                    hash = $8,
                    audit_log_id = $9
                    WHERE id = $1 AND hash = $10 AND audit_log_id = $11
                    "#,
                )
                .bind(item.id)
                .bind(item.name)
                .bind(item.minimum_balance)
                .bind(item.maximum_balance)
                .bind(item.interest_rate)
                .bind(item.antecedent_hash)
                .bind(item.antecedent_audit_log_id)
                .bind(item.hash)
                .bind(item.audit_log_id)
                .bind(previous_hash)
                .bind(previous_audit_log_id)
                .execute(&mut **transaction)
                .await?
                .rows_affected();

                if rows_affected == 0 {
                    return Err(format!(
                        "Concurrent update detected for InterestRateTier id: {}",
                        item.id
                    )
                    .into());
                }

                // Update index table
                let idx = item.to_index();
                sqlx::query(
                    r#"
                    UPDATE interest_rate_tier_idx SET
                    name = $2,
                    minimum_balance = $3,
                    maximum_balance = $4,
                    interest_rate = $5
                    WHERE id = $1
                    "#,
                )
                .bind(idx.id)
                .bind(idx.name)
                .bind(idx.minimum_balance)
                .bind(idx.maximum_balance)
                .bind(idx.interest_rate)
                .execute(&mut **transaction)
                .await?;

                // Create audit link
                let audit_link = AuditLinkModel {
                    audit_log_id,
                    entity_id: item.id,
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

                indices_to_remove.push(item.id);
                indices_to_add.push(idx);
                saved_items.push(item);
            }
        } // Transaction lock released here

        // Update cache after releasing transaction lock
        {
            let cache = repo.interest_rate_tier_idx_cache.write().await;
            for id in indices_to_remove {
                cache.remove(&id);
            }
            for idx in indices_to_add {
                cache.add(idx);
            }
        }

        Ok(saved_items)
    }
}

#[async_trait]
impl UpdateBatch<Postgres, InterestRateTierModel> for InterestRateTierRepositoryImpl {
    async fn update_batch(
        &self,
        items: Vec<InterestRateTierModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<InterestRateTierModel>, Box<dyn Error + Send + Sync>> {
        Self::update_batch_impl(self, items, audit_log_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::product::interest_rate_tier_repository::test_utils::create_test_interest_rate_tier;
    use crate::test_helper::setup_test_context;
    use business_core_db::{
        repository::{create_batch::CreateBatch, update_batch::UpdateBatch},
    };
    use crate::repository::person::test_utils::create_test_audit_log;
    use rust_decimal::Decimal;

    #[tokio::test]
    async fn test_update_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

        // Update entities
        let update_audit_log = create_test_audit_log();
        audit_log_repo.create(&update_audit_log).await?;
        let mut updated_entities = Vec::new();
        for mut interest_rate_tier in saved {
            interest_rate_tier.interest_rate = Decimal::new(5, 2); // 0.05
            updated_entities.push(interest_rate_tier);
        }

        let updated = interest_rate_tier_repo
            .update_batch(updated_entities, Some(update_audit_log.id))
            .await?;

        assert_eq!(updated.len(), 3);
        for interest_rate_tier in updated {
            assert_eq!(interest_rate_tier.interest_rate, Decimal::new(5, 2));
        }

        Ok(())
    }
}