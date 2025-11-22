# Entity Template Skill: Indexable and Auditable Entities

## Overview

This skill extends the [Entity with Index](entity_with_index.md) template to add **audit trail functionality**. It generates complete database access modules for entities that require both application-layer indexing and comprehensive audit logging.

## Purpose

This skill builds upon the base indexable entity template by adding:
- ✅ **Audit table** with complete entity state snapshots stored as tuples
- ✅ **Hash-based verification** using xxHash64 for audit integrity
- ✅ **Audit log integration** with audit_log_id references
- ✅ **Serialization-based hashing** using CBOR encoding
- ✅ **Transaction-level entity tracking** via the `audit_link` table

## Prerequisites

**You must read and understand** [Entity with Index](entity_with_index.md) first. This document only covers the **additional** patterns for audit functionality. All base patterns from the indexable entity template still apply.

**CRITICAL PATTERN - Finder Methods**: The finder method pattern for secondary index fields described in the base template applies to auditable entities as well:

**Rule**: For each secondary index field (field name not equal to `id`) in your `{Entity}IdxModel`, you MUST create a corresponding finder method that returns `Vec<{Entity}IdxModel>`.

**Example**: If your `EntityReferenceIdxModel` has:
- `id: Uuid` → No finder needed (primary key)
- `person_id: Uuid` → MUST create `find_by_person_id.rs`
- `reference_external_id_hash: i64` → MUST create `find_by_reference_external_id_hash.rs`

