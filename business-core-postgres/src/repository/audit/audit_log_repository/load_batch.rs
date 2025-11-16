use business_core_db::models::audit::AuditLogModel;
use postgres_unit_of_work::Executor;
use uuid::Uuid;

pub async fn load_batch_impl(
    executor: &Executor,
    ids: &[Uuid],
) -> Result<Vec<Option<AuditLogModel>>, Box<dyn std::error::Error + Send + Sync>> {
    if ids.is_empty() {
        return Ok(vec![]);
    }

    let query = sqlx::query_as::<_, AuditLogModel>(
        r#"
        SELECT id, updated_at, updated_by_person_id
        FROM audit_log
        WHERE id = ANY($1)
        "#,
    )
    .bind(ids);

    // Execute query using the new executor structure
    let mut tx = executor.tx.lock().await;
    let rows = if let Some(transaction) = tx.as_mut() {
        query.fetch_all(&mut **transaction).await?
    } else {
        return Err("Transaction has been consumed".into());
    };

    // Build a map of id -> model
    let mut map: std::collections::HashMap<Uuid, AuditLogModel> = rows
        .into_iter()
        .map(|model| (model.id, model))
        .collect();

    // Return results in the same order as input ids
    let result = ids
        .iter()
        .map(|id| map.remove(id))
        .collect();

    Ok(result)
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use crate::repository::person::test_utils::create_test_audit_log;
        use business_core_db::repository::load_batch::LoadBatch;
        use uuid::Uuid;
    
        #[tokio::test]
    async fn test_load_batch_empty_ids() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;

        let result = audit_log_repo.load_batch(&[]).await?;

        assert!(result.is_empty());
        Ok(())
    }

    #[tokio::test]
    async fn test_load_batch_single_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;

        // Create an audit log
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Load it via batch
        let result = audit_log_repo.load_batch(&[audit_log.id]).await?;

        assert_eq!(result.len(), 1);
        assert!(result[0].is_some());
        let loaded = result[0].as_ref().unwrap();
        assert_eq!(loaded.id, audit_log.id);
        assert_eq!(loaded.updated_by_person_id, audit_log.updated_by_person_id);

        Ok(())
    }

    #[tokio::test]
    async fn test_load_batch_single_nonexistent() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;

        let nonexistent_id = Uuid::new_v4();
        let result = audit_log_repo.load_batch(&[nonexistent_id]).await?;

        assert_eq!(result.len(), 1);
        assert!(result[0].is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_load_batch_multiple_all_exist() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;

        // Create multiple audit logs
        let audit_log1 = create_test_audit_log();
                let audit_log2 = create_test_audit_log();
                let audit_log3 = create_test_audit_log();

        audit_log_repo.create(&audit_log1).await?;
        audit_log_repo.create(&audit_log2).await?;
        audit_log_repo.create(&audit_log3).await?;

        // Load them via batch
        let ids = vec![audit_log1.id, audit_log2.id, audit_log3.id];
        let result = audit_log_repo.load_batch(&ids).await?;

        assert_eq!(result.len(), 3);
        assert!(result[0].is_some());
        assert!(result[1].is_some());
        assert!(result[2].is_some());

        // Verify order is preserved
        assert_eq!(result[0].as_ref().unwrap().id, audit_log1.id);
        assert_eq!(result[1].as_ref().unwrap().id, audit_log2.id);
        assert_eq!(result[2].as_ref().unwrap().id, audit_log3.id);

        Ok(())
    }

    #[tokio::test]
    async fn test_load_batch_mixed_existing_and_nonexistent() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;

        // Create one audit log
        let audit_log1 = create_test_audit_log();
        audit_log_repo.create(&audit_log1).await?;

        // Mix existing and non-existing IDs
        let nonexistent_id = Uuid::new_v4();
        let ids = vec![audit_log1.id, nonexistent_id];
        let result = audit_log_repo.load_batch(&ids).await?;

        assert_eq!(result.len(), 2);
        assert!(result[0].is_some());
        assert!(result[1].is_none());
        assert_eq!(result[0].as_ref().unwrap().id, audit_log1.id);

        Ok(())
    }

    #[tokio::test]
    async fn test_load_batch_preserves_order() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;

        // Create multiple audit logs
        let audit_log1 = create_test_audit_log();
                let audit_log2 = create_test_audit_log();
                let audit_log3 = create_test_audit_log();

        audit_log_repo.create(&audit_log1).await?;
        audit_log_repo.create(&audit_log2).await?;
        audit_log_repo.create(&audit_log3).await?;

        // Request in a specific order different from creation order
        let ids = vec![audit_log3.id, audit_log1.id, audit_log2.id];
        let result = audit_log_repo.load_batch(&ids).await?;

        assert_eq!(result.len(), 3);
        // Verify the order matches the request order
        assert_eq!(result[0].as_ref().unwrap().id, audit_log3.id);
        assert_eq!(result[1].as_ref().unwrap().id, audit_log1.id);
        assert_eq!(result[2].as_ref().unwrap().id, audit_log2.id);

        Ok(())
    }

    #[tokio::test]
    async fn test_load_batch_duplicate_ids() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;

        // Create an audit log
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Request the same ID multiple times
        let ids = vec![audit_log.id, audit_log.id, audit_log.id];
        let result = audit_log_repo.load_batch(&ids).await?;

        assert_eq!(result.len(), 3);
        // First occurrence should be found
        assert!(result[0].is_some());
        assert_eq!(result[0].as_ref().unwrap().id, audit_log.id);
        // Subsequent occurrences should be None (since the map removes the entry)
        assert!(result[1].is_none());
        assert!(result[2].is_none());

        Ok(())
    }
}