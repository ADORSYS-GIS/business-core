use async_trait::async_trait;
use business_core_db::repository::load_batch::LoadBatch;
use business_core_db::repository::delete_batch::DeleteBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::utils::hash_as_i64;

use super::repo_impl::LocationRepositoryImpl;

impl LocationRepositoryImpl {
    pub(super) async fn delete_batch_impl(
        repo: &LocationRepositoryImpl,
        ids: &[Uuid],
        audit_log_id: Uuid,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(0);
        }

        let entities_to_delete = repo.load_batch(ids).await?;
        let mut deleted_count = 0;

        {
            let mut tx = repo.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;

            for entity_option in entities_to_delete {
                if let Some(entity) = entity_option {
                    let mut final_audit_entity = entity.clone();
                    final_audit_entity.antecedent_hash = entity.hash;
                    final_audit_entity.antecedent_audit_log_id = entity.audit_log_id.ok_or("Entity must have audit_log_id for deletion")?;
                    final_audit_entity.audit_log_id = Some(audit_log_id);
                    final_audit_entity.hash = 0;

                    let final_hash = hash_as_i64(&final_audit_entity)?;
                    final_audit_entity.hash = final_hash;

                    sqlx::query(
                        r#"
                        INSERT INTO location_audit
                        (id, street_line1, street_line2, street_line3, street_line4, locality_id, postal_code, latitude, longitude, accuracy_meters, location_type, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
                        "#,
                    )
                    .bind(final_audit_entity.id)
                    .bind(final_audit_entity.street_line1.as_str())
                    .bind(final_audit_entity.street_line2.as_deref())
                    .bind(final_audit_entity.street_line3.as_deref())
                    .bind(final_audit_entity.street_line4.as_deref())
                    .bind(final_audit_entity.locality_id)
                    .bind(final_audit_entity.postal_code.as_deref())
                    .bind(final_audit_entity.latitude)
                    .bind(final_audit_entity.longitude)
                    .bind(final_audit_entity.accuracy_meters)
                    .bind(final_audit_entity.location_type)
                    .bind(final_audit_entity.antecedent_hash)
                    .bind(final_audit_entity.antecedent_audit_log_id)
                    .bind(final_audit_entity.hash)
                    .bind(final_audit_entity.audit_log_id)
                    .execute(&mut **transaction)
                    .await?;

                    let result = sqlx::query(r#"DELETE FROM location WHERE id = $1"#)
                        .bind(entity.id)
                        .execute(&mut **transaction)
                        .await?;
                    
                    deleted_count += result.rows_affected() as usize;
                }
            }
        }
        
        {
            let cache = repo.location_idx_cache.read().await;
            for id in ids {
                cache.remove(id);
            }
        }
        
        Ok(deleted_count)
    }
}

#[async_trait]
impl DeleteBatch<Postgres> for LocationRepositoryImpl {
    async fn delete_batch(
        &self,
        ids: &[Uuid],
        audit_log_id: Uuid,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Self::delete_batch_impl(self, ids, audit_log_id).await
    }
}