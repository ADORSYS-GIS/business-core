use std::error::Error;

use business_core_db::{
    models::{
        audit::{audit_link::AuditLinkModel, entity_type::EntityType},
        product::account_gl_mapping::AccountGlMappingModel,
    },
    utils::hash_as_i64,
};
use uuid::Uuid;

use super::repo_impl::AccountGlMappingRepositoryImpl;

impl AccountGlMappingRepositoryImpl {
    pub async fn create_batch_internal(
        &self,
        items: Vec<AccountGlMappingModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<AccountGlMappingModel>, Box<dyn Error + Send + Sync>> {
        let mut tx = self.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        let audit_log_id = audit_log_id.unwrap_or_else(Uuid::new_v4);

        let mut res: Vec<AccountGlMappingModel> = Vec::with_capacity(items.len());

        for mut item in items {
            let mut entity_for_hashing = item.clone();
            entity_for_hashing.hash = 0;
            entity_for_hashing.audit_log_id = Some(audit_log_id);

            let computed_hash = hash_as_i64(&entity_for_hashing)?;
            item.hash = computed_hash;
            item.audit_log_id = Some(audit_log_id);

            let audit_insert_query = sqlx::query(
                r#"
                INSERT INTO account_gl_mapping_audit
                (id, customer_account_code, overdraft_code, hash, audit_log_id)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(item.id)
            .bind(item.customer_account_code.as_str())
            .bind(item.overdraft_code.as_deref())
            .bind(item.hash)
            .bind(item.audit_log_id);

            let entity_insert_query = sqlx::query(
                r#"
                INSERT INTO account_gl_mapping
                (id, customer_account_code, overdraft_code, hash, audit_log_id)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(item.id)
            .bind(item.customer_account_code.as_str())
            .bind(item.overdraft_code.as_deref())
            .bind(item.hash)
            .bind(item.audit_log_id);

            let idx_insert_query = sqlx::query(
                r#"
                INSERT INTO account_gl_mapping_idx
                (id, customer_account_code, overdraft_code)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(item.id)
            .bind(item.customer_account_code.as_str())
            .bind(item.overdraft_code.as_deref());

            let audit_link = AuditLinkModel {
                audit_log_id,
                entity_id: item.id,
                entity_type: EntityType::AccountGlMapping,
            };

            let audit_link_query = sqlx::query(
                r#"
                INSERT INTO audit_link (audit_log_id, entity_id, entity_type)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(audit_link.audit_log_id)
            .bind(audit_link.entity_id)
            .bind(audit_link.entity_type);

            audit_insert_query.execute(&mut **transaction).await?;
            entity_insert_query.execute(&mut **transaction).await?;
            idx_insert_query.execute(&mut **transaction).await?;
            audit_link_query.execute(&mut **transaction).await?;

            res.push(item);
        }
        let cache = self.account_gl_mapping_idx_cache.read().await;
        res.iter()
            .for_each(|item| cache.add(item.to_index()));
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{repository::db_init::DbInitializer, test_helper::TestHelper};

    use super::*;

    #[tokio::test]
    async fn test_create_batch() -> Result<(), Box<dyn Error + Send + Sync>> {
        let db = DbInitializer::init_test_db().await?;
        let helper = TestHelper::new(db.clone());
        let audit_log = helper.create_audit_log().await?;

        let mut items = vec![];
        for i in 0..5 {
            let item = super::super::test_utils::create_test_account_gl_mapping(
                &format!("CUST{}", i),
                Some(&format!("OD{}", i)),
            );
            items.push(item);
        }
        let repo = AccountGlMappingRepositoryImpl::new(db.clone());
        let saved_items = repo.create_batch(items, Some(audit_log.id)).await?;

        assert_eq!(saved_items.len(), 5);
        for item in saved_items {
            assert!(item.customer_account_code.starts_with("CUST"));
            assert!(item.overdraft_code.unwrap().starts_with("OD"));
            assert_ne!(item.hash, 0);
            assert_eq!(item.audit_log_id, Some(audit_log.id));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_account_gl_mapping_insert_triggers_cache_notification(
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = DbInitializer::init_test_db_and_listen().await?;
        let pool = ctx.pool();
        let repo = AccountGlMappingRepositoryImpl::new(pool.clone());

        let audit_log = TestHelper::new(pool.clone()).create_audit_log().await?;
        let item = super::super::test_utils::create_test_account_gl_mapping(
            "CUST1",
            Some("OD1"),
        );
        let item_idx = item.to_index();

        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

        let mut item_for_hashing = item.clone();
        item_for_hashing.hash = 0;
        item_for_hashing.audit_log_id = Some(audit_log.id);
        let computed_hash = hash_as_i64(&item_for_hashing).unwrap();
        let final_item = AccountGlMappingModel {
            hash: computed_hash,
            audit_log_id: Some(audit_log.id),
            ..item
        };

        sqlx::query(
            r#"
            INSERT INTO account_gl_mapping
            (id, customer_account_code, overdraft_code, hash, audit_log_id)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(final_item.id)
        .bind(final_item.customer_account_code.as_str())
        .bind(final_item.overdraft_code.as_deref())
        .bind(final_item.hash)
        .bind(final_item.audit_log_id)
        .execute(&**pool)
        .await
        .expect("Failed to insert account_gl_mapping");

        sqlx::query("INSERT INTO account_gl_mapping_idx (id, customer_account_code, overdraft_code) VALUES ($1, $2, $3)")
            .bind(item_idx.id)
            .bind(item_idx.customer_account_code.as_str())
            .bind(item_idx.overdraft_code.as_deref())
            .execute(&**pool)
            .await
            .expect("Failed to insert account_gl_mapping index");

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let cache = repo.account_gl_mapping_idx_cache.read().await;
        assert!(
            cache.contains_primary(&item_idx.id),
            "AccountGlMapping should be in cache after insert"
        );
        drop(cache);

        sqlx::query("DELETE FROM account_gl_mapping WHERE id = $1")
            .bind(item_idx.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete account_gl_mapping");

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let cache = repo.account_gl_mapping_idx_cache.read().await;
        assert!(
            !cache.contains_primary(&item_idx.id),
            "AccountGlMapping should be removed from cache after delete"
        );

        Ok(())
    }
}