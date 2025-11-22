use std::error::Error;

use business_core_db::{
    models::{
        audit::{audit_link::AuditLinkModel, audit_log::AuditLogModel, entity_type::AuditEntityType},
        product::fee_type_gl_mapping::FeeTypeGlMappingModel,
    },
    utils::hash_as_i64,
};
use uuid::Uuid;

use super::repo_impl::FeeTypeGlMappingRepositoryImpl;

impl FeeTypeGlMappingRepositoryImpl {
    pub async fn delete_batch_internal(
        &self,
        ids: &[Uuid],
        audit_log_id: Option<Uuid>,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let mut tx = self.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        let audit_log_id = audit_log_id.unwrap_or_else(Uuid::new_v4);

        let items_to_delete = self.load_batch_internal(ids).await?;
        if items_to_delete.is_empty() {
            return Ok(0);
        }

        for mut item in items_to_delete {
            let previous_hash = item.hash;
            let previous_audit_log_id =
                item.audit_log_id.ok_or("Entity must have audit_log_id for update")?;

            item.antecedent_hash = previous_hash;
            item.antecedent_audit_log_id = previous_audit_log_id;
            item.audit_log_id = Some(audit_log_id);
            item.hash = 0;

            let new_computed_hash = hash_as_i64(&item)?;
            item.hash = new_computed_hash;

            let audit_insert_query = sqlx::query(
                r#"
                INSERT INTO fee_type_gl_mapping_audit
                (id, fee_type, gl_code, hash, audit_log_id, antecedent_hash, antecedent_audit_log_id)
                VALUES ($1, $2, $3, $4, $5, $6, $7)
                "#,
            )
            .bind(item.id)
            .bind(item.fee_type.clone())
            .bind(item.gl_code.as_str())
            .bind(item.hash)
            .bind(item.audit_log_id)
            .bind(item.antecedent_hash)
            .bind(item.antecedent_audit_log_id);

            let entity_delete_query = sqlx::query(
                r#"
                DELETE FROM fee_type_gl_mapping WHERE id = $1
                "#,
            )
            .bind(item.id);
            
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
            entity_delete_query.execute(&mut **transaction).await?;
            audit_link_query.execute(&mut **transaction).await?;

            let cache = self.fee_type_gl_mapping_idx_cache.read().await;
            cache.remove(&item.id);
        }

        Ok(ids.len())
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{repository::db_init::DbInitializer, test_helper::TestHelper};
    use business_core_db::models::product::fee_type_gl_mapping::FeeType;

    use super::*;

    #[tokio::test]
    async fn test_delete_batch() -> Result<(), Box<dyn Error + Send + Sync>> {
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
        let ids: Vec<Uuid> = saved_items.iter().map(|item| item.id).collect();

        let delete_audit_log = helper.create_audit_log().await?;
        let deleted_count = repo.delete_batch(&ids, Some(delete_audit_log.id)).await?;

        assert_eq!(deleted_count, 5);
        let loaded_items = repo.load_batch(&ids).await?;
        assert_eq!(loaded_items.len(), 0);

        Ok(())
    }
}