use std::error::Error;

use business_core_db::{
    models::{
        audit::{audit_link::AuditLinkModel, audit_log::AuditLogModel, entity_type::EntityType},
        product::fee_type_gl_mapping::FeeTypeGlMappingModel,
    },
    utils::hash_as_i64,
};
use uuid::Uuid;

use super::repo_impl::FeeTypeGlMappingRepositoryImpl;

impl FeeTypeGlMappingRepositoryImpl {
    pub async fn create_batch_internal(
        &self,
        items: Vec<FeeTypeGlMappingModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<FeeTypeGlMappingModel>, Box<dyn Error + Send + Sync>> {
        let mut tx = self.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        let audit_log_id = audit_log_id.unwrap_or_else(Uuid::new_v4);

        let mut res: Vec<FeeTypeGlMappingModel> = Vec::with_capacity(items.len());

        for mut item in items {
            let mut entity_for_hashing = item.clone();
            entity_for_hashing.hash = 0;
            entity_for_hashing.audit_log_id = Some(audit_log_id);

            let computed_hash = hash_as_i64(&entity_for_hashing)?;
            item.hash = computed_hash;
            item.audit_log_id = Some(audit_log_id);

            let audit_insert_query = sqlx::query(
                r#"
                INSERT INTO fee_type_gl_mapping_audit
                (id, fee_type, gl_code, hash, audit_log_id)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(item.id)
            .bind(item.fee_type.clone())
            .bind(item.gl_code.as_str())
            .bind(item.hash)
            .bind(item.audit_log_id);

            let entity_insert_query = sqlx::query(
                r#"
                INSERT INTO fee_type_gl_mapping
                (id, fee_type, gl_code, hash, audit_log_id)
                VALUES ($1, $2, $3, $4, $5)
                "#,
            )
            .bind(item.id)
            .bind(item.fee_type.clone())
            .bind(item.gl_code.as_str())
            .bind(item.hash)
            .bind(item.audit_log_id);

            let idx_insert_query = sqlx::query(
                r#"
                INSERT INTO fee_type_gl_mapping_idx
                (id, fee_type, gl_code)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(item.id)
            .bind(item.fee_type.clone())
            .bind(item.gl_code.as_str());

            let audit_link = AuditLinkModel {
                audit_log_id,
                entity_id: item.id,
                entity_type: AuditEntityType::FeeTypeGlMapping,
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
        let cache = self.fee_type_gl_mapping_idx_cache.read().await;
        res.iter()
            .for_each(|item| cache.add(item.to_index()));
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{repository::db_init::DbInitializer, test_helper::TestHelper};
    use business_core_db::models::product::fee_type_gl_mapping::FeeType;

    use super::*;

    #[tokio::test]
    async fn test_create_batch() -> Result<(), Box<dyn Error + Send + Sync>> {
        let db = DbInitializer::init_test_db().await?;
        let helper = TestHelper::new(db.clone());
        let audit_log = helper.create_audit_log().await?;

        let mut items = vec![];
        for i in 0..5 {
            let item = FeeTypeGlMappingModel {
                id: Uuid::new_v4(),
                fee_type: FeeType::InterestExpense,
                gl_code: format!("GLCODE{}", i).try_into().unwrap(),
                antecedent_hash: 0,
                antecedent_audit_log_id: Uuid::nil(),
                hash: 0,
                audit_log_id: None,
            };
            items.push(item);
        }
        let repo = FeeTypeGlMappingRepositoryImpl::new(db.clone());
        let saved_items = repo.create_batch(items, Some(audit_log.id)).await?;

        assert_eq!(saved_items.len(), 5);
        for item in saved_items {
            assert!(item.gl_code.starts_with("GLCODE"));
            assert_ne!(item.hash, 0);
            assert_eq!(item.audit_log_id, Some(audit_log.id));
        }

        Ok(())
    }
}
    #[tokio::test]
    async fn test_fee_type_gl_mapping_insert_triggers_cache_notification(
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = DbInitializer::init_test_db_and_listen().await?;
        let pool = ctx.pool();
        let repo = FeeTypeGlMappingRepositoryImpl::new(pool.clone());

        let audit_log = TestHelper::new(pool.clone()).create_audit_log().await?;
        let item = super::super::test_utils::create_test_fee_type_gl_mapping(
            FeeType::InterestExpense,
            "GLCODE1",
        );
        let item_idx = item.to_index();

        tokio::time::sleep(std::time::Duration::from_millis(2000)).await;

        let mut item_for_hashing = item.clone();
        item_for_hashing.hash = 0;
        item_for_hashing.audit_log_id = Some(audit_log.id);
        let computed_hash = hash_as_i64(&item_for_hashing).unwrap();
        let final_item = FeeTypeGlMappingModel {
            hash: computed_hash,
            audit_log_id: Some(audit_log.id),
            ..item
        };

        sqlx::query(
            r#"
            INSERT INTO fee_type_gl_mapping
            (id, fee_type, gl_code, hash, audit_log_id)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(final_item.id)
        .bind(final_item.fee_type.clone())
        .bind(final_item.gl_code.as_str())
        .bind(final_item.hash)
        .bind(final_item.audit_log_id)
        .execute(&**pool)
        .await
        .expect("Failed to insert fee_type_gl_mapping");

        sqlx::query("INSERT INTO fee_type_gl_mapping_idx (id, fee_type, gl_code) VALUES ($1, $2, $3)")
            .bind(item_idx.id)
            .bind(item_idx.fee_type)
            .bind(item_idx.gl_code.as_str())
            .execute(&**pool)
            .await
            .expect("Failed to insert fee_type_gl_mapping index");

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let cache = repo.fee_type_gl_mapping_idx_cache.read().await;
        assert!(
            cache.contains_primary(&item_idx.id),
            "FeeTypeGlMapping should be in cache after insert"
        );
        drop(cache);

        sqlx::query("DELETE FROM fee_type_gl_mapping WHERE id = $1")
            .bind(item_idx.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete fee_type_gl_mapping");

        tokio::time::sleep(std::time::Duration::from_millis(500)).await;

        let cache = repo.fee_type_gl_mapping_idx_cache.read().await;
        assert!(
            !cache.contains_primary(&item_idx.id),
            "FeeTypeGlMapping should be removed from cache after delete"
        );

        Ok(())
    }