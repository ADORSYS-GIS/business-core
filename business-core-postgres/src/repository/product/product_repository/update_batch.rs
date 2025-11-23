use async_trait::async_trait;
use business_core_db::models::{
    audit::{AuditLinkModel, AuditEntityType},
    product::product::ProductModel,
};
use business_core_db::models::index_aware::IndexAware;
use business_core_db::repository::update_batch::UpdateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::utils::hash_as_i64;

use super::repo_impl::ProductRepositoryImpl;

impl ProductRepositoryImpl {
    pub(super) async fn update_batch_impl(
        &self,
        items: Vec<ProductModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<ProductModel>, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for ProductModel")?;
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut updated_items = Vec::new();
        let mut indices_to_update = Vec::new();
        
        {
            let mut tx = self.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
            
            for mut item in items {
                let previous_hash = item.hash;
                let previous_audit_log_id = item.audit_log_id.ok_or("Entity must have audit_log_id for update")?;

                let mut entity_for_hashing = item.clone();
                entity_for_hashing.hash = 0;
                let computed_hash = hash_as_i64(&entity_for_hashing)?;

                if computed_hash == previous_hash {
                    updated_items.push(item);
                    continue;
                }

                item.antecedent_hash = previous_hash;
                item.antecedent_audit_log_id = previous_audit_log_id;
                item.audit_log_id = Some(audit_log_id);
                item.hash = 0;

                let new_computed_hash = hash_as_i64(&item)?;
                item.hash = new_computed_hash;

                sqlx::query(
                    r#"
                    INSERT INTO product_audit
                    (id, name, product_type, minimum_balance, maximum_balance, overdraft_allowed, overdraft_limit, interest_calculation_method, interest_posting_frequency, dormancy_threshold_days, minimum_opening_balance, closure_fee, maintenance_fee, maintenance_fee_frequency, default_dormancy_days, default_overdraft_limit, per_transaction_limit, daily_transaction_limit, weekly_transaction_limit, monthly_transaction_limit, overdraft_interest_rate, accrual_frequency, interest_rate_tier_1, interest_rate_tier_2, interest_rate_tier_3, interest_rate_tier_4, interest_rate_tier_5, account_gl_mapping, fee_type_gl_mapping, is_active, valid_from, valid_to, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34, $35, $36)
                    "#,
                )
                .bind(item.id)
                .bind(item.name)
                .bind(item.product_type)
                .bind(item.minimum_balance)
                .bind(item.maximum_balance)
                .bind(item.overdraft_allowed)
                .bind(item.overdraft_limit)
                .bind(item.interest_calculation_method)
                .bind(item.interest_posting_frequency)
                .bind(item.dormancy_threshold_days)
                .bind(item.minimum_opening_balance)
                .bind(item.closure_fee)
                .bind(item.maintenance_fee)
                .bind(item.maintenance_fee_frequency)
                .bind(item.default_dormancy_days)
                .bind(item.default_overdraft_limit)
                .bind(item.per_transaction_limit)
                .bind(item.daily_transaction_limit)
                .bind(item.weekly_transaction_limit)
                .bind(item.monthly_transaction_limit)
                .bind(item.overdraft_interest_rate)
                .bind(item.accrual_frequency)
                .bind(item.interest_rate_tier_1)
                .bind(item.interest_rate_tier_2)
                .bind(item.interest_rate_tier_3)
                .bind(item.interest_rate_tier_4)
                .bind(item.interest_rate_tier_5)
                .bind(item.account_gl_mapping)
                .bind(item.fee_type_gl_mapping)
                .bind(item.is_active)
                .bind(item.valid_from)
                .bind(item.valid_to)
                .bind(item.antecedent_hash)
                .bind(item.antecedent_audit_log_id)
                .bind(item.hash)
                .bind(item.audit_log_id)
                .execute(&mut **transaction)
                .await?;

                let rows_affected = sqlx::query(
                    r#"
                    UPDATE product SET
                    name = $2, product_type = $3, minimum_balance = $4, maximum_balance = $5,
                    overdraft_allowed = $6, overdraft_limit = $7, interest_calculation_method = $8,
                    interest_posting_frequency = $9, dormancy_threshold_days = $10, minimum_opening_balance = $11,
                    closure_fee = $12, maintenance_fee = $13, maintenance_fee_frequency = $14,
                    default_dormancy_days = $15, default_overdraft_limit = $16, per_transaction_limit = $17,
                    daily_transaction_limit = $18, weekly_transaction_limit = $19, monthly_transaction_limit = $20,
                    overdraft_interest_rate = $21, accrual_frequency = $22, interest_rate_tier_1 = $23,
                    interest_rate_tier_2 = $24, interest_rate_tier_3 = $25, interest_rate_tier_4 = $26,
                    interest_rate_tier_5 = $27, account_gl_mapping = $28, fee_type_gl_mapping = $29,
                    is_active = $30, valid_from = $31, valid_to = $32, antecedent_hash = $33,
                    antecedent_audit_log_id = $34, hash = $35, audit_log_id = $36
                    WHERE id = $1 AND hash = $37 AND audit_log_id = $38
                    "#,
                )
                .bind(item.id)
                .bind(item.name)
                .bind(item.product_type)
                .bind(item.minimum_balance)
                .bind(item.maximum_balance)
                .bind(item.overdraft_allowed)
                .bind(item.overdraft_limit)
                .bind(item.interest_calculation_method)
                .bind(item.interest_posting_frequency)
                .bind(item.dormancy_threshold_days)
                .bind(item.minimum_opening_balance)
                .bind(item.closure_fee)
                .bind(item.maintenance_fee)
                .bind(item.maintenance_fee_frequency)
                .bind(item.default_dormancy_days)
                .bind(item.default_overdraft_limit)
                .bind(item.per_transaction_limit)
                .bind(item.daily_transaction_limit)
                .bind(item.weekly_transaction_limit)
                .bind(item.monthly_transaction_limit)
                .bind(item.overdraft_interest_rate)
                .bind(item.accrual_frequency)
                .bind(item.interest_rate_tier_1)
                .bind(item.interest_rate_tier_2)
                .bind(item.interest_rate_tier_3)
                .bind(item.interest_rate_tier_4)
                .bind(item.interest_rate_tier_5)
                .bind(item.account_gl_mapping)
                .bind(item.fee_type_gl_mapping)
                .bind(item.is_active)
                .bind(item.valid_from)
                .bind(item.valid_to)
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
                    return Err("Concurrent update detected".into());
                }

                let idx = item.to_index();
                sqlx::query(
                    r#"
                    UPDATE product_idx SET
                    name = $2, product_type = $3, minimum_balance = $4, maximum_balance = $5,
                    overdraft_allowed = $6, overdraft_limit = $7, interest_calculation_method = $8,
                    interest_posting_frequency = $9, dormancy_threshold_days = $10, minimum_opening_balance = $11,
                    closure_fee = $12, maintenance_fee = $13, maintenance_fee_frequency = $14,
                    default_dormancy_days = $15, default_overdraft_limit = $16, per_transaction_limit = $17,
                    daily_transaction_limit = $18, weekly_transaction_limit = $19, monthly_transaction_limit = $20,
                    overdraft_interest_rate = $21, accrual_frequency = $22, interest_rate_tier_1 = $23,
                    interest_rate_tier_2 = $24, interest_rate_tier_3 = $25, interest_rate_tier_4 = $26,
                    interest_rate_tier_5 = $27, account_gl_mapping = $28, fee_type_gl_mapping = $29,
                    is_active = $30, valid_from = $31, valid_to = $32
                    WHERE id = $1
                    "#,
                )
                .bind(idx.id)
                .bind(idx.name)
                .bind(idx.product_type)
                .bind(idx.minimum_balance)
                .bind(idx.maximum_balance)
                .bind(idx.overdraft_allowed)
                .bind(idx.overdraft_limit)
                .bind(idx.interest_calculation_method)
                .bind(idx.interest_posting_frequency)
                .bind(idx.dormancy_threshold_days)
                .bind(idx.minimum_opening_balance)
                .bind(idx.closure_fee)
                .bind(idx.maintenance_fee)
                .bind(idx.maintenance_fee_frequency)
                .bind(idx.default_dormancy_days)
                .bind(idx.default_overdraft_limit)
                .bind(idx.per_transaction_limit)
                .bind(idx.daily_transaction_limit)
                .bind(idx.weekly_transaction_limit)
                .bind(idx.monthly_transaction_limit)
                .bind(idx.overdraft_interest_rate)
                .bind(idx.accrual_frequency)
                .bind(idx.interest_rate_tier_1)
                .bind(idx.interest_rate_tier_2)
                .bind(idx.interest_rate_tier_3)
                .bind(idx.interest_rate_tier_4)
                .bind(idx.interest_rate_tier_5)
                .bind(idx.account_gl_mapping)
                .bind(idx.fee_type_gl_mapping)
                .bind(idx.is_active)
                .bind(idx.valid_from)
                .bind(idx.valid_to)
                .execute(&mut **transaction)
                .await?;

                let audit_link = AuditLinkModel {
                    audit_log_id,
                    entity_id: item.id,
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

                indices_to_update.push((item.id, idx));
                updated_items.push(item);
            }
        }
        
        {
            let cache = self.product_idx_cache.read().await;
            for (id, idx) in indices_to_update {
                cache.remove(&id);
                cache.add(idx);
            }
        }

        Ok(updated_items)
    }
}

