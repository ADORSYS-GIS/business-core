use business_core_db::models::audit::{AuditLinkModel, AuditEntityType};
use async_trait::async_trait;
use business_core_db::repository::load_batch::LoadBatch;
use business_core_db::repository::delete_batch::DeleteBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::utils::hash_as_i64;

use super::repo_impl::ProductRepositoryImpl;

impl ProductRepositoryImpl {
    pub(super) async fn delete_batch_impl(
        repo: &ProductRepositoryImpl,
        ids: &[Uuid],
        audit_log_id: Option<Uuid>,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for ProductModel")?;
        if ids.is_empty() {
            return Ok(0);
        }

        let entities_to_delete = repo.load_batch(ids).await?;
        let mut deleted_count = 0;

        {
            let mut tx = repo.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;

            for entity in entities_to_delete.into_iter().flatten() {
                let mut final_audit_entity = entity.clone();
                final_audit_entity.antecedent_hash = entity.hash;
                final_audit_entity.antecedent_audit_log_id = entity.audit_log_id.ok_or("Entity must have audit_log_id for deletion")?;
                final_audit_entity.audit_log_id = Some(audit_log_id);
                final_audit_entity.hash = 0;

                let final_hash = hash_as_i64(&final_audit_entity)?;
                final_audit_entity.hash = final_hash;

                sqlx::query(
                    r#"
                    INSERT INTO product_audit
                    (id, name, product_type, minimum_balance, maximum_balance, overdraft_allowed, overdraft_limit, interest_calculation_method, interest_posting_frequency, dormancy_threshold_days, minimum_opening_balance, closure_fee, maintenance_fee, maintenance_fee_frequency, default_dormancy_days, default_overdraft_limit, per_transaction_limit, daily_transaction_limit, weekly_transaction_limit, monthly_transaction_limit, overdraft_interest_rate, accrual_frequency, interest_rate_tier_1, interest_rate_tier_2, interest_rate_tier_3, interest_rate_tier_4, interest_rate_tier_5, account_gl_mapping, fee_type_gl_mapping, is_active, valid_from, valid_to, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34, $35, $36)
                    "#,
                )
                .bind(final_audit_entity.id)
                .bind(final_audit_entity.name)
                .bind(final_audit_entity.product_type)
                .bind(final_audit_entity.minimum_balance)
                .bind(final_audit_entity.maximum_balance)
                .bind(final_audit_entity.overdraft_allowed)
                .bind(final_audit_entity.overdraft_limit)
                .bind(final_audit_entity.interest_calculation_method)
                .bind(final_audit_entity.interest_posting_frequency)
                .bind(final_audit_entity.dormancy_threshold_days)
                .bind(final_audit_entity.minimum_opening_balance)
                .bind(final_audit_entity.closure_fee)
                .bind(final_audit_entity.maintenance_fee)
                .bind(final_audit_entity.maintenance_fee_frequency)
                .bind(final_audit_entity.default_dormancy_days)
                .bind(final_audit_entity.default_overdraft_limit)
                .bind(final_audit_entity.per_transaction_limit)
                .bind(final_audit_entity.daily_transaction_limit)
                .bind(final_audit_entity.weekly_transaction_limit)
                .bind(final_audit_entity.monthly_transaction_limit)
                .bind(final_audit_entity.overdraft_interest_rate)
                .bind(final_audit_entity.accrual_frequency)
                .bind(final_audit_entity.interest_rate_tier_1)
                .bind(final_audit_entity.interest_rate_tier_2)
                .bind(final_audit_entity.interest_rate_tier_3)
                .bind(final_audit_entity.interest_rate_tier_4)
                .bind(final_audit_entity.interest_rate_tier_5)
                .bind(final_audit_entity.account_gl_mapping)
                .bind(final_audit_entity.fee_type_gl_mapping)
                .bind(final_audit_entity.is_active)
                .bind(final_audit_entity.valid_from)
                .bind(final_audit_entity.valid_to)
                .bind(final_audit_entity.antecedent_hash)
                .bind(final_audit_entity.antecedent_audit_log_id)
                .bind(final_audit_entity.hash)
                .bind(final_audit_entity.audit_log_id)
                .execute(&mut **transaction)
                .await?;

                let result = sqlx::query(r#"DELETE FROM product WHERE id = $1"#)
                    .bind(entity.id)
                    .execute(&mut **transaction)
                    .await?;

                let audit_link = AuditLinkModel {
                    audit_log_id,
                    entity_id: entity.id,
                    entity_type: AuditEntityType::Product,
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
                
                deleted_count += result.rows_affected() as usize;
            }
        }
        
        {
            let cache = repo.product_idx_cache.read().await;
            for id in ids {
                cache.remove(id);
            }
        }
        
        Ok(deleted_count)
    }
}

#[async_trait]
impl DeleteBatch<Postgres> for ProductRepositoryImpl {
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
    use crate::repository::person::test_utils::create_test_audit_log;
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::delete_batch::DeleteBatch;
    use uuid::Uuid;
    use crate::repository::product::product_repository::test_utils::create_test_product;
    use crate::repository::product::account_gl_mapping_repository::test_utils::create_test_account_gl_mapping;
    use crate::repository::product::fee_type_gl_mapping_repository::test_utils::create_test_fee_type_gl_mapping;
    use business_core_db::models::description::named::NamedModel;
    use business_core_db::models::description::named_entity_type::NamedEntityType;
    use heapless::String as HeaplessString;

    fn create_test_named(name: &str) -> NamedModel {
        NamedModel {
            id: Uuid::new_v4(),
            entity_type: NamedEntityType::Country,
            name_l1: HeaplessString::try_from(name).unwrap(),
            name_l2: None,
            name_l3: None,
            name_l4: None,
            description_l1: None,
            description_l2: None,
            description_l3: None,
            description_l4: None,
            antecedent_hash: 0,
            antecedent_audit_log_id: Uuid::nil(),
            hash: 0,
            audit_log_id: None,
        }
    }

    #[tokio::test]
    async fn test_delete_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let product_repo = &ctx.product_repos().product_repository;
        let account_gl_repo = &ctx.product_repos().account_gl_mapping_repository;
        let fee_type_gl_repo = &ctx.product_repos().fee_type_gl_mapping_repository;
        let named_repo = &ctx.description_repos().named_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create dependencies
        let named = create_test_named("Product to Delete");
        let saved_named = named_repo.create_batch(vec![named], Some(audit_log.id)).await?;

        let account_gl = create_test_account_gl_mapping("ACC001");
        let fee_type_gl = create_test_fee_type_gl_mapping("FEE001");
        let saved_account_gl = account_gl_repo.create_batch(vec![account_gl], Some(audit_log.id)).await?;
        let saved_fee_type_gl = fee_type_gl_repo.create_batch(vec![fee_type_gl], Some(audit_log.id)).await?;

        let product = create_test_product(
            saved_named[0].id,
            saved_account_gl[0].id,
            saved_fee_type_gl[0].id,
        );

        let saved = product_repo.create_batch(vec![product], Some(audit_log.id)).await?;

        let ids: Vec<Uuid> = saved.iter().map(|s| s.id).collect();
        let delete_audit_log = create_test_audit_log();
        audit_log_repo.create(&delete_audit_log).await?;
        let deleted_count = product_repo.delete_batch(&ids, Some(delete_audit_log.id)).await?;

        assert_eq!(deleted_count, 1);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_batch_with_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let product_repo = &ctx.product_repos().product_repository;
        let account_gl_repo = &ctx.product_repos().account_gl_mapping_repository;
        let fee_type_gl_repo = &ctx.product_repos().fee_type_gl_mapping_repository;
        let named_repo = &ctx.description_repos().named_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create dependencies
        let named = create_test_named("Product to Delete");
        let saved_named = named_repo.create_batch(vec![named], Some(audit_log.id)).await?;

        let account_gl = create_test_account_gl_mapping("ACC001");
        let fee_type_gl = create_test_fee_type_gl_mapping("FEE001");
        let saved_account_gl = account_gl_repo.create_batch(vec![account_gl], Some(audit_log.id)).await?;
        let saved_fee_type_gl = fee_type_gl_repo.create_batch(vec![fee_type_gl], Some(audit_log.id)).await?;

        let product = create_test_product(
            saved_named[0].id,
            saved_account_gl[0].id,
            saved_fee_type_gl[0].id,
        );

        let saved = product_repo.create_batch(vec![product], Some(audit_log.id)).await?;

        let mut ids = vec![saved[0].id];
        ids.push(Uuid::new_v4()); // Add non-existing ID

        let delete_audit_log = create_test_audit_log();
        audit_log_repo.create(&delete_audit_log).await?;
        let deleted_count = product_repo.delete_batch(&ids, Some(delete_audit_log.id)).await?;

        assert_eq!(deleted_count, 1); // Only one actually deleted

        Ok(())
    }
}