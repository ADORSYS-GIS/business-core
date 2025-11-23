use async_trait::async_trait;
use business_core_db::models::{
    audit::{AuditLinkModel, AuditEntityType},
    product::product::ProductModel,
};
use business_core_db::repository::create_batch::CreateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::models::index_aware::IndexAware;
use business_core_db::utils::hash_as_i64;

use super::repo_impl::ProductRepositoryImpl;

impl ProductRepositoryImpl {
    pub(super) async fn create_batch_impl(
        repo: &ProductRepositoryImpl,
        items: Vec<ProductModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<ProductModel>, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for ProductModel")?;
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut saved_items = Vec::new();
        let mut indices = Vec::new();
        
        {
            let mut tx = repo.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
            
            for mut item in items {
                // 1. Create a copy of entity for hashing
                let mut entity_for_hashing = item.clone();
                entity_for_hashing.hash = 0;
                entity_for_hashing.audit_log_id = Some(audit_log_id);

                // 2. Compute hash
                let computed_hash = hash_as_i64(&entity_for_hashing)?;

                // 3. Update original entity with computed hash and new audit_log_id
                item.hash = computed_hash;
                item.audit_log_id = Some(audit_log_id);

                // Execute audit insert
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

                // Execute main insert
                sqlx::query(
                    r#"
                    INSERT INTO product
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

                // Insert into index table
                let idx = item.to_index();
                sqlx::query(
                    r#"
                    INSERT INTO product_idx (id, name, product_type, minimum_balance, maximum_balance, overdraft_allowed, overdraft_limit, interest_calculation_method, interest_posting_frequency, dormancy_threshold_days, minimum_opening_balance, closure_fee, maintenance_fee, maintenance_fee_frequency, default_dormancy_days, default_overdraft_limit, per_transaction_limit, daily_transaction_limit, weekly_transaction_limit, monthly_transaction_limit, overdraft_interest_rate, accrual_frequency, interest_rate_tier_1, interest_rate_tier_2, interest_rate_tier_3, interest_rate_tier_4, interest_rate_tier_5, account_gl_mapping, fee_type_gl_mapping, is_active, valid_from, valid_to)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32)
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

                // Create audit link
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

                indices.push(idx);
                saved_items.push(item);
            }
        }
        
        {
            let cache = repo.product_idx_cache.read().await;
            for idx in indices {
                cache.add(idx);
            }
        }

        Ok(saved_items)
    }
}

#[async_trait]
impl CreateBatch<Postgres, ProductModel> for ProductRepositoryImpl {
    async fn create_batch(
        &self,
        items: Vec<ProductModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<ProductModel>, Box<dyn Error + Send + Sync>> {
        Self::create_batch_impl(self, items, audit_log_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::person::test_utils::create_test_audit_log;
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
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
    async fn test_create_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let product_repo = &ctx.product_repos().product_repository;
        let account_gl_repo = &ctx.product_repos().account_gl_mapping_repository;
        let fee_type_gl_repo = &ctx.product_repos().fee_type_gl_mapping_repository;
        let named_repo = &ctx.description_repos().named_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create dependencies
        let named1 = create_test_named("Product 1");
        let named2 = create_test_named("Product 2");
        let saved_named = named_repo.create_batch(vec![named1.clone(), named2.clone()], Some(audit_log.id)).await?;

        let account_gl = create_test_account_gl_mapping("ACC001");
        let fee_type_gl = create_test_fee_type_gl_mapping("FEE001");
        let saved_account_gl = account_gl_repo.create_batch(vec![account_gl], Some(audit_log.id)).await?;
        let saved_fee_type_gl = fee_type_gl_repo.create_batch(vec![fee_type_gl], Some(audit_log.id)).await?;

        let mut products = Vec::new();
        for i in 0..2 {
            let product = create_test_product(
                saved_named[i].id,
                saved_account_gl[0].id,
                saved_fee_type_gl[0].id,
            );
            products.push(product);
        }

        let saved_products = product_repo
            .create_batch(products.clone(), Some(audit_log.id))
            .await?;

        assert_eq!(saved_products.len(), 2);

        for saved_product in saved_products.iter() {
            assert!(saved_product.audit_log_id.is_some());
            assert_eq!(saved_product.audit_log_id.unwrap(), audit_log.id);
            assert_ne!(saved_product.hash, 0);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_create_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let product_repo = &ctx.product_repos().product_repository;

        let audit_log = create_test_audit_log();
        let saved_products = product_repo
            .create_batch(Vec::new(), Some(audit_log.id))
            .await?;

        assert_eq!(saved_products.len(), 0);

        Ok(())
    }

    #[tokio::test]
    async fn test_product_insert_triggers_cache_notification(
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::test_helper::setup_test_context_and_listen;
        use business_core_db::models::product::account_gl_mapping::AccountGlMappingModel;
        use business_core_db::models::product::fee_type_gl_mapping::FeeTypeGlMappingModel;
        use tokio::time::{sleep, Duration};
        use business_core_db::models::index_aware::IndexAware;

        // Setup test context with the handler
        let ctx = setup_test_context_and_listen().await?;
        let pool = ctx.pool();

        // Create a test product entity
        let test_named = create_test_named("Test Product");
        let test_account_gl = create_test_account_gl_mapping("ACC001");
        let test_fee_type_gl = create_test_fee_type_gl_mapping("FEE001");
        
        // Give listener more time to start and establish connection
        sleep(Duration::from_millis(2000)).await;

        // Insert the audit log
        let audit_log = create_test_audit_log();
        sqlx::query(
            r#"
            INSERT INTO audit_log (id, updated_at, updated_by_person_id)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(audit_log.id)
        .bind(audit_log.updated_at)
        .bind(audit_log.updated_by_person_id)
        .execute(&**pool)
        .await
        .expect("Failed to insert audit log");

        // Insert named dependency
        let mut test_named_for_hashing = test_named.clone();
        test_named_for_hashing.hash = 0;
        test_named_for_hashing.audit_log_id = Some(audit_log.id);
        let named_hash = business_core_db::utils::hash_as_i64(&test_named_for_hashing).unwrap();
        let final_named = NamedModel {
            hash: named_hash,
            audit_log_id: Some(audit_log.id),
            ..test_named
        };

        sqlx::query(
            r#"
            INSERT INTO named
            (id, entity_type, name_l1, name_l2, name_l3, name_l4, description_l1, description_l2, description_l3, description_l4, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
            "#,
        )
        .bind(final_named.id)
        .bind(final_named.entity_type)
        .bind(final_named.name_l1.as_str())
        .bind(final_named.name_l2.as_deref())
        .bind(final_named.name_l3.as_deref())
        .bind(final_named.name_l4.as_deref())
        .bind(final_named.description_l1.as_deref())
        .bind(final_named.description_l2.as_deref())
        .bind(final_named.description_l3.as_deref())
        .bind(final_named.description_l4.as_deref())
        .bind(final_named.antecedent_hash)
        .bind(final_named.antecedent_audit_log_id)
        .bind(final_named.hash)
        .bind(final_named.audit_log_id)
        .execute(&**pool)
        .await
        .expect("Failed to insert named");

        // Insert account_gl_mapping and fee_type_gl_mapping dependencies
        let mut account_gl_for_hashing = test_account_gl.clone();
        account_gl_for_hashing.hash = 0;
        account_gl_for_hashing.audit_log_id = Some(audit_log.id);
        let account_gl_hash =
            business_core_db::utils::hash_as_i64(&account_gl_for_hashing).unwrap();
        let final_account_gl = AccountGlMappingModel {
            hash: account_gl_hash,
            audit_log_id: Some(audit_log.id),
            ..test_account_gl
        };
        sqlx::query(
            r#"
            INSERT INTO account_gl_mapping
            (id, customer_account_code, overdraft_code, hash, audit_log_id, antecedent_hash, antecedent_audit_log_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(final_account_gl.id)
        .bind(final_account_gl.customer_account_code.as_str())
        .bind(final_account_gl.overdraft_code.as_deref())
        .bind(final_account_gl.hash)
        .bind(final_account_gl.audit_log_id)
        .bind(final_account_gl.antecedent_hash)
        .bind(final_account_gl.antecedent_audit_log_id)
        .execute(&**pool)
        .await
        .expect("Failed to insert account_gl_mapping");

        let mut fee_type_gl_for_hashing = test_fee_type_gl.clone();
        fee_type_gl_for_hashing.hash = 0;
        fee_type_gl_for_hashing.audit_log_id = Some(audit_log.id);
        let fee_type_gl_hash =
            business_core_db::utils::hash_as_i64(&fee_type_gl_for_hashing).unwrap();
        let final_fee_type_gl = FeeTypeGlMappingModel {
            hash: fee_type_gl_hash,
            audit_log_id: Some(audit_log.id),
            ..test_fee_type_gl
        };
        sqlx::query(
            r#"
            INSERT INTO fee_type_gl_mapping
            (id, fee_type, gl_code, hash, audit_log_id, antecedent_hash, antecedent_audit_log_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7)
            "#,
        )
        .bind(final_fee_type_gl.id)
        .bind(final_fee_type_gl.fee_type)
        .bind(final_fee_type_gl.gl_code.as_str())
        .bind(final_fee_type_gl.hash)
        .bind(final_fee_type_gl.audit_log_id)
        .bind(final_fee_type_gl.antecedent_hash)
        .bind(final_fee_type_gl.antecedent_audit_log_id)
        .execute(&**pool)
        .await
        .expect("Failed to insert fee_type_gl_mapping");

        // Create test product
        let test_product = create_test_product(final_named.id, test_account_gl.id, test_fee_type_gl.id);
        let product_idx = test_product.to_index();
        
        let mut test_product_for_hashing = test_product.clone();
        test_product_for_hashing.hash = 0;
        test_product_for_hashing.audit_log_id = Some(audit_log.id);
        let computed_hash = business_core_db::utils::hash_as_i64(&test_product_for_hashing).unwrap();
        let final_product = business_core_db::models::product::product::ProductModel {
            hash: computed_hash,
            audit_log_id: Some(audit_log.id),
            ..test_product
        };

        // Insert product directly
        sqlx::query(
            r#"
            INSERT INTO product
            (id, name, product_type, minimum_balance, maximum_balance, overdraft_allowed, overdraft_limit, interest_calculation_method, interest_posting_frequency, dormancy_threshold_days, minimum_opening_balance, closure_fee, maintenance_fee, maintenance_fee_frequency, default_dormancy_days, default_overdraft_limit, per_transaction_limit, daily_transaction_limit, weekly_transaction_limit, monthly_transaction_limit, overdraft_interest_rate, accrual_frequency, interest_rate_tier_1, interest_rate_tier_2, interest_rate_tier_3, interest_rate_tier_4, interest_rate_tier_5, account_gl_mapping, fee_type_gl_mapping, is_active, valid_from, valid_to, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32, $33, $34, $35, $36)
            "#,
        )
        .bind(final_product.id)
        .bind(final_product.name)
        .bind(final_product.product_type)
        .bind(final_product.minimum_balance)
        .bind(final_product.maximum_balance)
        .bind(final_product.overdraft_allowed)
        .bind(final_product.overdraft_limit)
        .bind(final_product.interest_calculation_method)
        .bind(final_product.interest_posting_frequency)
        .bind(final_product.dormancy_threshold_days)
        .bind(final_product.minimum_opening_balance)
        .bind(final_product.closure_fee)
        .bind(final_product.maintenance_fee)
        .bind(final_product.maintenance_fee_frequency)
        .bind(final_product.default_dormancy_days)
        .bind(final_product.default_overdraft_limit)
        .bind(final_product.per_transaction_limit)
        .bind(final_product.daily_transaction_limit)
        .bind(final_product.weekly_transaction_limit)
        .bind(final_product.monthly_transaction_limit)
        .bind(final_product.overdraft_interest_rate)
        .bind(final_product.accrual_frequency)
        .bind(final_product.interest_rate_tier_1)
        .bind(final_product.interest_rate_tier_2)
        .bind(final_product.interest_rate_tier_3)
        .bind(final_product.interest_rate_tier_4)
        .bind(final_product.interest_rate_tier_5)
        .bind(final_product.account_gl_mapping)
        .bind(final_product.fee_type_gl_mapping)
        .bind(final_product.is_active)
        .bind(final_product.valid_from)
        .bind(final_product.valid_to)
        .bind(final_product.antecedent_hash)
        .bind(final_product.antecedent_audit_log_id)
        .bind(final_product.hash)
        .bind(final_product.audit_log_id)
        .execute(&**pool)
        .await
        .expect("Failed to insert product");

        // Insert product index directly - this should trigger the cache notification
        sqlx::query(
            r#"
            INSERT INTO product_idx (id, name, product_type, minimum_balance, maximum_balance, overdraft_allowed, overdraft_limit, interest_calculation_method, interest_posting_frequency, dormancy_threshold_days, minimum_opening_balance, closure_fee, maintenance_fee, maintenance_fee_frequency, default_dormancy_days, default_overdraft_limit, per_transaction_limit, daily_transaction_limit, weekly_transaction_limit, monthly_transaction_limit, overdraft_interest_rate, accrual_frequency, interest_rate_tier_1, interest_rate_tier_2, interest_rate_tier_3, interest_rate_tier_4, interest_rate_tier_5, account_gl_mapping, fee_type_gl_mapping, is_active, valid_from, valid_to)
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26, $27, $28, $29, $30, $31, $32)
            "#,
        )
        .bind(product_idx.id)
        .bind(product_idx.name)
        .bind(product_idx.product_type)
        .bind(product_idx.minimum_balance)
        .bind(product_idx.maximum_balance)
        .bind(product_idx.overdraft_allowed)
        .bind(product_idx.overdraft_limit)
        .bind(product_idx.interest_calculation_method)
        .bind(product_idx.interest_posting_frequency)
        .bind(product_idx.dormancy_threshold_days)
        .bind(product_idx.minimum_opening_balance)
        .bind(product_idx.closure_fee)
        .bind(product_idx.maintenance_fee)
        .bind(product_idx.maintenance_fee_frequency)
        .bind(product_idx.default_dormancy_days)
        .bind(product_idx.default_overdraft_limit)
        .bind(product_idx.per_transaction_limit)
        .bind(product_idx.daily_transaction_limit)
        .bind(product_idx.weekly_transaction_limit)
        .bind(product_idx.monthly_transaction_limit)
        .bind(product_idx.overdraft_interest_rate)
        .bind(product_idx.accrual_frequency)
        .bind(product_idx.interest_rate_tier_1)
        .bind(product_idx.interest_rate_tier_2)
        .bind(product_idx.interest_rate_tier_3)
        .bind(product_idx.interest_rate_tier_4)
        .bind(product_idx.interest_rate_tier_5)
        .bind(product_idx.account_gl_mapping)
        .bind(product_idx.fee_type_gl_mapping)
        .bind(product_idx.is_active)
        .bind(product_idx.valid_from)
        .bind(product_idx.valid_to)
        .execute(&**pool)
        .await
        .expect("Failed to insert product index");

        // Give more time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        let product_repo = &ctx.product_repos().product_repository;

        // Verify the cache was updated via the trigger
        let cache = product_repo.product_idx_cache.read().await;
        assert!(
            cache.contains_primary(&product_idx.id),
            "Product should be in cache after insert"
        );

        let cached_product = cache.get_by_primary(&product_idx.id);
        assert!(
            cached_product.is_some(),
            "Product should be retrievable from cache"
        );

        // Verify the cached data matches
        let cached_product = cached_product.unwrap();
        assert_eq!(cached_product.id, product_idx.id);

        // Drop the read lock before proceeding
        drop(cache);

        // Delete the records from the database
        sqlx::query("DELETE FROM product WHERE id = $1")
            .bind(product_idx.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete product");

        sqlx::query("DELETE FROM fee_type_gl_mapping WHERE id = $1")
            .bind(final_fee_type_gl.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete fee_type_gl_mapping");

        sqlx::query("DELETE FROM account_gl_mapping WHERE id = $1")
            .bind(final_account_gl.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete account_gl_mapping");

        sqlx::query("DELETE FROM named WHERE id = $1")
            .bind(final_named.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete named");

        sqlx::query("DELETE FROM audit_log WHERE id = $1")
            .bind(audit_log.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete audit log");

        // Give more time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        // Verify the cache entry was removed
        let cache = product_repo.product_idx_cache.read().await;
        assert!(
            !cache.contains_primary(&product_idx.id),
            "Product should be removed from cache after delete"
        );

        Ok(())
    }
}