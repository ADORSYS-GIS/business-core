use business_core_db::models::audit::{AuditLinkModel, AuditEntityType};
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
        audit_log_id: Option<Uuid>,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for LocationModel")?;
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

                // Create audit link
                let audit_link = AuditLinkModel {
                    audit_log_id,
                    entity_id: entity.id,
                    entity_type: AuditEntityType::Location,
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
        audit_log_id: Option<Uuid>,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Self::delete_batch_impl(self, ids, audit_log_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::delete_batch::DeleteBatch;
    use uuid::Uuid;
    use crate::repository::person::test_utils::{create_test_audit_log, create_test_country, create_test_country_subdivision, create_test_locality, create_test_location};

    #[tokio::test]
    async fn test_delete_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let country_repo = &ctx.person_repos().country_repository;
        let country_subdivision_repo = &ctx.person_repos().country_subdivision_repository;
        let locality_repo = &ctx.person_repos().locality_repository;
        let location_repo = &ctx.person_repos().location_repository;

        // First create a country (required by foreign key constraint)
        let country = create_test_country("US", "United States");
        let country_id = country.id;
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        country_repo.create_batch(vec![country], Some(audit_log.id)).await?;

        // Create a country subdivision (required by foreign key constraint)
        let subdivision = create_test_country_subdivision(country_id, "CA", "California");
        let subdivision_id = subdivision.id;
        country_subdivision_repo.create_batch(vec![subdivision], Some(audit_log.id)).await?;

        // Create a locality (required by foreign key constraint)
        let locality = create_test_locality(subdivision_id, "LA", "Los Angeles");
        let locality_id = locality.id;
        locality_repo.create_batch(vec![locality], Some(audit_log.id)).await?;

        let mut locations = Vec::new();
        for i in 0..3 {
            let location = create_test_location(
                locality_id,
                &format!("{i} Sunset Blvd"),
            );
            locations.push(location);
        }

        let saved = location_repo.create_batch(locations, Some(audit_log.id)).await?;

        let ids: Vec<Uuid> = saved.iter().map(|s| s.id).collect();
        // # Attention, we are deleting in the same transaction. This will not happen in a real scenario
        // in order to prevent duplicate key, we will create a new audit log for the delete.
        let delete_audit_log = create_test_audit_log();
        audit_log_repo.create(&delete_audit_log).await?;
        let deleted_count = location_repo.delete_batch(&ids, Some(delete_audit_log.id)).await?;

        assert_eq!(deleted_count, 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_batch_with_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let country_repo = &ctx.person_repos().country_repository;
        let country_subdivision_repo = &ctx.person_repos().country_subdivision_repository;
        let locality_repo = &ctx.person_repos().locality_repository;
        let location_repo = &ctx.person_repos().location_repository;

        // First create a country (required by foreign key constraint)
        let country = create_test_country("CA", "Canada");
        let country_id = country.id;
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        country_repo.create_batch(vec![country], Some(audit_log.id)).await?;

        // Create a country subdivision (required by foreign key constraint)
        let subdivision = create_test_country_subdivision(country_id, "ON", "Ontario");
        let subdivision_id = subdivision.id;
        country_subdivision_repo.create_batch(vec![subdivision], Some(audit_log.id)).await?;

        // Create a locality (required by foreign key constraint)
        let locality = create_test_locality(subdivision_id, "TO", "Toronto");
        let locality_id = locality.id;
        locality_repo.create_batch(vec![locality], Some(audit_log.id)).await?;

        let location = create_test_location(
            locality_id,
            "123 Yonge St",
        );

        let saved = location_repo.create_batch(vec![location], Some(audit_log.id)).await?;

        let mut ids = vec![saved[0].id];
        ids.push(Uuid::new_v4()); // Add non-existing ID

        // # Attention, we are deleting in the same transaction. This will not happen in a real scenario
        // in order to prevent duplicate key, we will create a new audit log for the delete.
        let delete_audit_log = create_test_audit_log();
        audit_log_repo.create(&delete_audit_log).await?;
        let deleted_count = location_repo.delete_batch(&ids, Some(delete_audit_log.id)).await?;

        assert_eq!(deleted_count, 1); // Only one actually deleted

        Ok(())
    }
}