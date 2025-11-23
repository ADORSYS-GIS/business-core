# Entity Template Skill: Auditable Entities (Without Index)

## Overview

This skill generates complete database access modules for entities that require **audit trail functionality** but do not need application-layer indexing. Unlike the indexed templates, these entities are accessed primarily by ID and do not benefit from hash-based or secondary index lookups.

## Purpose

This skill builds upon database access patterns by adding:
- ✅ **Audit table** with complete entity state snapshots
- ✅ **Hash-based verification** using xxHash64 for audit integrity
- ✅ **Audit log integration** with audit_log_id references
- ✅ **Serialization-based hashing** using CBOR encoding
- ✅ **Transaction-level entity tracking** via the `audit_link` table
- ❌ **No index table** - entities are accessed by ID only
- ❌ **No cache integration** - no in-memory caching layer

## When to Use This Template

Use this template for entities that:
- Need complete audit history tracking
- Are primarily accessed by their ID (UUID)
- Don't require secondary index lookups (no hash-based searches)
- Don't need in-memory caching for performance
- Are less frequently accessed or have simpler access patterns

**Examples**: Configuration records, system settings, archived data, historical snapshots.

## Prerequisites

**IMPORTANT**: If your entity needs secondary index lookups (by hash, foreign key, etc.), you should use [Entity with Index and Audit](entity_with_index_and_audit.md) instead. This template is for ID-only access patterns.

---

## Template Reference

This pattern is based on auditable entities without indexing:
- **Auditable Trait**: `business-core/business-core-db/src/models/auditable.rs`
- **Audit Models**: `business-core/business-core-db/src/models/audit/`
- **Reference Implementation**: Location entity (but without the index table/cache)

---

## Artifacts for Auditable Entities (No Index)

### 1. Main Model Structure

The main entity model includes audit fields only:

```rust
use heapless::String as HeaplessString;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use crate::models::auditable::Auditable;
use crate::models::identifiable::Identifiable;

/// # Documentation
/// - {Entity description}
///
/// ## Enum Types
/// If your entity includes an enum, define it with `sqlx::Type`.
///
/// ```rust
/// /// Database model for {enum_name} enum
/// #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
/// #[sqlx(type_name = "{enum_name}", rename_all = "PascalCase")]
/// pub enum {EnumName} {
///     Variant1,
///     Variant2,
/// }
/// ```
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct {Entity}Model {
    pub id: Uuid,
    
    // ... entity fields
    
    /// Hash from the previous audit record for chain verification (0 for initial create)
    pub antecedent_hash: i64,
    
    /// Reference to the previous audit log entry (Uuid::nil() for initial create)
    pub antecedent_audit_log_id: Uuid,
    
    /// Hash of the entity with hash field set to 0
    /// - 0: for new entities not yet created or not yet hashed
    /// - Non-zero: computed hash providing tamper detection
    pub hash: i64,
    
    /// Reference to the current audit log entry for this entity
    /// - None: for new entities not yet created
    /// - Some(uuid): updated on every create/update operation to reference the latest audit log
    /// 
    /// This field, together with `id`, forms the composite primary key in the audit table
    pub audit_log_id: Option<Uuid>,
}

impl Identifiable for {Entity}Model {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Auditable for {Entity}Model {
    fn get_audit_log_id(&self) -> Option<Uuid> {
        self.audit_log_id
    }
}
```

**Key Differences from Indexed Entities**:
- ❌ No `{Entity}IdxModel` struct
- ❌ No `IndexAware` trait implementation
- ❌ No `Indexable` trait implementation
- ❌ No cache type definition
- ✅ Only `Identifiable` and `Auditable` traits

---

## Repository Implementation (Without Index/Cache)

### Repository Structure

```
{entity}_repository/
├── mod.rs
├── repo_impl.rs
├── create_batch.rs
├── load_batch.rs
├── load_audits.rs
├── update_batch.rs
├── delete_batch.rs
└── exist_by_ids.rs
```

**Note**: No custom finder methods since there's no index table. However, `load_audits.rs` is required for all auditable entities.

### repo_impl.rs Pattern

```rust
use business_core_db::models::{module}::{entity}::{Entity}Model;
use crate::utils::{get_heapless_string, get_optional_heapless_string, TryFromRow};
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use async_trait::async_trait;

pub struct {Entity}RepositoryImpl {
    pub executor: Executor,
}

impl {Entity}RepositoryImpl {
    pub fn new(executor: Executor) -> Self {
        Self { executor }
    }
}

impl TryFromRow<PgRow> for {Entity}Model {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok({Entity}Model {
            id: row.get("id"),
            // ... field mappings with appropriate helper functions
            antecedent_hash: row.get("antecedent_hash"),
            antecedent_audit_log_id: row.get("antecedent_audit_log_id"),
            hash: row.get("hash"),
            audit_log_id: row.try_get("audit_log_id").ok(),
        })
    }
}