#[async_trait]
impl UpdateBatch<Postgres, ProductModel> for ProductRepositoryImpl {
    async fn update_batch(
        &self,
        items: Vec<ProductModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<ProductModel>, Box<dyn Error + Send + Sync>> {
        Self::update_batch_impl(self, items, audit_log_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::person::test_utils::create_test_audit_log;
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::update_batch::UpdateBatch;
    use rust_decimal::Decimal;
    use crate::repository::product::product_repository::test_utils::create_test_product;
    use crate::repository::product::account_gl_mapping_repository::test_utils::create_test_account_gl_mapping;
    use crate::repository::product::fee_type_gl_mapping_repository::test_utils::create_test_fee_type_gl_mapping;
    use business_core_db::models::description::named::NamedModel;
    use business_core_db::models::description::named_entity_type::NamedEntityType;
    use heapless::String as HeaplessString;
    use uuid::Uuid;

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
    async fn test_update_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let product_repo = &ctx.product_repos().product_repository;
        let account_gl_repo = &ctx.product_repos().account_gl_mapping_repository;
        let fee_type_gl_repo = &ctx.product_repos().fee_type_gl_mapping_repository;
        let named_repo = &ctx.description_repos().named_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create dependencies
        let named = create_test_named("Original Product");
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

        // Update product
        let update_audit_log = create_test_audit_log();
        audit_log_repo.create(&update_audit_log).await?;
        let mut updated_product = saved[0].clone();
        updated_product.minimum_balance = Decimal::from(500);

        let updated = product_repo.update_batch(vec![updated_product], Some(update_audit_log.id)).await?;

        assert_eq!(updated.len(), 1);
        assert_eq!(updated[0].minimum_balance, Decimal::from(500));

        Ok(())
    }

    #[tokio::test]
    async fn test_update_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let product_repo = &ctx.product_repos().product_repository;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        let updated = product_repo.update_batch(Vec::new(), Some(audit_log.id)).await?;

        assert_eq!(updated.len(), 0);

        Ok(())
    }
}