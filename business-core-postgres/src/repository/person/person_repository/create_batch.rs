use async_trait::async_trait;
use business_core_db::models::{
    audit::{AuditLinkModel, EntityType},
    person::person::PersonModel,
};
use business_core_db::repository::create_batch::CreateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::models::index_aware::IndexAware;
use business_core_db::utils::hash_as_i64;

use super::repo_impl::PersonRepositoryImpl;

impl PersonRepositoryImpl {
    pub(super) async fn create_batch_impl(
        repo: &PersonRepositoryImpl,
        items: Vec<PersonModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<PersonModel>, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for PersonModel")?;
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
                    INSERT INTO person_audit
                    (id, person_type, display_name, external_identifier, entity_reference_count, organization_person_id, messaging_info1, messaging_info2, messaging_info3, messaging_info4, messaging_info5, department, location_id, duplicate_of_person_id, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
                    "#,
                )
                .bind(item.id)
                .bind(item.person_type)
                .bind(item.display_name.as_str())
                .bind(item.external_identifier.as_deref())
                .bind(item.entity_reference_count)
                .bind(item.organization_person_id)
                .bind(item.messaging_info1.as_deref())
                .bind(item.messaging_info2.as_deref())
                .bind(item.messaging_info3.as_deref())
                .bind(item.messaging_info4.as_deref())
                .bind(item.messaging_info5.as_deref())
                .bind(item.department.as_deref())
                .bind(item.location_id)
                .bind(item.duplicate_of_person_id)
                .bind(item.antecedent_hash)
                .bind(item.antecedent_audit_log_id)
                .bind(item.hash)
                .bind(item.audit_log_id)
                .execute(&mut **transaction)
                .await?;

                // Execute main insert
                sqlx::query(
                    r#"
                    INSERT INTO person
                    (id, person_type, display_name, external_identifier, entity_reference_count, organization_person_id, messaging_info1, messaging_info2, messaging_info3, messaging_info4, messaging_info5, department, location_id, duplicate_of_person_id, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18)
                    "#,
                )
                .bind(item.id)
                .bind(item.person_type)
                .bind(item.display_name.as_str())
                .bind(item.external_identifier.as_deref())
                .bind(item.entity_reference_count)
                .bind(item.organization_person_id)
                .bind(item.messaging_info1.as_deref())
                .bind(item.messaging_info2.as_deref())
                .bind(item.messaging_info3.as_deref())
                .bind(item.messaging_info4.as_deref())
                .bind(item.messaging_info5.as_deref())
                .bind(item.department.as_deref())
                .bind(item.location_id)
                .bind(item.duplicate_of_person_id)
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
                    INSERT INTO person_idx (id, external_identifier_hash, organization_person_id, duplicate_of_person_id)
                    VALUES ($1, $2, $3, $4)
                    "#,
                )
                .bind(idx.id)
                .bind(idx.external_identifier_hash)
                .bind(idx.organization_person_id)
                .bind(idx.duplicate_of_person_id)
                .execute(&mut **transaction)
                .await?;

                // Create audit link
                let audit_link = AuditLinkModel {
                    audit_log_id,
                    entity_id: item.id,
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

                indices.push(idx);
                saved_items.push(item);
            }
        } // Transaction lock released here
        
        // Update cache after releasing transaction lock
        {
            let cache = repo.person_idx_cache.read().await;
            for idx in indices {
                cache.add(idx);
            }
        }

        Ok(saved_items)
    }
}

#[async_trait]
impl CreateBatch<Postgres, PersonModel> for PersonRepositoryImpl {
    async fn create_batch(
        &self,
        items: Vec<PersonModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<PersonModel>, Box<dyn Error + Send + Sync>> {
        Self::create_batch_impl(self, items, audit_log_id).await
    }
}