#[async_trait]
impl TransactionAware for {Entity}RepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        Ok(())
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        Ok(())
    }
}
```

**Key Differences from Indexed Repositories**:
- ❌ No cache field in the repository struct
- ❌ No `load_all_{entity}_idx` method
- ❌ No `{Entity}IdxModel` TryFromRow implementation
- ✅ Simpler `TransactionAware` implementation (no cache to commit/rollback)

---

## Repository Method Patterns

### CREATE Path Pattern

```rust
use business_core_db::utils::hash_as_i64;
use business_core_db::models::audit::audit_link::AuditLinkModel;
use business_core_db::models::audit::entity_type::EntityType;

async fn create_batch_impl(
    repo: &{Entity}RepositoryImpl,
    items: Vec<{Entity}Model>,
    audit_log_id: Uuid,
) -> Result<Vec<{Entity}Model>, Box<dyn Error + Send + Sync>> {
    if items.is_empty() {
        return Ok(Vec::new());
    }

    let mut saved_items = Vec::new();
    let mut tx = repo.executor.tx.lock().await;
    let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
    
    for mut entity in items {
        // 1. Create a copy of entity for hashing
        let mut entity_for_hashing = entity.clone();
        entity_for_hashing.hash = 0;  // Must be 0 before hashing
        entity_for_hashing.audit_log_id = Some(audit_log_id); // Set ID before hashing
        
        // 2. Compute hash
        let computed_hash = hash_as_i64(&entity_for_hashing)?;
        
        // 3. Update original entity with computed hash and new audit_log_id
        entity.hash = computed_hash;
        entity.audit_log_id = Some(audit_log_id);
        
        // 4. Build audit insert query - inserts the entity
        let audit_insert_query = sqlx::query(
            r#"
            INSERT INTO {table_name}_audit
            (id, field1, field2, ..., hash, audit_log_id)
            VALUES ($1, $2, $3, ..., $N, $N+1)
            "#,
        )
        .bind(entity.id)
        .bind(entity.field1.as_str())
        .bind(entity.field2.as_str())
        // ... bind remaining entity fields
        .bind(entity.hash)
        .bind(entity.audit_log_id);
        // Antecedent fields have default value for a new entity. So no need to bind.
        
        // 5. Build entity insert query
        let entity_insert_query = sqlx::query(
            r#"
            INSERT INTO {table_name}
            (id, field1, field2, ..., hash, audit_log_id)
            VALUES ($1, $2, $3, ..., $N, $N+1)
            "#,
        )
        .bind(entity.id)
        .bind(entity.field1.as_str())
        // ... bind all fields
        .bind(entity.hash)
        .bind(entity.audit_log_id);
        // Antecedent fields have default value for a new entity. So no need to bind.
        
        // 6. Create audit link to track the entity modification in the transaction
        let audit_link = AuditLinkModel {
            audit_log_id,
            entity_id: entity.id,
            entity_type: EntityType::{Entity},
        };
        let audit_link_query = sqlx::query(
            r#"
            INSERT INTO audit_link (audit_log_id, entity_id, entity_type)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(audit_link.audit_log_id)
        .bind(audit_link.entity_id)
        .bind(audit_link.entity_type);
        
        // 7. Execute in transaction (audit first!)
        audit_insert_query.execute(&mut **transaction).await?;
        entity_insert_query.execute(&mut **transaction).await?;
        audit_link_query.execute(&mut **transaction).await?;
        
        saved_items.push(entity);
    }

    Ok(saved_items)
}
```

**Key Differences from Indexed Entities**:
- ❌ No index table insert
- ❌ No cache update
- ✅ Simpler flow: audit → entity → audit_link

### UPDATE Path Pattern

```rust
async fn update_batch_impl(
    repo: &{Entity}RepositoryImpl,
    items: Vec<{Entity}Model>,
    audit_log_id: Uuid,
) -> Result<Vec<{Entity}Model>, Box<dyn Error + Send + Sync>> {
    if items.is_empty() {
        return Ok(Vec::new());
    }

    let mut updated_items = Vec::new();
    let mut tx = repo.executor.tx.lock().await;
    let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
    
    for mut entity in items {
        // 1. Save current hash and audit_log_id for antecedent tracking
        let previous_hash = entity.hash;
        let previous_audit_log_id = entity.audit_log_id
            .ok_or("Entity must have audit_log_id for update")?;
        
        // 2. Check if entity has actually changed by recomputing hash
        let mut entity_for_hashing = entity.clone();
        entity_for_hashing.hash = 0;
        
        let computed_hash = hash_as_i64(&entity_for_hashing)?;
        
        // 3. Only proceed with update if entity has changed
        if computed_hash == previous_hash {
            updated_items.push(entity);
            continue;
        }
        
        // 4. Entity has changed, update with new hash and audit_log_id
        entity.antecedent_hash = previous_hash;
        entity.antecedent_audit_log_id = previous_audit_log_id;
        entity.audit_log_id = Some(audit_log_id);
        entity.hash = 0;
        
        let new_computed_hash = hash_as_i64(&entity)?;
        entity.hash = new_computed_hash;
        
        // 5. Build audit insert query
        let audit_insert_query = sqlx::query(
            r#"
            INSERT INTO {table_name}_audit
            (id, field1, field2, ..., hash, audit_log_id, antecedent_hash, antecedent_audit_log_id)
            VALUES ($1, $2, $3, ..., $N, $N+1, $N+2, $N+3)
            "#,
        )
        .bind(entity.id)
        .bind(entity.field1.as_str())
        // ... bind remaining entity fields
        .bind(entity.hash)
        .bind(entity.audit_log_id)
        .bind(entity.antecedent_hash)
        .bind(entity.antecedent_audit_log_id);
        
        // 6. Build entity update query
        let entity_update_query = sqlx::query(
            r#"
            UPDATE {table_name} SET
                field1 = $2,
                field2 = $3,
                // ... all fields
                hash = $N,
                audit_log_id = $N+1,
                antecedent_hash = $N+2,
                antecedent_audit_log_id = $N+3
            WHERE id = $1
              AND hash = $N+4
              AND audit_log_id = $N+5
            "#,
        )
        .bind(entity.id)
        .bind(entity.field1.as_str())
        // ... bind all fields
        .bind(entity.hash)
        .bind(entity.audit_log_id)
        .bind(entity.antecedent_hash)
        .bind(entity.antecedent_audit_log_id)
        .bind(previous_hash)
        .bind(previous_audit_log_id);
        
        // 7. Create audit link
        let audit_link = AuditLinkModel {
            audit_log_id,
            entity_id: entity.id,
            entity_type: EntityType::{Entity},
        };
        let audit_link_query = sqlx::query(
            r#"
            INSERT INTO audit_link (audit_log_id, entity_id, entity_type)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(audit_link.audit_log_id)
        .bind(audit_link.entity_id)
        .bind(audit_link.entity_type);
        
        // 8. Execute in transaction (audit first!)
        audit_insert_query.execute(&mut **transaction).await?;
        entity_update_query.execute(&mut **transaction).await?;
        audit_link_query.execute(&mut **transaction).await?;
        
        updated_items.push(entity);
    }

    Ok(updated_items)
}
```

**Key Differences from Indexed Entities**:
- ❌ No index table update
- ❌ No cache update
- ✅ Simpler flow: audit → entity → audit_link

### DELETE Path Pattern

```rust
async fn delete_batch_impl(
    repo: &{Entity}RepositoryImpl,
    ids: &[Uuid],
    audit_log_id: Uuid,
) -> Result<usize, Box<dyn Error + Send + Sync>> {
    if ids.is_empty() {
        return Ok(0);
    }

    // 1. Load the full entities to be deleted
    let entities_to_delete = repo.load_batch(ids).await?;
    
    let mut deleted_count = 0;
    let mut tx = repo.executor.tx.lock().await;
    let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
    
    for entity_opt in entities_to_delete {
        let entity = match entity_opt {
            Some(e) => e,
            None => continue,
        };
        
        // 2. Create a final audit record before deletion
        let mut final_audit_entity = entity.clone();
        final_audit_entity.antecedent_hash = entity.hash;
        final_audit_entity.antecedent_audit_log_id = entity.audit_log_id
            .ok_or("Entity must have audit_log_id for deletion")?;
        final_audit_entity.audit_log_id = Some(audit_log_id);
        final_audit_entity.hash = 0;
        
        let final_hash = hash_as_i64(&final_audit_entity)?;
        final_audit_entity.hash = final_hash;
        
        // 3. Build the audit insert query
        let audit_insert_query = sqlx::query(
            r#"
            INSERT INTO {table_name}_audit
            (id, field1, field2, ..., hash, audit_log_id, antecedent_hash, antecedent_audit_log_id)
            VALUES ($1, $2, $3, ..., $N, $N+1, $N+2, $N+3)
            "#,
        )
        .bind(final_audit_entity.id)
        // ... bind all fields from final_audit_entity
        .bind(final_audit_entity.hash)
        .bind(final_audit_entity.audit_log_id)
        .bind(final_audit_entity.antecedent_hash)
        .bind(final_audit_entity.antecedent_audit_log_id);
        
        // 4. Build the entity delete query
        let entity_delete_query = sqlx::query(
            r#"
            DELETE FROM {table_name} WHERE id = $1
            "#,
        )
        .bind(entity.id);
        
        // 5. Create audit link
        let audit_link = AuditLinkModel {
            audit_log_id,
            entity_id: entity.id,
            entity_type: EntityType::{Entity},
        };
        let audit_link_query = sqlx::query(
            r#"
            INSERT INTO audit_link (audit_log_id, entity_id, entity_type)
            VALUES ($1, $2, $3)
            "#,
        )
        .bind(audit_link.audit_log_id)
        .bind(audit_link.entity_id)
        .bind(audit_link.entity_type);
        
        // 6. Execute in transaction (audit first!)
        audit_insert_query.execute(&mut **transaction).await?;
        entity_delete_query.execute(&mut **transaction).await?;
        audit_link_query.execute(&mut **transaction).await?;
        
        deleted_count += 1;
    }

    Ok(deleted_count)
}
```

**Key Differences from Indexed Entities**:
- ❌ No index table delete (no `ON DELETE CASCADE`)
- ❌ No cache removal
- ✅ Simpler flow: audit → entity → audit_link

### LOAD Path Pattern

```rust
async fn load_batch_impl(
    repo: &{Entity}RepositoryImpl,
    ids: &[Uuid],
) -> Result<Vec<Option<{Entity}Model>>, Box<dyn Error + Send + Sync>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    
    let query = r#"SELECT * FROM {table_name} WHERE id = ANY($1)"#;
    let rows = {
        let mut tx = repo.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        sqlx::query(query).bind(ids).fetch_all(&mut **transaction).await?
    };
    
    let mut item_map = std::collections::HashMap::new();
    for row in rows {
        let item = {Entity}Model::try_from_row(&row)?;
        item_map.insert(item.id, item);
    }
    
    let mut result = Vec::with_capacity(ids.len());
    for id in ids {
        result.push(item_map.remove(id));
    }
    Ok(result)
}
```

**Key Differences from Indexed Entities**:
- ✅ Same pattern - direct database query by IDs
- ❌ No cache lookup

### EXIST Path Pattern

```rust
async fn exist_by_ids_impl(
    repo: &{Entity}RepositoryImpl,
    ids: &[Uuid],
) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
    if ids.is_empty() {
        return Ok(Vec::new());
    }
    
    let query = r#"SELECT id FROM {table_name} WHERE id = ANY($1)"#;
    let rows = {
        let mut tx = repo.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        sqlx::query(query).bind(ids).fetch_all(&mut **transaction).await?
    };
    
    let existing_ids: std::collections::HashSet<Uuid> = rows
        .iter()
        .map(|row| row.get("id"))
        .collect();
    
    let mut result = Vec::new();
    for &id in ids {
        result.push((id, existing_ids.contains(&id)));
    }
    Ok(result)
}
```

**Key Differences from Indexed Entities**:
- ❌ No cache lookup - queries database directly
- ✅ Returns same result structure

---

## Database Schema

A complete migration script for an auditable entity (without index) includes the main table and an audit table.

**IMPORTANT**: Each new auditable entity must extend the `audit_entity_type` ENUM in the database.
This is a manual step that must be included in the migration script.

Example:
```sql
-- Add new entity type to audit_entity_type enum
ALTER TYPE audit_entity_type ADD VALUE 'YourNewEntityType';
```

```sql
-- Migration: Initial {Entity} Schema with Audit Support (No Index)
-- Description: Creates {entity}-related tables with audit trail.
 
