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
    pub async fn update_batch_internal(
        &self,
        items: Vec<AccountGlMappingModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<AccountGlMappingModel>, Box<dyn Error + Send + Sync>> {
        let mut tx = self.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        let audit_log_id = audit_log_id.unwrap_or_else(Uuid::new_v4);

        let mut res: Vec<AccountGlMappingModel> = Vec::with_capacity(items.len());

        for mut item in items {
            let previous_hash = item.hash;
            let previous_audit_log_id = item
                .audit_log_id
                .ok_or("Entity must have audit_log_id for update")?;

            let mut entity_for_hashing = item.clone();
            entity_for_hashing.hash = 0;

            let computed_hash = hash_as_i64(&entity_for_hashing)?;

            if computed_hash == previous_hash {
                res.push(item);
                continue;
            }

            item.antecedent_hash = previous_hash;
            item.antecedent_audit_log_id = previous_audit_log_id;
            item.audit_log_id = Some(audit_log_id);
            item.hash = 0;

            let new_computed_hash = hash_as_i64(&item)?;
            item.hash = new_computed_hash;

            let audit_insert_query = sqlx::query(
                r#"
                INSERT INTO account_gl_mapping_audit
                (id, customer_account_code, overdraft_code, hash, audit_log_id, antecedent_hash, antecedent_audit_log_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(item.id)
            .bind(item.customer_account_code.as_str())
            .bind(item.overdraft_code.as_deref())
            .bind(item.hash)
            .bind(item.audit_log_id)
            .bind(item.antecedent_hash)
            .bind(item.antecedent_audit_log_id);

            let entity_update_query = sqlx::query(
                r#"
                UPDATE account_gl_mapping SET
                customer_account_code = $2,
                overdraft_code = $3,
                hash = $4,
                audit_log_id = $5,
                antecedent_hash = $6,
                antecedent_audit_log_id = $7
                WHERE id = $1 AND hash = $6 AND audit_log_id = $7
                "#,
            )
            .bind(item.id)
            .bind(item.customer_account_code.as_str())
            .bind(item.overdraft_code.as_deref())
            .bind(item.hash)
            .bind(item.audit_log_id)
            .bind(item.antecedent_hash)
            .bind(item.antecedent_audit_log_id);

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
            entity_update_query.execute(&mut **transaction).await?;
            audit_link_query.execute(&mut **transaction).await?;

            res.push(item);
        }
        let cache = self.account_gl_mapping_idx_cache.read().await;
        res.iter().for_each(|item| {
            cache.remove(&item.id);
            cache.add(item.to_index());
        });
        Ok(res)
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{repository::db_init::DbInitializer, test_helper::TestHelper};

    use super::*;

    #[tokio::test]
    async fn test_update_batch() -> Result<(), Box<dyn Error + Send + Sync>> {
        let db = DbInitializer::init_test_db().await?;
        let helper = TestHelper::new(db.clone());
        let audit_log = helper.create_audit_log().await?;
        let repo = AccountGlMappingRepositoryImpl::new(db.clone());

        let mut items = vec![];
        for i in 0..5 {
            let item = super::super::test_utils::create_test_account_gl_mapping(
                &format!("CUST{}", i),
                Some(&format!("OD{}", i)),
            );
            items.push(item);
        }
        let saved_items = repo.create_batch(items, Some(audit_log.id)).await?;

        let update_audit_log = helper.create_audit_log().await?;
        let mut updated_items = vec![];
        for mut item in saved_items {
            item.customer_account_code = "UPDATED".try_into().unwrap();
            updated_items.push(item);
        }

        let updated = repo
            .update_batch(updated_items, Some(update_audit_log.id))
            .await?;

        assert_eq!(updated.len(), 5);
        for item in updated {
            assert_eq!(item.customer_account_code.as_str(), "UPDATED");
        }

        Ok(())
    }
}