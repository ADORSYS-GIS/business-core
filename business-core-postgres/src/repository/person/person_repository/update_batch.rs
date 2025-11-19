use async_trait::async_trait;
use business_core_db::models::{
    audit::{AuditLinkModel, EntityType},
    person::person::PersonModel,
};
use business_core_db::models::index_aware::IndexAware;
use business_core_db::repository::update_batch::UpdateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::utils::hash_as_i64;

use super::repo_impl::PersonRepositoryImpl;

impl PersonRepositoryImpl {
    pub(super) async fn update_batch_impl(
        &self,
        items: Vec<PersonModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<PersonModel>, Box<dyn Error + Send + Sync>> {
        let audit_log_id = audit_log_id.ok_or("audit_log_id is required for PersonModel")?;
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
                    INSERT INTO person_audit
                    (id, person_type, display_name, external_identifier, id_type, id_number, entity_reference_count, organization_person_id, messaging_info1, messaging_info2, messaging_info3, messaging_info4, messaging_info5, department, location_id, duplicate_of_person_id, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
                    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14, $15, $16, $17, $18, $19, $20)
                    "#,
                )
                .bind(item.id)
                .bind(item.person_type)
                .bind(item.display_name.as_str())
                .bind(item.external_identifier.as_deref())
                .bind(item.id_type)
                .bind(item.id_number.as_str())
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

                let rows_affected = sqlx::query(
                    r#"
                    UPDATE person SET
                    person_type = $2, display_name = $3, external_identifier = $4,
                    id_type = $5, id_number = $6,
                    entity_reference_count = $7, organization_person_id = $8,
                    messaging_info1 = $9, messaging_info2 = $10, messaging_info3 = $11,
                    messaging_info4 = $12, messaging_info5 = $13, department = $14,
                    location_id = $15, duplicate_of_person_id = $16,
                    antecedent_hash = $17, antecedent_audit_log_id = $18,
                    hash = $19, audit_log_id = $20
                    WHERE id = $1 AND hash = $21 AND audit_log_id = $22
                    "#,
                )
                .bind(item.id)
                .bind(item.person_type)
                .bind(item.display_name.as_str())
                .bind(item.external_identifier.as_deref())
                .bind(item.id_type)
                .bind(item.id_number.as_str())
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
                    UPDATE person_idx SET
                    external_identifier_hash = $2,
                    organization_person_id = $3,
                    duplicate_of_person_id = $4,
                    id_number_hash = $5
                    WHERE id = $1
                    "#,
                )
                .bind(idx.id)
                .bind(idx.external_identifier_hash)
                .bind(idx.organization_person_id)
                .bind(idx.duplicate_of_person_id)
                .bind(idx.id_number_hash)
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

                indices_to_update.push((item.id, idx));
                updated_items.push(item);
            }
        }
        
        {
            let cache = self.person_idx_cache.read().await;
            for (id, idx) in indices_to_update {
                cache.remove(&id);
                cache.add(idx);
            }
        }

        Ok(updated_items)
    }
}

#[async_trait]
impl UpdateBatch<Postgres, PersonModel> for PersonRepositoryImpl {
    async fn update_batch(
        &self,
        items: Vec<PersonModel>,
        audit_log_id: Option<Uuid>,
    ) -> Result<Vec<PersonModel>, Box<dyn Error + Send + Sync>> {
        Self::update_batch_impl(self, items, audit_log_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::repository::person::test_utils::create_test_audit_log;
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::update_batch::UpdateBatch;
    use heapless::String as HeaplessString;
    use business_core_db::models::person::person::PersonType;
    use crate::repository::person::person_repository::test_utils::create_test_person;

    #[tokio::test]
    async fn test_update_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let mut persons = Vec::new();
        for i in 0..3 {
            let person = create_test_person(
                &format!("Original Person {i}"),
                PersonType::Natural,
            );
            persons.push(person);
        }

        let saved = person_repo.create_batch(persons, Some(audit_log.id)).await?;

        // Update persons
        // # Attention, we are updating in the same transaction. This will not happen in a real scenario
        // in order to prevent duplicate key, we will create a new audit log for the update.
        let update_audit_log = create_test_audit_log();
        audit_log_repo.create(&update_audit_log).await?;
        let mut updated_persons = Vec::new();
        for mut person in saved {
            person.display_name = HeaplessString::try_from("Updated Person").unwrap();
            updated_persons.push(person);
        }

        let updated = person_repo.update_batch(updated_persons, Some(update_audit_log.id)).await?;

        assert_eq!(updated.len(), 3);
        for person in updated {
            assert_eq!(person.display_name.as_str(), "Updated Person");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_update_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let person_repo = &ctx.person_repos().person_repository;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        let updated = person_repo.update_batch(Vec::new(), Some(audit_log.id)).await?;

        assert_eq!(updated.len(), 0);

        Ok(())
    }
}