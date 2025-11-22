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
    pub async fn delete_batch_internal(
        &self,
        ids: &[Uuid],
        audit_log_id: Option<Uuid>,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let mut tx = self.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        let audit_log_id = audit_log_id.unwrap_or_else(Uuid::new_v4);

        let items_to_delete = self.load_batch(ids).await?;
        for item in &items_to_delete {
            let mut final_audit_entity = item.clone();

            final_audit_entity.antecedent_hash = item.hash;
            final_audit_entity.antecedent_audit_log_id = item
                .audit_log_id
                .ok_or("Entity must have audit_log_id for deletion")?;
            final_audit_entity.audit_log_id = Some(audit_log_id);
            final_audit_entity.hash = 0;

            let final_hash = hash_as_i64(&final_audit_entity)?;
            final_audit_entity.hash = final_hash;

            let audit_insert_query = sqlx::query(
                r#"
                INSERT INTO account_gl_mapping_audit
                (id, customer_account_code, overdraft_code, hash, audit_log_id, antecedent_hash, antecedent_audit_log_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(final_audit_entity.id)
            .bind(final_audit_entity.customer_account_code.as_str())
            .bind(final_audit_entity.overdraft_code.as_deref())
            .bind(final_audit_entity.hash)
            .bind(final_audit_entity.audit_log_id)
            .bind(final_audit_entity.antecedent_hash)
            .bind(final_audit_entity.antecedent_audit_log_id);

            let entity_delete_query =
                sqlx::query("DELETE FROM account_gl_mapping WHERE id = $1").bind(item.id);

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
            entity_delete_query.execute(&mut **transaction).await?;
            audit_link_query.execute(&mut **transaction).await?;
        }

        let cache = self.account_gl_mapping_idx_cache.read().await;
        for id in ids {
            cache.remove(id);
        }

        Ok(ids.len())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{repository::db_init::DbInitializer, test_helper::TestHelper};

    use super::*;

    #[tokio::test]
    async fn test_delete_batch() -> Result<(), Box<dyn Error + Send + Sync>> {
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
        let ids: Vec<Uuid> = saved_items.iter().map(|item| item.id).collect();

        let delete_audit_log = helper.create_audit_log().await?;
        let deleted_count = repo.delete_batch(&ids, Some(delete_audit_log.id)).await?;
        assert_eq!(deleted_count, 5);

        let loaded_items = repo.load_batch(&ids).await?;
        assert_eq!(loaded_items.len(), 0);

        Ok(())
    }
}