-- Enum Types (if any)
CREATE TYPE IF NOT EXISTS {enum_name} AS ENUM ('Variant1', 'Variant2');

-- Main {Entity} Table
-- Stores the current state of the entity.
CREATE TABLE IF NOT EXISTS {table_name} (
    id UUID PRIMARY KEY,
    field1 VARCHAR(...),
    field2 VARCHAR(...),
    -- ... other fields
    hash BIGINT NOT NULL DEFAULT 0,
    audit_log_id UUID REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
);

-- {Entity} Audit Table
-- Stores a complete, immutable snapshot of the entity at each change.
CREATE TABLE IF NOT EXISTS {table_name}_audit (
    -- All entity fields are duplicated here for a complete snapshot.
    id UUID NOT NULL,
    field1 VARCHAR(...),
    field2 VARCHAR(...),
    -- ... all other entity fields
    
    -- Audit-specific fields
    hash BIGINT NOT NULL,
    audit_log_id UUID NOT NULL REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    
    -- Composite primary key ensures one audit entry per entity version.
    PRIMARY KEY (id, audit_log_id)
);

-- Index on id for efficient audit queries by entity ID.
-- Note: The audit table intentionally lacks a foreign key to the main table
-- with `ON DELETE CASCADE`. This ensures that audit history is preserved
-- even if the main entity record is deleted.
CREATE INDEX IF NOT EXISTS idx_{table_name}_audit_id
    ON {table_name}_audit(id);
