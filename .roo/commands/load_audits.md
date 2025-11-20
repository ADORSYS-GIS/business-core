# LoadAudits Trait Implementation Generator

## Purpose
Implement the `LoadAudits` trait for an auditable entity, including database index optimization and comprehensive test coverage.

## Parameters
- **Entity Model File Path**: Path to the entity model file (e.g., `business-core-db/src/models/person/location.rs`)

## Instructions

### 1. Verify Entity is Auditable
- Read the entity model file provided
- Confirm the entity implements the `Auditable` trait
- Extract entity information:
  - Entity name (e.g., `Location` from `LocationModel`)
  - Table name (typically lowercase of entity name, e.g., `location`)
  - Repository path pattern (e.g., `business-core-postgres/src/repository/person/location_repository/`)

### 2. Update Database Migration Index
- Locate the migration file for the entity (search in `business-core-postgres/migrations/` for files containing the entity's audit table)
- Find the index on the audit table that looks like: `idx_<table>_audit_audit_log_id`
- Replace it with an index on the `id` column:
  ```sql
  -- Index on id for efficient audit queries by entity ID.
  -- Note: The audit table intentionally lacks a foreign key to the main table
  -- with `ON DELETE CASCADE`. This ensures that audit history is preserved
  -- even if the main entity record is deleted.
  CREATE INDEX IF NOT EXISTS idx_<table>_audit_id
      ON <table>_audit(id);
  ```
- This index enables efficient queries to load all audit records for a specific entity

### 3. Implement LoadAudits Trait
Create a new file `load_audits.rs` in the entity's repository directory with:

```rust
use async_trait::async_trait;
use business_core_db::models::<module_path>::<EntityModel>;
use business_core_db::repository::load_audits::LoadAudits;
use business_core_db::repository::pagination::{Page, PageRequest};
use crate::utils::TryFromRow;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::<EntityRepository>Impl;

impl <EntityRepository>Impl {
    pub(super) async fn load_audits_impl(
        repo: &<EntityRepository>Impl,
        id: Uuid,
        page: PageRequest,
    ) -> Result<Page<<EntityModel>>, Box<dyn Error + Send + Sync>> {
        // First, get the total count of audit records for this entity
        let count_query = r#"SELECT COUNT(*) as count FROM <table>_audit WHERE id = $1"#;
        let total: i64 = {
            let mut tx = repo.executor.tx.lock().await;
            if let Some(transaction) = tx.as_mut() {
                sqlx::query_scalar(count_query)
                    .bind(id)
                    .fetch_one(&mut **transaction)
                    .await?
            } else {
                return Err("Transaction has been consumed".into());
            }
        };

        // Then fetch the paginated audit records, ordered by audit_log_id (most recent first)
        let query = r#"
            SELECT * FROM <table>_audit 
            WHERE id = $1 
            ORDER BY audit_log_id DESC
            LIMIT $2 OFFSET $3
        "#;
        
        let rows = {
            let mut tx = repo.executor.tx.lock().await;
            if let Some(transaction) = tx.as_mut() {
                sqlx::query(query)
                    .bind(id)
                    .bind(page.limit as i64)
                    .bind(page.offset as i64)
                    .fetch_all(&mut **transaction)
                    .await?
            } else {
                return Err("Transaction has been consumed".into());
            }
        };

        let mut items = Vec::with_capacity(rows.len());
        for row in rows {
            let item = <EntityModel>::try_from_row(&row)?;
            items.push(item);
        }

        Ok(Page::new(items, total as usize, page.limit, page.offset))
    }
}

#[async_trait]
impl LoadAudits<Postgres, <EntityModel>> for <EntityRepository>Impl {
    async fn load_audits(&self, id: Uuid, page: PageRequest) -> Result<Page<<EntityModel>>, Box<dyn Error + Send + Sync>> {
        Self::load_audits_impl(self, id, page).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::load_audits::LoadAudits;
    use business_core_db::repository::pagination::PageRequest;
    use business_core_db::repository::update_batch::UpdateBatch;
    use crate::repository::<module_path>::test_utils::{create_test_audit_log, /* add entity-specific test utils */};

    #[tokio::test]
    async fn test_load_audits() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let <entity>_repo = &ctx.<module>_repos().<entity>_repository;

        // Create necessary dependencies (foreign keys, etc.)
        // ...

        // Create initial entity
        let <entity> = create_test_<entity>(/* params */);
        let <entity>_id = <entity>.id;
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        let mut saved = <entity>_repo.create_batch(vec![<entity>.clone()], Some(audit_log.id)).await?;

        // Update the entity multiple times to create audit history
        // IMPORTANT: Must capture the returned updated entity to get the new hash and audit_log_id
        // This prevents "Concurrent update detected" errors on subsequent updates
        //
        // CRITICAL: You MUST modify at least one field on each update iteration.
        // Objects that haven't changed do not produce new audit records.
        // Simply cloning without modification will NOT create a new audit record.
        for i in 1..=3 {
            let audit_log = create_test_audit_log();
            audit_log_repo.create(&audit_log).await?;
            
            let mut updated = saved[0].clone();
            // REQUIRED: Modify a field to create a new version - choose an appropriate field for your entity
            // Examples:
            // - For strings: updated.field_name = HeaplessString::try_from(format!("Value {i}").as_str()).unwrap();
            // - For booleans: updated.field_name = !updated.field_name;
            // - For integers: updated.field_name = i + 1;
            // - For decimals: updated.field_name = Decimal::from(i);
            saved = <entity>_repo.update_batch(vec![updated], Some(audit_log.id)).await?;
        }

        // Load first page of audit records
        let page = <entity>_repo.load_audits(<entity>_id, PageRequest::new(2, 0)).await?;

        // Should have 4 total audit records (1 create + 3 updates)
        assert_eq!(page.total, 4);
        assert_eq!(page.items.len(), 2); // First page with limit of 2
        assert_eq!(page.page_number(), 1);
        assert_eq!(page.total_pages(), 2);
        assert!(page.has_more());

        // Load second page
        let page2 = <entity>_repo.load_audits(<entity>_id, PageRequest::new(2, 2)).await?;
        assert_eq!(page2.total, 4);
        assert_eq!(page2.items.len(), 2); // Second page with remaining 2 records
        assert_eq!(page2.page_number(), 2);
        assert!(!page2.has_more());

        Ok(())
    }

    #[tokio::test]
    async fn test_load_audits_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let <entity>_repo = &ctx.<module>_repos().<entity>_repository;

        // Try to load audits for non-existing entity
        let non_existing_id = uuid::Uuid::new_v4();
        let page = <entity>_repo.load_audits(non_existing_id, PageRequest::new(20, 0)).await?;

        assert_eq!(page.total, 0);
        assert_eq!(page.items.len(), 0);
        assert_eq!(page.page_number(), 1);
        assert!(!page.has_more());

        Ok(())
    }
}
```

### 4. Update Module Declaration
Add the `load_audits` module to the entity's repository `mod.rs`:

```rust
pub mod load_audits;
```

### 5. Verify Compilation
Run `cargo check --package business-core-postgres` to ensure everything compiles correctly.

## Example Usage
```
Implement LoadAudits trait for business-core-db/src/models/person/location.rs
```

This will:
1. Find and update the migration file `business-core-postgres/migrations/005_initial_schema_person_location.sql`
2. Create `business-core-postgres/src/repository/person/location_repository/load_audits.rs`
3. Update `business-core-postgres/src/repository/person/location_repository/mod.rs`
4. Verify the implementation compiles

## Key Points
- The database index on `id` (not `audit_log_id`) enables efficient loading of all audit records for a specific entity
- Audit records are returned in descending order by `audit_log_id` (most recent first)
- Pagination support allows handling large audit histories efficiently
- Comprehensive tests verify both normal operation and edge cases
- This trait is only applicable to entities implementing the `Auditable` trait