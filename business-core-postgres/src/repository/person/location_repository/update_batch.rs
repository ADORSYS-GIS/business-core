use async_trait::async_trait;
use business_core_db::models::person::location::LocationModel;
use business_core_db::models::index_aware::IndexAware;
use business_core_db::repository::update_batch::UpdateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::utils::hash_as_i64;

use super::repo_impl::LocationRepositoryImpl;

impl LocationRepositoryImpl {
    pub(super) async fn update_batch_impl(
        &self,
        items: Vec<LocationModel>,
        audit_log_id: Uuid,
    ) -> Result<Vec<LocationModel>, Box<dyn Error + Send + Sync>> {
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
                    INSERT INTO location_audit
                    (id, street_line1, street_line2, street_line3, street_line4, locality_id, postal_code, latitude, longitude, accuracy_meters, location_type, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15)
                    "#,
                )
                .bind(item.id)
                .bind(item.street_line1.as_str())
                .bind(item.street_line2.as_deref())
                .bind(item.street_line3.as_deref())
                .bind(item.street_line4.as_deref())
                .bind(item.locality_id)
                .bind(item.postal_code.as_deref())
                .bind(item.latitude)
                .bind(item.longitude)
                .bind(item.accuracy_meters)
                .bind(item.location_type)
                .bind(item.antecedent_hash)
                .bind(item.antecedent_audit_log_id)
                .bind(item.hash)
                .bind(item.audit_log_id)
                .execute(&mut **transaction)
                .await?;

                let rows_affected = sqlx::query(
                    r#"
                    UPDATE location SET
                    street_line1 = $2, street_line2 = $3, street_line3 = $4, street_line4 = $5,
                    locality_id = $6, postal_code = $7, latitude = $8, longitude = $9,
                    accuracy_meters = $10, location_type = $11, antecedent_hash = $12,
                    antecedent_audit_log_id = $13, hash = $14, audit_log_id = $15
                    WHERE id = $1 AND hash = $16 AND audit_log_id = $17
                    "#,
                )
                .bind(item.id)
                .bind(item.street_line1.as_str())
                .bind(item.street_line2.as_deref())
                .bind(item.street_line3.as_deref())
                .bind(item.street_line4.as_deref())
                .bind(item.locality_id)
                .bind(item.postal_code.as_deref())
                .bind(item.latitude)
                .bind(item.longitude)
                .bind(item.accuracy_meters)
                .bind(item.location_type)
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
                    UPDATE location_idx SET locality_id = $2 WHERE id = $1
                    "#,
                )
                .bind(idx.id)
                .bind(idx.locality_id)
                .execute(&mut **transaction)
                .await?;

                indices_to_update.push((item.id, idx));
                updated_items.push(item);
            }
        }
        
        {
            let cache = self.location_idx_cache.read().await;
            for (id, idx) in indices_to_update {
                cache.remove(&id);
                cache.add(idx);
            }
        }

        Ok(updated_items)
    }
}

#[async_trait]
impl UpdateBatch<Postgres, LocationModel> for LocationRepositoryImpl {
    async fn update_batch(
        &self,
        items: Vec<LocationModel>,
        audit_log_id: Uuid,
    ) -> Result<Vec<LocationModel>, Box<dyn Error + Send + Sync>> {
        Self::update_batch_impl(self, items, audit_log_id).await
    }
}