# Entity Template Skill: Indexable and Auditable Entities

## Overview

This skill extends the [Entity with Index](entity_with_index.md) template to add **audit trail functionality**. It generates complete database access modules for entities that require both application-layer indexing and comprehensive audit logging.

## Purpose

This skill builds upon the base indexable entity template by adding:
- ✅ **Audit table** with complete entity state snapshots stored as tuples
- ✅ **Hash-based verification** using xxHash64 for audit integrity
- ✅ **Audit log integration** with audit_log_id references
- ✅ **Serialization-based hashing** using CBOR encoding

## Prerequisites

**You must read and understand** [Entity with Index](entity_with_index.md) first. This document only covers the **additional** patterns for audit functionality. All base patterns from the indexable entity template still apply.

---

## Template Reference

The audit pattern is based on the Location entity implementation:
- **Model**: `ledger-banking-rust/banking-db/src/models/person/location.rs`
- **Repository**: `ledger-banking-rust/banking-db-postgres/src/repository/person/location_repository/`
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
async fn create_batch(&self, items: Vec<{Entity}Model>, audit_log_id: Uuid)

async fn update_batch(&self, items: Vec<{Entity}Model>, audit_log_id: Uuid)

async fn delete_batch(&self, ids: &[Uuid], audit_log_id: Uuid)
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
    INSERT INTO {table_name}_audit
    (id, field1, field2, ..., hash, audit_log_id)
    VALUES ($1, $2, $3, ..., $N, $N+1)
    "#,
)
.bind(entity.id)
// Bind all entity fields (including hash and audit_log_id)
.bind(entity.field1.as_str())
.bind(entity.field2.as_str())
// ... bind remaining entity fields
.bind(entity.hash)  // Entity's hash field (computed above)
.bind(entity.audit_log_id)  // Entity's audit_log_id field (set above)
// Antecedent fields have default value for a new entity. So no need to bind.

// Build entity insert query
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
.bind(entity.hash)  // Set hash
.bind(entity.audit_log_id);  // Set audit_log_id
// Antecedent fields have default value for a new entity. So no need to bind.

// Build index insert query (same as base template)
let idx_insert_query = sqlx::query(
    r#"
    INSERT INTO {table_name}_idx
    (id, index_field1, index_field2, ...)
    VALUES ($1, $2, $3, ...)
    "#,
)
.bind(entity.id)
// ... bind index fields
;

// Execute in transaction (audit first!)
match &repo.executor {
    Executor::Pool(pool) => {
        audit_insert_query.execute(&**pool).await?;
        entity_insert_query.execute(&**pool).await?;
        idx_insert_query.execute(&**pool).await?;
    }
    Executor::Tx(tx) => {
        let mut tx = tx.lock().await;
        audit_insert_query.execute(&mut **tx).await?;
        entity_insert_query.execute(&mut **tx).await?;
        idx_insert_query.execute(&mut **tx).await?;
    }
}

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
    INSERT INTO {table_name}_audit
    (id, field1, field2, ..., hash, audit_log_id, antecedent_hash, antecedent_audit_log_id)
    VALUES ($1, $2, $3, ..., $N, $N+1, $N+2, $N+3)
    "#,
)
.bind(entity.id)
// Bind all entity fields (including hash and audit_log_id)
.bind(entity.field1.as_str())
// ... bind remaining entity fields
.bind(entity.hash)  // Entity's hash field (updated above)
.bind(entity.audit_log_id)  // Entity's audit_log_id field (updated above)
// Bind antecedent fields
.bind(entity.antecedent_hash)
.bind(entity.antecedent_audit_log_id);

// Build entity update query
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
      AND hash = $N+2
      AND audit_log_id = $N+3
    "#,
)
.bind(entity.id)
.bind(entity.field1.as_str())
// ... bind all fields
.bind(entity.hash)  // Update hash
.bind(entity.audit_log_id)  // Update audit_log_id
// Bind antecedent fields
.bind(entity.antecedent_hash)
.bind(entity.antecedent_audit_log_id);

// Execute in transaction (audit first!)
match &repo.executor {
    Executor::Pool(pool) => {
        audit_insert_query.execute(&**pool).await?;
        entity_update_query.execute(&**pool).await?;
    }
    Executor::Tx(tx) => {
        let mut tx = tx.lock().await;
        audit_insert_query.execute(&mut **tx).await?;
        entity_update_query.execute(&mut **tx).await?;
    }
}

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

    // 4. Build the entity delete query. The corresponding index record
    //    will be deleted automatically via `ON DELETE CASCADE`.
    let entity_delete_query = sqlx::query(
        r#"
        DELETE FROM {table_name} WHERE id = $1
        "#,
    )
    .bind(entity.id);

    // 5. Execute in transaction (audit first!)
    match &repo.executor {
        Executor::Pool(_pool) => {
            // In a real scenario, this would be part of a larger transaction
            // managed by a Unit of Work.
            panic!("Delete should be executed within a transaction");
        }
        Executor::Tx(tx) => {
            let mut tx = tx.lock().await;
            audit_insert_query.execute(&mut **tx).await?;
            entity_delete_query.execute(&mut **tx).await?;
        }
    }

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

