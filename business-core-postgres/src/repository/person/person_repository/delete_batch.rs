use business_core_db::models::audit::{AuditLinkModel, EntityType};
use async_trait::async_trait;
use business_core_db::repository::load_batch::LoadBatch;
use business_core_db::repository::delete_batch::DeleteBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::utils::hash_as_i64;

use super::repo_impl::PersonRepositoryImpl;

impl PersonRepositoryImpl {
    pub(super) async fn delete_batch_impl(
        repo: &PersonRepositoryImpl,
        ids: &[Uuid],
        audit_log_id: Option<Uuid>,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for PersonModel")?;
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
                    INSERT INTO person_audit
                    (id, person_type, display_name, external_identifier, entity_reference_count, organization_person_id, messaging_info1, messaging_info2, messaging_info3, messaging_info4, messaging_info5, department, location_id, duplicate_of_person_id, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
                    "#,
                )
                .bind(final_audit_entity.id)
                .bind(final_audit_entity.person_type)
                .bind(final_audit_entity.display_name.as_str())
                .bind(final_audit_entity.external_identifier.as_deref())
                .bind(final_audit_entity.entity_reference_count)
                .bind(final_audit_entity.organization_person_id)
                .bind(final_audit_entity.messaging_info1.as_deref())
                .bind(final_audit_entity.messaging_info2.as_deref())
                .bind(final_audit_entity.messaging_info3.as_deref())
                .bind(final_audit_entity.messaging_info4.as_deref())
                .bind(final_audit_entity.messaging_info5.as_deref())
                .bind(final_audit_entity.department.as_deref())
                .bind(final_audit_entity.location_id)
                .bind(final_audit_entity.duplicate_of_person_id)
                .bind(final_audit_entity.antecedent_hash)
                .bind(final_audit_entity.antecedent_audit_log_id)
                .bind(final_audit_entity.hash)
                .bind(final_audit_entity.audit_log_id)
                .execute(&mut **transaction)
                .await?;

                let result = sqlx::query(r#"DELETE FROM person WHERE id = $1"#)
                    .bind(entity.id)
                    .execute(&mut **transaction)
                    .await?;

                // Create audit link
                let audit_link = AuditLinkModel {
                    audit_log_id,
                    entity_id: entity.id,
                    entity_type: EntityType::Person,
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
            let cache = repo.person_idx_cache.read().await;
            for id in ids {
                cache.remove(id);
            }
        }
        
        Ok(deleted_count)
    }
}

#[async_trait]
impl DeleteBatch<Postgres> for PersonRepositoryImpl {
    async fn delete_batch(
        &self,
        ids: &[Uuid],
        audit_log_id: Option<Uuid>,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Self::delete_batch_impl(self, ids, audit_log_id).await
    }
}