mod repo_impl;
mod create_batch;
mod load_batch;
mod update_batch;
mod delete_batch;
mod exist_by_ids;
mod find_by_country_id;
mod find_by_country_subdivision_id;
mod find_by_rule_name_hash;
mod test_utils;


#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::load_batch::LoadBatch;
    use business_core_db::repository::update_batch::UpdateBatch;
    use business_core_db::repository::delete_batch::DeleteBatch;
    use uuid::Uuid;
    use super::test_utils::test_utils::create_test_date_calculation_rule;

    #[tokio::test]
    async fn test_create_batch_updates_main_cache() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let date_calculation_rules_repo = &ctx.calendar_repos().date_calculation_rules_repository;

        let country_id = Uuid::new_v4();
        let items = vec![
            create_test_date_calculation_rule(country_id, None, "Rule1"),
            create_test_date_calculation_rule(country_id, Some(Uuid::new_v4()), "Rule2"),
        ];
        let saved = date_calculation_rules_repo.create_batch(items, None).await?;

        // Verify entities are in main cache
        let main_cache = date_calculation_rules_repo.date_calculation_rules_cache.read().await;
        for item in &saved {
            assert!(main_cache.contains(&item.id), "Entity should be in main cache");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_load_batch_uses_cache() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let date_calculation_rules_repo = &ctx.calendar_repos().date_calculation_rules_repository;

        let country_id = Uuid::new_v4();
        let items = vec![
            create_test_date_calculation_rule(country_id, None, "Rule1"),
            create_test_date_calculation_rule(country_id, None, "Rule2"),
        ];
        let saved = date_calculation_rules_repo.create_batch(items, None).await?;
        let ids: Vec<Uuid> = saved.iter().map(|i| i.id).collect();

        // First load - should populate cache
        let loaded1 = date_calculation_rules_repo.load_batch(&ids).await?;
        
        // Second load - should hit cache
        let loaded2 = date_calculation_rules_repo.load_batch(&ids).await?;
        
        assert_eq!(loaded1.len(), loaded2.len());

        Ok(())
    }

    #[tokio::test]
    async fn test_update_batch_updates_main_cache() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let date_calculation_rules_repo = &ctx.calendar_repos().date_calculation_rules_repository;

        let country_id = Uuid::new_v4();
        let items = vec![create_test_date_calculation_rule(country_id, None, "Rule1")];
        let mut saved = date_calculation_rules_repo.create_batch(items, None).await?;
        
        saved[0].is_active = false;
        saved[0].priority = 99;
        let updated = date_calculation_rules_repo.update_batch(saved, None).await?;

        // Verify updated entity in cache
        let main_cache = date_calculation_rules_repo.date_calculation_rules_cache.read().await;
        let cached = main_cache.get(&updated[0].id);
        assert!(cached.is_some());
        let cached_value = cached.as_ref().unwrap();
        assert_eq!(cached_value.is_active, updated[0].is_active);
        assert_eq!(cached_value.priority, updated[0].priority);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_batch_removes_from_main_cache() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let date_calculation_rules_repo = &ctx.calendar_repos().date_calculation_rules_repository;

        let country_id = Uuid::new_v4();
        let items = vec![
            create_test_date_calculation_rule(country_id, None, "Rule1"),
            create_test_date_calculation_rule(country_id, Some(Uuid::new_v4()), "Rule2"),
        ];
        let saved = date_calculation_rules_repo.create_batch(items, None).await?;
        let ids: Vec<Uuid> = saved.iter().map(|i| i.id).collect();

        // Delete entities
        let deleted_count = date_calculation_rules_repo.delete_batch(&ids, None).await?;
        assert_eq!(deleted_count, ids.len());

        // Verify removed from main cache
        let main_cache = date_calculation_rules_repo.date_calculation_rules_cache.read().await;
        for id in &ids {
            assert!(!main_cache.contains(id), "Entity should be removed from main cache");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_country_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let date_calculation_rules_repo = &ctx.calendar_repos().date_calculation_rules_repository;

        let country_id = Uuid::new_v4();
        let item1 = create_test_date_calculation_rule(country_id, None, "Rule1");
        let item2 = create_test_date_calculation_rule(country_id, None, "Rule2");
        let item3 = create_test_date_calculation_rule(Uuid::new_v4(), None, "Rule3");
        
        let _saved = date_calculation_rules_repo.create_batch(vec![item1, item2, item3], None).await?;

        let found_items = date_calculation_rules_repo.find_by_country_id(country_id).await?;
        
        assert_eq!(found_items.len(), 2);
        assert!(found_items.iter().all(|i| i.country_id == Some(country_id)));

        let non_existent_country_id = Uuid::new_v4();
        let found_items = date_calculation_rules_repo.find_by_country_id(non_existent_country_id).await?;
        assert!(found_items.is_empty());

        Ok(())
    }

    #[tokio::test]
    async fn test_find_by_country_subdivision_id() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let date_calculation_rules_repo = &ctx.calendar_repos().date_calculation_rules_repository;

        let country_id = Uuid::new_v4();
        let subdivision_id = Uuid::new_v4();
        let item1 = create_test_date_calculation_rule(country_id, Some(subdivision_id), "Rule1");
        let item2 = create_test_date_calculation_rule(country_id, Some(subdivision_id), "Rule2");
        let item3 = create_test_date_calculation_rule(country_id, None, "Rule3");
        
        let _saved = date_calculation_rules_repo.create_batch(vec![item1, item2, item3], None).await?;

        let found_items = date_calculation_rules_repo.find_by_country_subdivision_id(subdivision_id).await?;
        
        assert_eq!(found_items.len(), 2);
        assert!(found_items.iter().all(|i| i.country_subdivision_id == Some(subdivision_id)));

        Ok(())
    }

    #[tokio::test]
    async fn test_date_calculation_rules_insert_triggers_main_cache_notification() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        use crate::test_helper::setup_test_context_and_listen;
        use business_core_db::models::index_aware::IndexAware;
        use tokio::time::{sleep, Duration};
        
        // Setup test context with the notification listener
        let ctx = setup_test_context_and_listen().await?;
        let pool = ctx.pool();

        // Create a test entity
        let country_id = Uuid::new_v4();
        let test_item = create_test_date_calculation_rule(country_id, None, "TestRule");
        let item_idx = test_item.to_index();

        // Give listener time to start
        sleep(Duration::from_millis(2000)).await;

        // Insert the entity record directly into database (triggers main cache notification)
        sqlx::query(
            r#"
            INSERT INTO calendar_date_calculation_rules (
                id, country_id, country_subdivision_id, rule_name, rule_purpose,
                default_shift_rule, weekend_days_id, priority, is_active,
                effective_date, expiry_date
            )
            VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
            "#,
        )
        .bind(test_item.id)
        .bind(test_item.country_id)
        .bind(test_item.country_subdivision_id)
        .bind(test_item.rule_name.as_str())
        .bind(test_item.rule_purpose)
        .bind(test_item.default_shift_rule)
        .bind(test_item.weekend_days_id)
        .bind(test_item.priority)
        .bind(test_item.is_active)
        .bind(test_item.effective_date)
        .bind(test_item.expiry_date)
        .execute(&**pool)
        .await
        .expect("Failed to insert date_calculation_rule");

        // Insert the index record directly into database (triggers index cache notification)
        sqlx::query("INSERT INTO calendar_date_calculation_rules_idx (id, country_id, country_subdivision_id, rule_name_hash) VALUES ($1, $2, $3, $4)")
            .bind(item_idx.id)
            .bind(item_idx.country_id)
            .bind(item_idx.country_subdivision_id)
            .bind(item_idx.rule_name_hash)
            .execute(&**pool)
            .await
            .expect("Failed to insert date_calculation_rule index");

        // Give time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        let date_calculation_rules_repo = &ctx.calendar_repos().date_calculation_rules_repository;

        // Verify the INDEX cache was updated
        let idx_cache = date_calculation_rules_repo.date_calculation_rules_idx_cache.read().await;
        assert!(
            idx_cache.contains_primary(&item_idx.id),
            "DateCalculationRules should be in index cache after insert"
        );
        drop(idx_cache);

        // Verify the MAIN cache was updated
        let main_cache = date_calculation_rules_repo.date_calculation_rules_cache.read().await;
        assert!(
            main_cache.contains(&test_item.id),
            "DateCalculationRules should be in main cache after insert"
        );
        drop(main_cache);

        // Delete the record from database (triggers both cache notifications)
        sqlx::query("DELETE FROM calendar_date_calculation_rules WHERE id = $1")
            .bind(test_item.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete date_calculation_rule");

        // Give time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        // Verify removed from both caches
        let idx_cache = date_calculation_rules_repo.date_calculation_rules_idx_cache.read().await;
        assert!(
            !idx_cache.contains_primary(&item_idx.id),
            "DateCalculationRules should be removed from index cache after delete"
        );
        drop(idx_cache);

        let main_cache = date_calculation_rules_repo.date_calculation_rules_cache.read().await;
        assert!(
            !main_cache.contains(&test_item.id),
            "DateCalculationRules should be removed from main cache after delete"
        );

        Ok(())
    }
}
pub use repo_impl::DateCalculationRulesRepositoryImpl;