```

**Key Differences from Indexed Entities**:
- ❌ No `{table_name}_idx` table
- ❌ No trigger for cache notification
- ✅ Simpler schema: main table + audit table only

---

## Repository Factory Pattern (Without Cache)

### Factory Implementation

```rust
use std::sync::Arc;
use postgres_unit_of_work::UnitOfWorkSession;
use super::{Entity}RepositoryImpl;

/// Factory for creating {module} module repositories (without caching)
pub struct {Module}RepoFactory {
    // No cache fields needed
}

impl {Module}RepoFactory {
    /// Create a new {Module}RepoFactory singleton
    pub fn new() -> Arc<Self> {
        Arc::new(Self {})
    }

    /// Build a {Entity}Repository with the given executor
    pub fn build_{entity}_repo(&self, session: &impl UnitOfWorkSession) -> Arc<{Entity}RepositoryImpl> {
        let repo = Arc::new({Entity}RepositoryImpl::new(
            session.executor().clone(),
        ));
        session.register_transaction_aware(repo.clone());
        repo
    }
}
```

**Key Differences from Indexed Factories**:
- ❌ No cache initialization
- ❌ No cache notification listener registration
- ❌ No cache fields in the factory
- ✅ Simpler factory - just creates repositories with executors

---

## Testing Patterns for Auditable Entities (No Index)

### Standard Test Cases

```rust
#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::models::audit::audit_log::AuditLogModel;
    use uuid::Uuid;

    fn create_test_audit_log() -> AuditLogModel {
        AuditLogModel {
            id: Uuid::new_v4(),
            updated_at: chrono::Utc::now(),
            updated_by_person_id: Uuid::new_v4(),
        }
    }

    #[tokio::test]
    async fn test_create_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let {entity}_repo = &ctx.{module}_repos().{entity}_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let mut entities = Vec::new();
        for i in 0..3 {
            entities.push(create_test_{entity}(/* params */));
        }

        let saved = {entity}_repo.create_batch(entities, audit_log.id).await?;

        assert_eq!(saved.len(), 3);
        for entity in &saved {
            assert!(entity.hash != 0, "Hash should be computed");
            assert_eq!(entity.audit_log_id, Some(audit_log.id));
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_update_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let {entity}_repo = &ctx.{module}_repos().{entity}_repository;

        // Create initial entities
        let create_audit_log = create_test_audit_log();
        audit_log_repo.create(&create_audit_log).await?;
        
        let entities = vec![create_test_{entity}(/* params */)];
        let saved = {entity}_repo.create_batch(entities, create_audit_log.id).await?;

        // Update entities
        let update_audit_log = create_test_audit_log();
        audit_log_repo.create(&update_audit_log).await?;
        
        let mut updated_entities = Vec::new();
        for mut entity in saved {
            entity.field1 = HeaplessString::try_from("Updated Value").unwrap();
            updated_entities.push(entity);
        }

        let updated = {entity}_repo.update_batch(updated_entities, update_audit_log.id).await?;

        assert_eq!(updated.len(), 1);
        assert_eq!(updated[0].field1.as_str(), "Updated Value");
        assert_eq!(updated[0].audit_log_id, Some(update_audit_log.id));

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let {entity}_repo = &ctx.{module}_repos().{entity}_repository;

        // Create entities
        let create_audit_log = create_test_audit_log();
        audit_log_repo.create(&create_audit_log).await?;
        
        let entities = vec![create_test_{entity}(/* params */)];
        let saved = {entity}_repo.create_batch(entities, create_audit_log.id).await?;

        let ids: Vec<Uuid> = saved.iter().map(|e| e.id).collect();

        // Delete entities
        let delete_audit_log = create_test_audit_log();
        audit_log_repo.create(&delete_audit_log).await?;
        
        let deleted_count = {entity}_repo.delete_batch(&ids, delete_audit_log.id).await?;

        assert_eq!(deleted_count, 1);

        // Verify deletion
        let loaded = {entity}_repo.load_batch(&ids).await?;
        assert!(loaded[0].is_none());

        Ok(())
    }

    #[tokio::test]
    async fn test_load_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let {entity}_repo = &ctx.{module}_repos().{entity}_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let entity = create_test_{entity}(/* params */);
        let entity_id = entity.id;
        
        let saved = {entity}_repo.create_batch(vec![entity], audit_log.id).await?;

        let loaded = {entity}_repo.load_batch(&[entity_id]).await?;

        assert_eq!(loaded.len(), 1);
        assert!(loaded[0].is_some());
        assert_eq!(loaded[0].as_ref().unwrap().id, entity_id);

        Ok(())
    }

    #[tokio::test]
    async fn test_exist_by_ids() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let {entity}_repo = &ctx.{module}_repos().{entity}_repository;

        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let entity = create_test_{entity}(/* params */);
        let entity_id = entity.id;
        let non_existent_id = Uuid::new_v4();
        
        {entity}_repo.create_batch(vec![entity], audit_log.id).await?;

        let results = {entity}_repo.exist_by_ids(&[entity_id, non_existent_id]).await?;

        assert_eq!(results.len(), 2);
        assert_eq!(results[0], (entity_id, true));
        assert_eq!(results[1], (non_existent_id, false));

        Ok(())
    }

    #[tokio::test]
    async fn test_load_audits() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let {entity}_repo = &ctx.{module}_repos().{entity}_repository;

        // Create necessary dependencies (foreign keys, etc.)
        // ...

        // Create initial entity
        let {entity} = create_test_{entity}(/* params */);
        let {entity}_id = {entity}.id;
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        let mut saved = {entity}_repo.create_batch(vec![{entity}.clone()], Some(audit_log.id)).await?;

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
            saved = {entity}_repo.update_batch(vec![updated], Some(audit_log.id)).await?;
        }

        // Load first page of audit records
        let page = {entity}_repo.load_audits({entity}_id, PageRequest::new(2, 0)).await?;

        // Should have 4 total audit records (1 create + 3 updates)
        assert_eq!(page.total, 4);
        assert_eq!(page.items.len(), 2); // First page with limit of 2
        assert_eq!(page.page_number(), 1);
        assert_eq!(page.total_pages(), 2);
        assert!(page.has_more());

        // Load second page
        let page2 = {entity}_repo.load_audits({entity}_id, PageRequest::new(2, 2)).await?;
        assert_eq!(page2.total, 4);
        assert_eq!(page2.items.len(), 2); // Second page with remaining 2 records
        assert_eq!(page2.page_number(), 2);
        assert!(!page2.has_more());

        Ok(())
    }

    #[tokio::test]
    async fn test_load_audits_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let {entity}_repo = &ctx.{module}_repos().{entity}_repository;

        // Try to load audits for non-existing entity
        let non_existing_id = uuid::Uuid::new_v4();
        let page = {entity}_repo.load_audits(non_existing_id, PageRequest::new(20, 0)).await?;

        assert_eq!(page.total, 0);
        assert_eq!(page.items.len(), 0);
        assert_eq!(page.page_number(), 1);
        assert!(!page.has_more());

        Ok(())
    }
}
```

**Key Differences from Indexed Entity Tests**:
- ❌ No cache verification tests
- ❌ No cache notification tests
- ❌ No finder method tests (no secondary indexes)
- ✅ Simpler tests focused on CRUD operations and audit trail
- ✅ LoadAudits tests verify pagination and audit history

---

## Validation Checklist

After generating code, verify:

- [ ] Main entity model has `hash` and `audit_log_id` fields
- [ ] Auditable trait is implemented
- [ ] Identifiable trait is implemented
- [ ] ❌ No IndexAware trait (not needed)
- [ ] ❌ No Indexable trait (not needed)
- [ ] ❌ No IdxModel struct (not needed)
- [ ] ❌ No cache in repository (not needed)
- [ ] Hash computation uses the entity (with `hash=0`) pattern
- [ ] For UPDATE: entity's current hash and audit_log_id are used as antecedent values
- [ ] For CREATE: entity's hash = 0 and audit_log_id = None initially
- [ ] Audit record is inserted **before** entity modification
- [ ] Batch operations (create/update/delete) include audit_log_id parameter
- [ ] Audit table has composite primary key `(id, audit_log_id)`
- [ ] ❌ No index table in migration (not needed)
- [ ] ❌ No trigger for cache notification (not needed)
- [ ] Audit table includes all entity fields
- [ ] Audit table does NOT have `ON DELETE CASCADE` (audit survives deletion)
- [ ] Audit table has index on `id` column (not `audit_log_id`) for efficient audit queries
- [ ] All audit tests pass (creation, update, deletion, hash integrity)
- [ ] LoadAudits trait is implemented with pagination support
- [ ] LoadAudits tests verify correct pagination and audit history retrieval
- [ ] TransactionAware implementation (simple, no cache)
- [ ] Migration includes audit table with correct schema
- [ ] Cleanup script removes audit table

---

## Best Practices

### ✅ DO:

- Always create audit log entry before calling create/update/delete operations
- Insert audit record before modifying the main entity
- Set `entity.hash` to zero before computing the hash
- Use CBOR serialization for hash computation (via `hash_as_i64`)
- Store antecedent hash and audit_log_id for chain verification
- Test audit trail integrity and hash verification
- Use direct database queries for load and exist operations
- Implement TransactionAware (simple version without cache)

### ❌ DON'T:

- Use this template if you need secondary index lookups (use indexed template instead)
- Create index tables or cache infrastructure
- Modify main entity before creating audit record
- Skip hash computation or forget to set hash to zero before hashing
- Forget to update audit_log_id in the entity
- Use version numbers (audit records are keyed by id and audit_log_id)
- Add finder methods (no index to query)

---

## When to Upgrade to Indexed Template

Consider migrating to [Entity with Index and Audit](entity_with_index_and_audit.md) if:

- You need to query entities by fields other than ID
- You need hash-based lookups (e.g., by code, external ID)
- You need foreign key relationship queries
- Performance requires in-memory caching
- Access patterns show frequent non-ID lookups

---

## Dependencies

Add to `Cargo.toml`:

```toml
[dependencies]
# For hash computation
twox-hash = "1.6"

# For CBOR serialization
ciborium = "0.2"
```

---

## References

- **Auditable Template with Index**: [Entity with Index and Audit](entity_with_index_and_audit.md)
- **Non-Auditable Template**: [Entity with Index](entity_with_index.md)
- **Audit Traits**: `business-core/business-core-db/src/models/auditable.rs`
- **Audit Models**: `business-core/business-core-db/src/models/audit/`
- **Hash Library**: [twox-hash](https://docs.rs/twox-hash/)
- **CBOR Library**: [ciborium](https://docs.rs/ciborium/)

---

## License

Same as business-core project