See the [Finder Methods section](entity_with_index.md#finder-methods-for-secondary-index-fields) in the base template for detailed implementation patterns.

---

## Template Reference

The audit pattern is based on the Named entity implementation:
- **Model**: `business-core/business-core-db/src/models/description/named.rs`
- **Repository**: `business-core/business-core-postgres/src/repository/description/named_repository/`
- **Audit Hash Logic**: Integrated directly within the `{Entity}Model`'s audit fields.
- **Auditable Trait**: `business-core/business-core-db/src/models/auditable.rs`

---

## Additional Artifacts for Auditable Entities

The main model {Entity}Model is used to maintain both tables: {entity} and {entity}_audit. The main model receives 4 special fields: `hash`, `audit_log_id`, `antecedent_hash` and `antecedent_audit_log_id`.



### 1. Main Model Modifications

The main entity model must include:

```rust
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
```

### 3. Auditable Trait Implementation

```rust
use business_core_db::models::auditable::Auditable;
use business_core_db::models::identifiable::Identifiable;

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

---

## Repository Method Signature Changes

For auditable entities, the repository methods that modify data require an `audit_log_id` parameter:

### Standard Batch Operations

```rust
async fn create_batch(&self, items: Vec<{Entity}Model>, audit_log_id: Option<Uuid>)

async fn update_batch(&self, items: Vec<{Entity}Model>, audit_log_id: Option<Uuid>)

async fn delete_batch(&self, ids: &[Uuid], audit_log_id: Option<Uuid>)
    -> Result<usize, Box<dyn Error + Send + Sync>>;
```

**Note**: The `audit_log_id` parameter is used directly (no underscore prefix) as it's always utilized in auditable entities.

### CREATE Path Pattern

```rust
// CREATE path - insert entity with audit trail

// For auditable entities, compute the hash of the entity
use business_core_db::utils::hash_as_i64;
// 1. Create a copy of entity for hashing
let mut entity_for_hashing = entity.clone();
entity_for_hashing.hash = 0;  // Must be 0 before hashing
entity_for_hashing.audit_log_id = Some(audit_log_id); // Set ID before hashing

// 2. Compute hash
let computed_hash = hash_as_i64(&entity_for_hashing)?;

// 3. Update original entity with computed hash and new audit_log_id
entity.hash = computed_hash;
entity.audit_log_id = Some(audit_log_id);

// Build audit insert query - inserts the entity
let audit_insert_query = sqlx::query(
    r#"
    INSERT INTO named_audit
    (id, entity_type, name_l1, name_l2, name_l3, name_l4, description_l1, description_l2, description_l3, description_l4, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
    "#,
    )
    .bind(item.id)
    .bind(item.entity_type)
    .bind(item.name_l1.as_str())
    .bind(item.name_l2.as_deref())
    .bind(item.name_l3.as_deref())
    .bind(item.name_l4.as_deref())
    .bind(item.description_l1.as_deref())
    .bind(item.description_l2.as_deref())
    .bind(item.description_l3.as_deref())
    .bind(item.description_l4.as_deref())
    .bind(item.antecedent_hash)
    .bind(item.antecedent_audit_log_id)
    .bind(item.hash)
    .bind(item.audit_log_id)
// Antecedent fields have default value for a new entity. So no need to bind.

// Build entity insert query
let entity_insert_query = sqlx::query(
    r#"
    INSERT INTO named
    (id, entity_type, name_l1, name_l2, name_l3, name_l4, description_l1, description_l2, description_l3, description_l4, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
    "#,
    )
    .bind(item.id)
    .bind(item.entity_type)
    .bind(item.name_l1.as_str())
    .bind(item.name_l2.as_deref())
    .bind(item.name_l3.as_deref())
    .bind(item.name_l4.as_deref())
    .bind(item.description_l1.as_deref())
    .bind(item.description_l2.as_deref())
    .bind(item.description_l3.as_deref())
    .bind(item.description_l4.as_deref())
    .bind(item.antecedent_hash)
    .bind(item.antecedent_audit_log_id)
    .bind(item.hash)
    .bind(item.audit_log_id);
// Antecedent fields have default value for a new entity. So no need to bind.

// Build index insert query (same as base template)
let idx_insert_query = sqlx::query(
    r#"
    INSERT INTO named_idx (id, entity_type)
    VALUES ($1, $2)
    "#,
    )
    .bind(idx.id)
    .bind(idx.entity_type);

// Create audit link to track the entity modification in the transaction
let audit_link = AuditLinkModel {
    audit_log_id,
    entity_id: entity.id,
    entity_type: AuditEntityType::Named,
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

// Execute in transaction (audit first!)
let mut tx = repo.executor.tx.lock().await;
let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
audit_insert_query.execute(&mut **transaction).await?;
entity_insert_query.execute(&mut **transaction).await?;
idx_insert_query.execute(&mut **transaction).await?;
audit_link_query.execute(&mut **transaction).await?;

// Update cache (same as base template)
let new_idx = {Entity}IdxModel {
    id: entity.id,
    // ... index fields
};
repo.{entity}_idx_cache.read().await.add(new_idx);
```

### UPDATE Path Pattern

```rust
// UPDATE path - update entity with audit trail

// 1. Save current hash and audit_log_id for antecedent tracking
let previous_hash = entity.hash;
let previous_audit_log_id = entity.audit_log_id.ok_or("Entity must have audit_log_id for update")?;

// 2. Check if entity has actually changed by recomputing hash
let mut entity_for_hashing = entity.clone();
entity_for_hashing.hash = 0;

// Compute hash of entity_for_hashing
let computed_hash = hash_as_i64(&entity_for_hashing)?;

// 3. Only proceed with update if entity has changed
if computed_hash == previous_hash {
    // No changes detected, return entity as-is
    return Ok(vec![entity]);
}

// The antecedent hash and audit log ID are now part of the entity itself.
entity.antecedent_hash = previous_hash;
entity.antecedent_audit_log_id = previous_audit_log_id;

// 4. Entity has changed, update with new hash and audit_log_id
// The hash used for the change check is not the final hash.
// The entity must be re-hashed with the antecedent fields and new audit_log_id.
entity.audit_log_id = Some(audit_log_id);
entity.hash = 0; // Set to 0 before final hashing

// Compute final hash for storage
let new_computed_hash = hash_as_i64(&entity)?;
entity.hash = new_computed_hash;

// 5. Build audit insert query (includes all entity fields)
let audit_insert_query = sqlx::query(
    r#"
    INSERT INTO named_audit
    (id, entity_type, name_l1, name_l2, name_l3, name_l4, description_l1, description_l2, description_l3, description_l4, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
    "#,
    )
    .bind(item.id)
    .bind(item.entity_type)
    .bind(item.name_l1.as_str())
    .bind(item.name_l2.as_deref())
    .bind(item.name_l3.as_deref())
    .bind(item.name_l4.as_deref())
    .bind(item.description_l1.as_deref())
    .bind(item.description_l2.as_deref())
    .bind(item.description_l3.as_deref())
    .bind(item.description_l4.as_deref())
    .bind(item.antecedent_hash)
    .bind(item.antecedent_audit_log_id)
    .bind(item.hash)
    .bind(item.audit_log_id);

// Build entity update query
let entity_update_query = sqlx::query(
    r#"
    UPDATE named SET
    entity_type = $2,
    name_l1 = $3,
    name_l2 = $4,
    name_l3 = $5,
    name_l4 = $6,
    description_l1 = $7,
    description_l2 = $8,
    description_l3 = $9,
    description_l4 = $10,
    antecedent_hash = $11,
    antecedent_audit_log_id = $12,
    hash = $13,
    audit_log_id = $14
    WHERE id = $1 AND hash = $15 AND audit_log_id = $16
    "#,
    )
    .bind(item.id)
    .bind(item.entity_type)
    .bind(item.name_l1.as_str())
    .bind(item.name_l2.as_deref())
    .bind(item.name_l3.as_deref())
    .bind(item.name_l4.as_deref())
    .bind(item.description_l1.as_deref())
    .bind(item.description_l2.as_deref())
    .bind(item.description_l3.as_deref())
    .bind(item.description_l4.as_deref())
    .bind(item.antecedent_hash)
    .bind(item.antecedent_audit_log_id)
    .bind(item.hash)
    .bind(item.audit_log_id)
    .bind(previous_hash)
    .bind(previous_audit_log_id);

// Create audit link
let audit_link = AuditLinkModel {
    audit_log_id,
    entity_id: entity.id,
    entity_type: AuditEntityType::Named,
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

// Execute in transaction (audit first!)
let mut tx = repo.executor.tx.lock().await;
let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
audit_insert_query.execute(&mut **transaction).await?;
entity_update_query.execute(&mut **transaction).await?;
audit_link_query.execute(&mut **transaction).await?;

// Update cache (remove old, add new)
let new_idx = {Entity}IdxModel {
    id: entity.id,
    // ... index fields from entity
};
let cache = repo.{entity}_idx_cache.read().await;
cache.remove(&entity.id);
cache.add(new_idx);
```

### DELETE Path Pattern

When deleting an entity, a final audit record is created to capture its state at the moment of deletion. This ensures the audit trail is complete and immutable.

```rust
// DELETE path - delete entity with a final audit record

// 1. Load the full entities to be deleted to get their final state for auditing.
//    This is necessary because the delete operation only receives IDs.
let entities_to_delete = repo.load_batch(ids).await?;

for entity in &entities_to_delete {
    // 2. Create a final audit record before deletion. This follows a similar
    //    pattern to an update.
    let mut final_audit_entity = entity.clone();

    // Set antecedent fields from the entity's current state
    final_audit_entity.antecedent_hash = entity.hash;
    final_audit_entity.antecedent_audit_log_id = entity.audit_log_id.ok_or("Entity must have audit_log_id for deletion")?;

    // Set the new audit_log_id for this final "delete" event
    final_audit_entity.audit_log_id = Some(audit_log_id);
    final_audit_entity.hash = 0; // Set to 0 before final hashing

    // Compute the final hash for the audit record
    let final_hash = hash_as_i64(&final_audit_entity)?;
    final_audit_entity.hash = final_hash;

    // 3. Build the audit insert query for the final state snapshot
    let audit_insert_query = sqlx::query(
        r#"
        INSERT INTO named_audit
        (id, entity_type, name_l1, name_l2, name_l3, name_l4, description_l1, description_l2, description_l3, description_l4, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        "#,
        )
        .bind(final_audit_entity.id)
        .bind(final_audit_entity.entity_type)
        .bind(final_audit_entity.name_l1.as_str())
        .bind(final_audit_entity.name_l2.as_deref())
        .bind(final_audit_entity.name_l3.as_deref())
        .bind(final_audit_entity.name_l4.as_deref())
        .bind(final_audit_entity.description_l1.as_deref())
        .bind(final_audit_entity.description_l2.as_deref())
        .bind(final_audit_entity.description_l3.as_deref())
        .bind(final_audit_entity.description_l4.as_deref())
        .bind(final_audit_entity.antecedent_hash)
        .bind(final_audit_entity.antecedent_audit_log_id)
        .bind(final_audit_entity.hash)
        .bind(final_audit_entity.audit_log_id);

    // 4. Build the entity delete query. The corresponding index record
    //    will be deleted automatically via `ON DELETE CASCADE`.
    let entity_delete_query = sqlx::query(
        r#"
        DELETE FROM named WHERE id = $1
        "#,
    )
    .bind(entity.id);

    // Create audit link for the deleted entity
    let audit_link = AuditLinkModel {
        audit_log_id,
        entity_id: entity.id,
        entity_type: AuditEntityType::Named,
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

    // 5. Execute in transaction (audit first!)
    let mut tx = repo.executor.tx.lock().await;
    let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
    audit_insert_query.execute(&mut **transaction).await?;
    entity_delete_query.execute(&mut **transaction).await?;
    audit_link_query.execute(&mut **transaction).await?;

    // 6. Remove the entity from the cache
    repo.{entity}_idx_cache.read().await.remove(&entity.id);
}
```

---

## Database Schema

A complete migration script for an auditable entity includes the main table, an index table, and an audit table.

```sql
-- Migration: Initial {Entity} Schema with Audit Support
-- Description: Creates {entity}-related tables with audit trail.
 
-- Enum Types
CREATE TYPE named_entity_type AS ENUM ('Country', 'CountrySubdivision', 'Locality');

-- Enum for auditable entity types
CREATE TYPE audit_entity_type AS ENUM ('Location', 'Named', ...);
 
-- Main Named Table
-- Stores the current state of the entity.
CREATE TABLE IF NOT EXISTS named (
    id UUID PRIMARY KEY,
    entity_type named_entity_type NOT NULL,
    name_l1 VARCHAR(50) NOT NULL,
    name_l2 VARCHAR(50),
    name_l3 VARCHAR(50),
    name_l4 VARCHAR(50),
    description_l1 VARCHAR(255),
    description_l2 VARCHAR(255),
    description_l3 VARCHAR(255),
    description_l4 VARCHAR(255),
    hash BIGINT NOT NULL DEFAULT 0,
    audit_log_id UUID REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
);

-- Named Index Table
-- Contains fields for application-layer indexing and caching.
CREATE TABLE IF NOT EXISTS named_idx (
    id UUID PRIMARY KEY REFERENCES named(id) ON DELETE CASCADE,
    entity_type named_entity_type NOT NULL
);

-- Create trigger for named_idx table to notify listeners of changes
DROP TRIGGER IF EXISTS named_idx_notify ON named_idx;
CREATE TRIGGER named_idx_notify
    AFTER INSERT OR UPDATE OR DELETE ON named_idx
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();

-- Named Audit Table
-- Stores a complete, immutable snapshot of the entity at each change.
CREATE TABLE IF NOT EXISTS named_audit (
    -- All entity fields are duplicated here for a complete snapshot.
    id UUID NOT NULL,
    entity_type named_entity_type NOT NULL,
    name_l1 VARCHAR(50) NOT NULL,
    name_l2 VARCHAR(50),
    name_l3 VARCHAR(50),
    name_l4 VARCHAR(50),
    description_l1 VARCHAR(255),
    description_l2 VARCHAR(255),
    description_l3 VARCHAR(255),
    description_l4 VARCHAR(255),
    
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
CREATE INDEX IF NOT EXISTS idx_named_audit_id
    ON named_audit(id);
    
    -- Audit Link Table
    -- Tracks all entities modified in a single transaction.
    CREATE TABLE IF NOT EXISTS audit_link (
        audit_log_id UUID NOT NULL REFERENCES audit_log(id),
        entity_id UUID NOT NULL,
        entity_type audit_entity_type NOT NULL,
        PRIMARY KEY (audit_log_id, entity_id)
    );
    
    -- Index for audit_link table
    CREATE INDEX IF NOT EXISTS idx_audit_link_audit_log_id ON audit_link(audit_log_id);
    ```

---

## Key Differences from Base Template

### 1. Execution Order

**Critical**: Always insert audit record **BEFORE** modifying the main entity:

```rust
// ✅ CORRECT ORDER
audit_insert_query.execute(&mut **tx).await?;
entity_insert_or_update_query.execute(&mut **tx).await?;
idx_insert_or_update_query.execute(&mut **tx).await?;  // If needed

// ❌ WRONG ORDER - never do this
entity_update_query.execute(&mut **tx).await?;
audit_insert_query.execute(&mut **tx).await?;
```

### 2. Audit Chain Verification

The `antecedent_hash` and `antecedent_audit_log_id` fields in the entity itself enable audit chain verification:

**For CREATE operations**:
- Entity's `hash` = 0 before hashing (no previous state)
- Entity's `audit_log_id` = None
- `antecedent_hash` = 0 (no previous record)
- `antecedent_audit_log_id` = Uuid::nil() (no previous record)

**For UPDATE operations**:
- First, a change-detection step is performed. The current `hash` is stored, and a new hash is computed over the entity's data fields (with the `hash` field temporarily zeroed).
- If the new hash matches the stored hash, no data has changed, and the update process is aborted to avoid creating redundant audit records.
- If the data has changed, the entity's current `hash` and `audit_log_id` are copied to the `antecedent_hash` and `antecedent_audit_log_id` fields.
- Finally, a new hash is computed over the entire entity, which now includes the updated antecedent values and the new `audit_log_id`. This final hash is stored in the `hash` field before persisting the record.

This creates a cryptographic chain where each audit record links to its predecessor through the entity's own fields, eliminating the need to query the database for previous audit data.

---

## Testing Patterns for Auditable Entities

### Additional Test Cases

Beyond the base template tests, add:

```rust
#[tokio::test]
async fn test_create_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ctx = setup_test_context().await?;
    let audit_log_repo = &ctx.audit_repos().audit_log_repository;
    let named_repo = &ctx.description_repos().named_repository;

    let audit_log = create_test_audit_log();
    audit_log_repo.create(&audit_log).await?;

    let mut named_entities = Vec::new();
    for i in 0..5 {
        let named = create_test_named(&format!("Entity {i}"));
        named_entities.push(named);
    }

    let saved_entities = named_repo
        .create_batch(named_entities.clone(), Some(audit_log.id))
        .await?;

    assert_eq!(saved_entities.len(), 5);

    for (i, saved_entity) in saved_entities.iter().enumerate() {
        assert_eq!(saved_entity.name_l1.as_str(), format!("Entity {i}"));
        assert!(saved_entity.audit_log_id.is_some());
        assert_eq!(saved_entity.audit_log_id.unwrap(), audit_log.id);
    }

    Ok(())
}

#[tokio::test]
async fn test_update_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ctx = setup_test_context().await?;
    let audit_log_repo = &ctx.audit_repos().audit_log_repository;
    let named_repo = &ctx.description_repos().named_repository;

    let audit_log = create_test_audit_log();
    audit_log_repo.create(&audit_log).await?;

    let mut named_entities = Vec::new();
    for i in 0..3 {
        let named = create_test_named(&format!("Original Entity {i}"));
        named_entities.push(named);
    }

    let saved = named_repo.create_batch(named_entities, Some(audit_log.id)).await?;

    // Update entities
    // # Attention, we are updating in the same transaction. This will not happen in a real scenario
    // in order to prevent duplicate key, we will create a new audit log for the update.
    let update_audit_log = create_test_audit_log();
    audit_log_repo.create(&update_audit_log).await?;
    let mut updated_entities = Vec::new();
    for mut named in saved {
        named.name_l1 = HeaplessString::try_from("Updated Entity").unwrap();
        updated_entities.push(named);
    }

    let updated = named_repo.update_batch(updated_entities, Some(update_audit_log.id)).await?;

    assert_eq!(updated.len(), 3);
    for named in updated {
        assert_eq!(named.name_l1.as_str(), "Updated Entity");
    }

    Ok(())
}
```

#[tokio::test]
async fn test_delete_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ctx = setup_test_context().await?;
    let audit_log_repo = &ctx.audit_repos().audit_log_repository;
    let named_repo = &ctx.description_repos().named_repository;

    let audit_log = create_test_audit_log();
    audit_log_repo.create(&audit_log).await?;

    let mut named_entities = Vec::new();
    for i in 0..3 {
        let named = create_test_named(&format!("Entity to Delete {i}"));
        named_entities.push(named);
    }

    let saved = named_repo.create_batch(named_entities, Some(audit_log.id)).await?;

    let ids: Vec<Uuid> = saved.iter().map(|s| s.id).collect();
    // # Attention, we are deleting in the same transaction. This will not happen in a real scenario
    // in order to prevent duplicate key, we will create a new audit log for the delete.
    let delete_audit_log = create_test_audit_log();
    audit_log_repo.create(&delete_audit_log).await?;
    let deleted_count = named_repo.delete_batch(&ids, Some(delete_audit_log.id)).await?;

    assert_eq!(deleted_count, 3);

    Ok(())

#[tokio::test]
async fn test_load_audits() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ctx = setup_test_context().await?;
    let audit_log_repo = &ctx.audit_repos().audit_log_repository;
    // Create necessary dependencies (foreign keys, etc.)
    // ...

    let {entity}_repo = &ctx.{module}_repos().{entity}_repository;

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

## Validation Checklist

Extends the base template checklist with:

- [ ] Main entity model has `hash` and `audit_log_id` fields
- [ ] Auditable trait is implemented
- [ ] Hash computation uses the entity (with `hash=0`) pattern
- [ ] Entity's `hash` field is set to 0 before hashing (using a copy)
- [ ] For UPDATE: entity's current hash and audit_log_id are used as antecedent values
- [ ] For CREATE: entity's hash = 0 and audit_log_id = None initially
- [ ] Audit record is inserted **before** entity modification
- [ ] Batch operations (create/update) include audit_log_id parameter
- [ ] Audit table has composite primary key `(id, audit_log_id)`
- [ ] Audit table includes all entity fields, including `hash`, `audit_log_id`, `antecedent_hash`, and `antecedent_audit_log_id`
- [ ] Audit table does NOT have `ON DELETE CASCADE` (audit survives deletion)
- [ ] Audit table has index on `id` column (not `audit_log_id`) for efficient audit queries
- [ ] LoadAudits trait is implemented with pagination support
- [ ] LoadAudits tests verify correct pagination and audit history retrieval
- [ ] All audit tests pass (creation, update, hash integrity, chain verification)
- [ ] Migration includes audit table with correct schema
- [ ] Cleanup script removes audit table
- [ ] No version field in audit table
- [ ] **One finder method is created for each secondary index field** (field name not equal to `id`)
- [ ] **Comprehensive tests are implemented for ALL repository methods** (create_batch, load_batch, update_batch, delete_batch, exist_by_ids, and all custom finder methods)
- [ ] **Cache notification test is included** to verify direct database operations trigger cache updates
- [ ] All tests verify correct cache synchronization and audit trail integrity

---

## Usage Example

```rust
// Initialize repository factory (same as base template)
let description_factory = DescriptionRepoFactory::new(Some(&mut listener));

// Use with unit of work
let session = unit_of_work.start_session().await?;
let named_repo = description_factory.build_named_repo(&session);

// Create audit log entry first (application responsibility)
let audit_log_id = audit_log_repo.create_log(/* audit details */).await?;

// Create entity with audit
let named = NamedModel {
    id: Uuid::new_v4(),
    name_l1: HeaplessString::try_from("Example").unwrap(),
    // ... other fields
    audit_log_id: None,  // Will be set by create_batch operation
};

let saved_named = named_repo.create_batch(vec![named], Some(audit_log_id)).await?;

// Commit - audit record is persisted atomically with entity
session.commit().await?;
```

---

## Best Practices

### ✅ DO:

- Always create audit log entry before calling create/update operations
- Insert audit record before modifying the main entity
- Use the full entity model for audit records
- Set `entity.hash` to zero before computing the hash
- Use CBOR serialization for hash computation
- Store antecedent hash and audit_log_id for chain verification
- Test audit trail integrity and hash verification
- **Generate comprehensive tests for EVERY repository method implemented**
- **Include cache notification tests to verify database triggers work correctly**
- **Verify cache synchronization in all tests**

### ❌ DON'T:

- Modify main entity before creating audit record
- Skip hash computation or forget to set hash to zero before hashing
- Use different serialization for hashing
- Forget to update audit_log_id in the entity
- Create a dedicated audit model (all fields are in the main entity model)
- Use version numbers (audit records are keyed by id and audit_log_id)
- **Skip test generation for any repository method**
- **Forget to test cache notification mechanism**

---

## Audit Chain Verification

The entity's audit fields enable cryptographic audit chain verification:

```rust
// Entity Model contains hash and audit_log_id
pub struct {Entity}Model {
    pub hash: i64,                       // Hash of the entity (with hash=0)
    pub audit_log_id: Option<Uuid>,     // Current audit log reference
    // ... other entity fields
}
```

**Chain Verification Process**:

1. Load all audit records for an entity ordered by creation time
2. For the first record (CREATE):
   - Verify `antecedent_hash` = 0
   - Verify `antecedent_audit_log_id` = Uuid::nil()
   - Verify the hash by recomputing it from the entity (with hash=0)
3. For subsequent records (UPDATE):
   - Verify the hash by recomputing it from the entity (with hash=0)
   - Verify `antecedent_hash` matches the previous record's `hash` field
   - Verify `antecedent_audit_log_id` matches the previous record's `audit_log_id` field
4. Any mismatch indicates tampering or data corruption

This creates an immutable audit chain where each record cryptographically links to its predecessor through the entity's own audit fields, enabling complete history verification.

---

## References

- **Base Template**: [Entity with Index](entity_with_index.md)
- **Audit Traits**: `business-core/business-core-db/src/models/auditable.rs`
- **Audit Models**: `business-core/business-core-db/src/models/audit/`
- **Complete Example**: Named entity in `business-core`
- **Hash Library**: [twox-hash](https://docs.rs/twox-hash/)
- **CBOR Library**: [ciborium](https://docs.rs/ciborium/)

---

## License

Same as business-core project

### Cache Notification on Direct Insert Test

This test verifies that a direct `INSERT` into the entity's table (and its index table) correctly triggers a database notification that the application's cache listener picks up. This ensures that even database changes made outside the application's repository layer (e.g., by another service or a DBA) are reflected in the cache.

**Key steps in this test:**
1.  Set up a test context with a `CacheNotificationListener` using `setup_test_context_and_listen()`.
2.  Create all prerequisite records (e.g., country, subdivision, locality, audit_log) via direct `sqlx::query` inserts. This simulates an external process writing to the database.
3.  Insert the main entity and its index record directly.
4.  Wait for a short duration to allow the notification to be processed.
5.  Verify that the entity now exists in the repository's cache.
6.  Delete all created records directly.
7.  Wait again for the delete notification to be processed.
8.  Verify that the entity has been removed from the cache.

```rust
#[tokio::test]
async fn test_named_insert_triggers_cache_notification() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    // Setup test context with the handler
    let ctx = setup_test_context_and_listen().await?;
    let pool = ctx.pool();

    // Create a test named entity
    let test_named = create_test_named(&random(20));
    let named_idx = test_named.to_index();

    // Give listener more time to start and establish connection
    // The listener needs time to connect and execute LISTEN command
    sleep(Duration::from_millis(2000)).await;

    // Insert the named record
    let audit_log = create_test_audit_log();
    sqlx::query(
        r#"
        INSERT INTO audit_log (id, updated_at, updated_by_person_id)
        VALUES ($1, $2, $3)
        "#,
    )
    .bind(audit_log.id)
    .bind(audit_log.updated_at)
    .bind(audit_log.updated_by_person_id)
    .execute(&**pool)
    .await
    .expect("Failed to insert audit log");
    
    let mut test_named_for_hashing = test_named.clone();
    test_named_for_hashing.hash = 0;
    test_named_for_hashing.audit_log_id = Some(audit_log.id);
    let computed_hash =
        business_core_db::utils::hash_as_i64(&test_named_for_hashing).unwrap();
    let final_named = NamedModel {
        hash: computed_hash,
        audit_log_id: Some(audit_log.id),
        ..test_named
    };

    sqlx::query(
        r#"
        INSERT INTO named
        (id, entity_type, name_l1, name_l2, name_l3, name_l4, description_l1, description_l2, description_l3, description_l4, antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
        VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12, $13, $14)
        "#,
    )
    .bind(final_named.id)
    .bind(final_named.entity_type)
    .bind(final_named.name_l1.as_str())
    .bind(final_named.name_l2.as_deref())
    .bind(final_named.name_l3.as_deref())
    .bind(final_named.name_l4.as_deref())
    .bind(final_named.description_l1.as_deref())
    .bind(final_named.description_l2.as_deref())
    .bind(final_named.description_l3.as_deref())
    .bind(final_named.description_l4.as_deref())
    .bind(final_named.antecedent_hash)
    .bind(final_named.antecedent_audit_log_id)
    .bind(final_named.hash)
    .bind(final_named.audit_log_id)
    .execute(&**pool)
    .await
    .expect("Failed to insert named");

    // Then insert the named index directly into the database using raw SQL
    sqlx::query("INSERT INTO named_idx (id, entity_type) VALUES ($1, $2)")
        .bind(named_idx.id)
        .bind(named_idx.entity_type)
        .execute(&**pool)
        .await
        .expect("Failed to insert named index");

    // Give more time for notification to be processed
    sleep(Duration::from_millis(500)).await;

    let named_repo = &ctx.description_repos().named_repository;

    // Verify the cache was updated via the trigger
    let cache = named_repo.named_idx_cache.read().await;
    assert!(
        cache.contains_primary(&named_idx.id),
        "Named should be in cache after insert"
    );

    let cached_named = cache.get_by_primary(&named_idx.id);
    assert!(
        cached_named.is_some(),
        "Named should be retrievable from cache"
    );

    // Verify the cached data matches
    let cached_named = cached_named.unwrap();
    assert_eq!(cached_named.id, named_idx.id);
    assert_eq!(cached_named.entity_type, named_idx.entity_type);

    // Drop the read lock before proceeding to allow notification handler to process
    drop(cache);

    // Delete the records from the database, will cascade delete named_idx
    sqlx::query("DELETE FROM named WHERE id = $1")
        .bind(named_idx.id)
        .execute(&**pool)
        .await
        .expect("Failed to delete named");

    sqlx::query("DELETE FROM audit_log WHERE id = $1")
        .bind(audit_log.id)
        .execute(&**pool)
        .await
        .expect("Failed to delete audit log");

    // Give more time for notification to be processed
    sleep(Duration::from_millis(500)).await;

    // Verify the cache entry was removed
    let cache = named_repo.named_idx_cache.read().await;
    assert!(
        !cache.contains_primary(&named_idx.id),
        "Named should be removed from cache after delete"
    );

    Ok(())
}
```