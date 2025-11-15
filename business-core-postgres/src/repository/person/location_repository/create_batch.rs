use async_trait::async_trait;
use business_core_db::models::person::location::LocationModel;
use business_core_db::repository::create_batch::CreateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::models::index_aware::IndexAware;
use business_core_db::utils::hash_as_i64;

use super::repo_impl::LocationRepositoryImpl;

impl LocationRepositoryImpl {
    pub(super) async fn create_batch_impl(
        repo: &LocationRepositoryImpl,
        items: Vec<LocationModel>,
        audit_log_id: Uuid,
    ) -> Result<Vec<LocationModel>, Box<dyn Error + Send + Sync>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut saved_items = Vec::new();
        let mut indices = Vec::new();
        
        // Acquire lock once and do all database operations
        {
            let mut tx = repo.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
            
            for mut item in items {
                // 1. Create a copy of entity for hashing
                let mut entity_for_hashing = item.clone();
                entity_for_hashing.hash = 0;  // Must be 0 before hashing
                entity_for_hashing.audit_log_id = Some(audit_log_id); // Set ID before hashing

                // 2. Compute hash
                let computed_hash = hash_as_i64(&entity_for_hashing)?;

                // 3. Update original entity with computed hash and new audit_log_id
                item.hash = computed_hash;
                item.audit_log_id = Some(audit_log_id);

                // Execute audit insert
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

                // Execute main insert
                sqlx::query(
                    r#"
                    INSERT INTO location
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

                // Insert into index table
                let idx = item.to_index();
                sqlx::query(
                    r#"
                    INSERT INTO location_idx (id, locality_id)
                    VALUES ($1, $2)
                    "#,
                )
                .bind(idx.id)
                .bind(idx.locality_id)
                .execute(&mut **transaction)
                .await?;

                indices.push(idx);
                saved_items.push(item);
            }
        } // Transaction lock released here
        
        // Update cache after releasing transaction lock
        {
            let cache = repo.location_idx_cache.read().await;
            for idx in indices {
                cache.add(idx);
            }
        }

        Ok(saved_items)
    }
}

#[async_trait]
impl CreateBatch<Postgres, LocationModel> for LocationRepositoryImpl {
    async fn create_batch(
        &self,
        items: Vec<LocationModel>,
        audit_log_id: Uuid,
    ) -> Result<Vec<LocationModel>, Box<dyn Error + Send + Sync>> {
        Self::create_batch_impl(self, items, audit_log_id).await
    }
}