-- {Entity} Index Table
-- Contains fields for application-layer indexing and caching.
CREATE TABLE IF NOT EXISTS {table_name}_idx (
    id UUID PRIMARY KEY REFERENCES {table_name}(id) ON DELETE CASCADE,
    index_field1 BIGINT,
    index_field2 UUID,
    -- ... other index fields
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

-- Index on audit_log_id for efficient audit log queries.
-- Note: The audit table intentionally lacks a foreign key to the main table
-- with `ON DELETE CASCADE`. This ensures that audit history is preserved
-- even if the main entity record is deleted.
CREATE INDEX IF NOT EXISTS idx_{table_name}_audit_audit_log_id
    ON {table_name}_audit(audit_log_id);
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
async fn test_create_batch_creates_audit_record() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ctx = setup_test_context().await?;
    let repo = &ctx.{module}_repos().{entity}_repository;
    
    let entity = create_test_{entity}(/* params */);
    let audit_log_id = Uuid::new_v4();
    
    // Create entity
    let saved = repo.create_batch(vec![entity.clone()], audit_log_id).await?;
    
    // Verify audit record exists
    let audit_records = load_audit_records(&ctx, saved[0].id).await?;
    assert_eq!(audit_records.len(), 1);
    assert_eq!(audit_records[0].audit_log_id, Some(audit_log_id));
    
    Ok(())
}

#[tokio::test]
async fn test_update_batch_creates_new_audit_record() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ctx = setup_test_context().await?;
    let repo = &ctx.{module}_repos().{entity}_repository;
    
    // Initial create
    let entity = create_test_{entity}(/* params */);
    let audit_log_id_1 = Uuid::new_v4();
    let saved = repo.create_batch(vec![entity.clone()], audit_log_id_1).await?;
    
    // Update
    let mut updated_entity = saved[0].clone();
    updated_entity.field1 = /* new value */;
    let audit_log_id_2 = Uuid::new_v4();
    repo.update_batch(vec![updated_entity], audit_log_id_2).await?;
    
    // Verify two audit records
    let audit_records = load_audit_records(&ctx, saved[0].id).await?;
    assert_eq!(audit_records.len(), 2);
    assert_eq!(audit_records[0].audit_log_id, Some(audit_log_id_1));
    assert_eq!(audit_records[1].audit_log_id, Some(audit_log_id_2));
    
    Ok(())
}

#[tokio::test]
async fn test_audit_hash_integrity() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let ctx = setup_test_context().await?;
    let repo = &ctx.{module}_repos().{entity}_repository;
    
    let entity = create_test_{entity}(/* params */);
        let audit_log_id = Uuid::new_v4();
        let saved = repo.create_batch(vec![entity.clone()], audit_log_id).await?;
        
        // Load audit record and verify hash
        let audit_records = load_audit_records(&ctx, saved[0].id).await?;
        let stored_entity = &audit_records[0];
        
        // Recompute hash
        let mut entity_for_hashing = stored_entity.clone();
        entity_for_hashing.hash = 0;
        let computed_hash = hash_as_i64(&entity_for_hashing)?;
        
        assert_eq!(stored_entity.hash, computed_hash, "Hash mismatch");
    
    Ok(())
}
```

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
- [ ] All audit tests pass (creation, update, hash integrity, chain verification)
- [ ] Migration includes audit table with correct schema
- [ ] Cleanup script removes audit table
- [ ] No version field in audit table

---

## Usage Example

```rust
// Initialize repository factory (same as base template)
let person_factory = PersonRepoFactory::new(Some(&mut listener));

// Use with unit of work
let session = unit_of_work.start_session().await?;
let location_repo = person_factory.build_location_repo(&session);

// Create audit log entry first (application responsibility)
let audit_log_id = audit_log_repo.create_log(/* audit details */).await?;

// Create entity with audit
let location = LocationModel {
    id: Uuid::new_v4(),
    street_line1: HeaplessString::try_from("123 Main St").unwrap(),
    locality_id: locality.id,
    // ... other fields
    audit_log_id: None,  // Will be set by create_batch operation
};

let saved_locations = location_repo.create_batch(vec![location], audit_log_id).await?;

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

### ❌ DON'T:

- Modify main entity before creating audit record
- Skip hash computation or forget to set hash to zero before hashing
- Use different serialization for hashing
- Forget to update audit_log_id in the entity
- Create a dedicated audit model (all fields are in the main entity model)
- Use version numbers (audit records are keyed by id and audit_log_id)

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
- **Complete Example**: Location entity in `ledger-banking-rust`
- **Hash Library**: [twox-hash](https://docs.rs/twox-hash/)
- **CBOR Library**: [ciborium](https://docs.rs/ciborium/)

---

## License

Same as business-core project