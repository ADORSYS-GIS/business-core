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
                    (id, person_type, risk_rating, status, display_name, external_identifier, id_type, id_number, entity_reference_count, organization_person_id, messaging_info1, messaging_info2, messaging_info3, messaging_info4, messaging_info5, department, location_id, duplicate_of_person_id, last_activity_log, last_compliance_status, last_document, last_portfolio, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20, $21, $22, $23, $24, $25, $26)
                    "#,
                )
                .bind(final_audit_entity.id)
                .bind(final_audit_entity.person_type)
                .bind(final_audit_entity.risk_rating)
                .bind(final_audit_entity.status)
                .bind(final_audit_entity.display_name.as_str())
                .bind(final_audit_entity.external_identifier.as_deref())
                .bind(final_audit_entity.id_type)
                .bind(final_audit_entity.id_number.as_str())
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
                .bind(final_audit_entity.last_activity_log)
                .bind(final_audit_entity.last_compliance_status)
                .bind(final_audit_entity.last_document)
                .bind(final_audit_entity.last_portfolio)
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

#[cfg(test)]
mod tests {
    use crate::repository::person::test_utils::create_test_audit_log;
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::delete_batch::DeleteBatch;
    use uuid::Uuid;
    use business_core_db::models::person::person::PersonType;
    use crate::repository::person::person_repository::test_utils::create_test_person;

    #[tokio::test]
    async fn test_delete_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let mut persons = Vec::new();
        for i in 0..3 {
            let person = create_test_person(
                &format!("Person to Delete {i}"),
                PersonType::System,
            );
            persons.push(person);
        }

        let saved = person_repo.create_batch(persons, Some(audit_log.id)).await?;

        let ids: Vec<Uuid> = saved.iter().map(|s| s.id).collect();
        // # Attention, we are deleting in the same transaction. This will not happen in a real scenario
        // in order to prevent duplicate key, we will create a new audit log for the delete.
        let delete_audit_log = create_test_audit_log();
        audit_log_repo.create(&delete_audit_log).await?;
        let deleted_count = person_repo.delete_batch(&ids, Some(delete_audit_log.id)).await?;

        assert_eq!(deleted_count, 3);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_batch_with_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let person = create_test_person(
            "Person to Delete",
            PersonType::Integration,
        );

        let saved = person_repo.create_batch(vec![person], Some(audit_log.id)).await?;

        let mut ids = vec![saved[0].id];
        ids.push(Uuid::new_v4()); // Add non-existing ID

        // # Attention, we are deleting in the same transaction. This will not happen in a real scenario
        // in order to prevent duplicate key, we will create a new audit log for the delete.
        let delete_audit_log = create_test_audit_log();
        audit_log_repo.create(&delete_audit_log).await?;
        let deleted_count = person_repo.delete_batch(&ids, Some(delete_audit_log.id)).await?;

        assert_eq!(deleted_count, 1); // Only one actually deleted

        Ok(())
    }
}