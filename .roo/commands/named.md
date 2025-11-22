# Generate Named Entity Model and Repository

## Context
Generate complete model and repository code for the `Named` entity using the entity template skill. The `Named` entity provides multilingual support for names and descriptions with up to 4 language variants.

## Reference Files
- **Skill Template**: `docs/skills/entity_template/entity_with_index_and_audit.md`
- **Sample Model**: `business-core-db/src/models/description/named_example.rs`
- **Example Model**: `business-core-db/src/models/person/person.rs`
- **Example Repository**: `business-core-postgres/src/repository/person/person_repository`

## Entity Characteristics
- **Entity Name**: Named
- **Module**: description
- **Table Name**: named
- **Auditable**: Yes (includes hash, audit_log_id, antecedent_hash, antecedent_audit_log_id)
- **Indexable**: Yes (includes entity_type field, but it is not an index field)

## Model Structure

### NamedModel Fields
```rust
pub struct NamedModel {
    pub id: Uuid,
    pub entity_type: NamedEntityType,
    pub name_l1: HeaplessString<50>,
    pub name_l2: Option<HeaplessString<50>>,
    pub name_l3: Option<HeaplessString<50>>,
    pub name_l4: Option<HeaplessString<50>>,
    pub description_l1: Option<HeaplessString<255>>,
    pub description_l2: Option<HeaplessString<255>>,
    pub description_l3: Option<HeaplessString<255>>,
    pub description_l4: Option<HeaplessString<255>>,
    pub antecedent_hash: i64,
    pub antecedent_audit_log_id: Uuid,
    pub hash: i64,
    pub audit_log_id: Option<Uuid>,
}
```

### NamedIdxModel Fields
```rust
pub struct NamedIdxModel {
    pub id: Uuid,
    pub entity_type: NamedEntityType,
}
```

## Index Fields Analysis
- **Primary Key**: `id` (no finder needed)

## Tasks to Complete

### 1. Model File
- [ ] Create `business-core-db/src/models/description/named.rs` based on `named_example.rs`
- [ ] Implement `Identifiable` trait for `NamedModel`
- [ ] Implement `Auditable` trait for `NamedModel`
- [ ] Implement `IndexAware` trait for `NamedModel`
- [ ] Implement `HasPrimaryKey` trait for `NamedIdxModel`
- [ ] Implement `Identifiable` trait for `NamedIdxModel`
- [ ] Implement `Index` trait for `NamedIdxModel`
- [ ] Implement `Indexable` trait for `NamedIdxModel` (no secondary indexes needed)
- [ ] Add serialization support for `NamedEntityType` using existing functions in `named_entity_type.rs`

### 2. Update Model Module
- [ ] Update `business-core-db/src/models/description/mod.rs` to export `named` instead of `named_example`

### 3. Repository Structure
Create directory: `business-core-postgres/src/repository/description/named_repository/`

### 4. Repository Implementation Files
- [ ] **repo_impl.rs** - Main repository implementation with cache
- [ ] **create_batch.rs** - Create entities with audit trail
- [ ] **load_batch.rs** - Load entities by IDs
- [ ] **load_audits.rs** - Load audit history with pagination
- [ ] **update_batch.rs** - Update entities with audit trail and change detection
- [ ] **delete_batch.rs** - Delete entities with final audit record
- [ ] **exist_by_ids.rs** - Check existence of entities
- [ ] **find_by_entity_type.rs** - Finder method for entity_type secondary index
- [ ] **test_utils.rs** - Test helper functions

### 5. Repository Module Files
- [ ] Create `business-core-postgres/src/repository/description/mod.rs`
- [ ] Create `business-core-postgres/src/repository/description/factory.rs`
- [ ] Update `business-core-postgres/src/repository/mod.rs` to include description module

### 6. Test Implementation
Implement comprehensive tests in each repository file:

#### test_utils.rs
- [ ] `create_test_named()` - Helper to create test Named entities
- [ ] `setup_test_context()` - Test context setup helper

#### create_batch.rs
- [ ] `test_create_batch()` - Test creating multiple Named entities

#### load_batch.rs
- [ ] `test_load_batch()` - Test loading entities by IDs
- [ ] `test_load_batch_empty()` - Test loading non-existent IDs

#### update_batch.rs
- [ ] `test_update_batch()` - Test updating entities with change detection
- [ ] `test_update_no_change()` - Test that unchanged entities don't create audit records

#### delete_batch.rs
- [ ] `test_delete_batch()` - Test deleting entities with final audit record

#### exist_by_ids.rs
- [ ] `test_exist_by_ids()` - Test checking existence of entities

#### load_audits.rs
- [ ] `test_load_audits()` - Test loading audit history with pagination
- [ ] `test_load_audits_empty()` - Test loading audits for non-existent entity

#### Cache Notification Test
- [ ] `test_named_insert_triggers_cache_notification()` - Test direct database insert triggers cache update

### 7. Database Schema
The schema already exists as:
- Migration: `business-core-postgres/migrations/002_initial_schema_named.sql`
- Cleanup: `business-core-postgres/cleanup/002_cleanup_named.sql`

### 8. Cleanup
- [ ] Delete `business-core-db/src/models/description/named_example.rs` after successful generation

## Implementation Guidelines

### Hash Computation Pattern
```rust
use business_core_db::utils::hash_as_i64;

// For CREATE
let mut entity_for_hashing = entity.clone();
entity_for_hashing.hash = 0;
entity_for_hashing.audit_log_id = Some(audit_log_id);
let computed_hash = hash_as_i64(&entity_for_hashing)?;
entity.hash = computed_hash;
entity.audit_log_id = Some(audit_log_id);
```

### SQL Query Pattern for Create
```sql
INSERT INTO named_audit
(id, entity_type, name_l1, name_l2, name_l3, name_l4, 
 description_l1, description_l2, description_l3, description_l4, 
 hash, audit_log_id)
VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11, $12)
```

### SQL Query Pattern for Update
```sql
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
    hash = $11,
    audit_log_id = $12,
    antecedent_hash = $13,
    antecedent_audit_log_id = $14
WHERE id = $1
  AND hash = $13
  AND audit_log_id = $14
```

## Validation Checklist
- [ ] All model traits implemented correctly
- [ ] Repository methods include audit_log_id parameter
- [ ] Hash computation follows the pattern (entity with hash=0)
- [ ] Audit records inserted before entity modifications
- [ ] Change detection in update_batch prevents redundant audit records
- [ ] Delete operations create final audit record
- [ ] All repository methods have comprehensive tests (at least one test per method)
- [ ] Cache notification test implemented
- [ ] Test helper functions in test_utils.rs
- [ ] Factory pattern implemented for repository creation
- [ ] Module files updated correctly

## Expected File Structure
```
business-core-db/src/models/description/
  ├── mod.rs (updated)
  ├── named.rs (new)
  └── named_entity_type.rs (existing)

business-core-postgres/src/repository/description/
  ├── mod.rs (new)
  ├── factory.rs (new)
  └── named_repository/
      ├── mod.rs (new)
      ├── repo_impl.rs (new)
      ├── create_batch.rs (new)
      ├── load_batch.rs (new)
      ├── load_audits.rs (new)
      ├── update_batch.rs (new)
      ├── delete_batch.rs (new)
      ├── exist_by_ids.rs (new)
      └── test_utils.rs (new)
```

## Notes
- The entity_type field uses the existing `NamedEntityType` enum from `named_entity_type.rs`
- Serialization functions already exist: `serialize_entity_type` and `deserialize_entity_type`
- No additional enums need to be created
- The entity is auditable AND indexable, so follow the full auditable entity template
- Entity has NO secondary index field.  Field (entity_type) is not an index field