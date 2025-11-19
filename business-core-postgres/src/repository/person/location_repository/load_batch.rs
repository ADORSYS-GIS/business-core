use async_trait::async_trait;
use business_core_db::models::person::location::LocationModel;
use business_core_db::repository::load_batch::LoadBatch;
use crate::utils::TryFromRow;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::LocationRepositoryImpl;

impl LocationRepositoryImpl {
    pub(super) async fn load_batch_impl(
        repo: &LocationRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<Option<LocationModel>>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        
        let query = r#"SELECT * FROM location WHERE id = ANY($1)"#;
        let rows = {
            let mut tx = repo.executor.tx.lock().await;
            if let Some(transaction) = tx.as_mut() {
                sqlx::query(query).bind(ids).fetch_all(&mut **transaction).await?
            } else {
                return Err("Transaction has been consumed".into());
            }
        };
        
        let mut item_map = std::collections::HashMap::new();
        for row in rows {
            let item = LocationModel::try_from_row(&row)?;
            item_map.insert(item.id, item);
        }
        
        let mut result = Vec::with_capacity(ids.len());
        for id in ids {
            result.push(item_map.remove(id));
        }
        Ok(result)
    }
}

#[async_trait]
impl LoadBatch<Postgres, LocationModel> for LocationRepositoryImpl {
    async fn load_batch(&self, ids: &[Uuid]) -> Result<Vec<Option<LocationModel>>, Box<dyn Error + Send + Sync>> {
        Self::load_batch_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::load_batch::LoadBatch;
    use uuid::Uuid;
    use crate::repository::person::test_utils::{create_test_audit_log, create_test_country, create_test_country_subdivision, create_test_locality, create_test_location};

    #[tokio::test]
    async fn test_load_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let country_repo = &ctx.person_repos().country_repository;
        let country_subdivision_repo = &ctx.person_repos().country_subdivision_repository;
        let locality_repo = &ctx.person_repos().locality_repository;
        let location_repo = &ctx.person_repos().location_repository;

        // First create a country (required by foreign key constraint)
        let country = create_test_country("IT", "Italy");
        let country_id = country.id;
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        country_repo.create_batch(vec![country], Some(audit_log.id)).await?;

        // Create a country subdivision (required by foreign key constraint)
        let subdivision = create_test_country_subdivision(country_id, "RM", "Rome");
        let subdivision_id = subdivision.id;
        country_subdivision_repo.create_batch(vec![subdivision], Some(audit_log.id)).await?;

        // Create a locality (required by foreign key constraint)
        let locality = create_test_locality(subdivision_id, "RM", "Rome");
        let locality_id = locality.id;
        locality_repo.create_batch(vec![locality], Some(audit_log.id)).await?;

        let mut locations = Vec::new();
        for i in 0..3 {
            let location = create_test_location(
                locality_id,
                &format!("{i} Via del Corso"),
            );
            locations.push(location);
        }

        let saved = location_repo.create_batch(locations.clone(), Some(audit_log.id)).await?;

        let ids: Vec<Uuid> = saved.iter().map(|s| s.id).collect();
        let loaded = location_repo.load_batch(&ids).await?;

        assert_eq!(loaded.len(), 3);
        for item in loaded {
            assert!(item.is_some());
            let location = item.unwrap();
            assert_eq!(location.locality_id, locality_id);
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_load_batch_with_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let country_repo = &ctx.person_repos().country_repository;
        let country_subdivision_repo = &ctx.person_repos().country_subdivision_repository;
        let locality_repo = &ctx.person_repos().locality_repository;
        let location_repo = &ctx.person_repos().location_repository;

        // First create a country (required by foreign key constraint)
        let country = create_test_country("ES", "Spain");
        let country_id = country.id;
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        country_repo.create_batch(vec![country], Some(audit_log.id)).await?;

        // Create a country subdivision (required by foreign key constraint)
        let subdivision = create_test_country_subdivision(country_id, "MD", "Madrid");
        let subdivision_id = subdivision.id;
        country_subdivision_repo.create_batch(vec![subdivision], Some(audit_log.id)).await?;

        // Create a locality (required by foreign key constraint)
        let locality = create_test_locality(subdivision_id, "MD", "Madrid");
        let locality_id = locality.id;
        locality_repo.create_batch(vec![locality], Some(audit_log.id)).await?;

        let location = create_test_location(
            locality_id,
            "1 Gran Vía",
        );

        let saved = location_repo.create_batch(vec![location], Some(audit_log.id)).await?;

        let mut ids = vec![saved[0].id];
        ids.push(Uuid::new_v4()); // Add non-existing ID

        let loaded = location_repo.load_batch(&ids).await?;

        assert_eq!(loaded.len(), 2);
        assert!(loaded[0].is_some());
        assert!(loaded[1].is_none());

        Ok(())
    }
}