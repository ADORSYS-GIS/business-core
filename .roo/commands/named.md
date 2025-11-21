# Generate Named Entity with Audit Support

## Objective
Generate a complete auditable entity implementation for the `Named` entity following the auditable entity pattern (without indexing). The Named entity provides multilingual support for names and descriptions.

## Source Material
- **Skill Template**: `docs/skills/entity_template/entity_with_audit.md`
- **Example Structure**: `business-core-db/src/models/description/named_example.rs`
- **Reference Implementation**: `business-core-db/src/models/reason_and_purpose/reason_reference.rs`
- **Reference Repository**: `business-core-postgres/src/repository/reason_and_purpose/reason_reference_repository/`

## Entity Specification

### Named Model Fields
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
    // Audit fields
    pub antecedent_hash: i64,
    pub antecedent_audit_log_id: Uuid,
    pub hash: i64,
    pub audit_log_id: Option<Uuid>,
}
```

**Note**: The example file has duplicate field names (`name_l3` and `description_l3` appear twice). The correct fields should be `name_l1`, `name_l2`, `name_l3`, `name_l4` and `description_l1`, `description_l2`, `description_l3`, `description_l4`.

### Entity Characteristics
- **Module**: `description`
- **Table Name**: `named`
- **Entity Type**: Add `Named` to `EntityType` enum in `business-core-db/src/models/audit/entity_type.rs`
- **Auditable**: Yes (complete audit trail with hash verification)
- **Indexable**: No (accessed by ID only)
- **Cacheable**: No (no in-memory cache)

## Implementation Tasks

### 1. Model Implementation
**File**: `business-core-db/src/models/description/named.rs`

- [ ] Define `NamedModel` struct with all fields (correcting duplicate field names from example)
- [ ] Add audit fields: `antecedent_hash`, `antecedent_audit_log_id`, `hash`, `audit_log_id`
- [ ] Implement `Identifiable` trait
- [ ] Implement `Auditable` trait
- [ ] Add proper documentation comments
- [ ] Use `HeaplessString<50>` for name fields
- [ ] Use `HeaplessString<255>` for description fields

### 2. Module Registration
**File**: `business-core-db/src/models/description/mod.rs`

- [ ] Create if not exists
- [ ] Add `pub mod named;` declaration

**File**: `business-core-db/src/models/mod.rs`

- [ ] Add `pub mod description;` if not already present

**File**: `business-core-db/src/models/audit/entity_type.rs`

- [ ] Add `Named` variant to `EntityType` enum
- [ ] Update all match statements to include `Named` variant

### 3. Repository Implementation
**Directory**: `business-core-postgres/src/repository/description/named_repository/`

#### 3.1 Repository Structure Files

**File**: `repo_impl.rs`
- [ ] Define `NamedRepositoryImpl` struct with `executor: Executor`
- [ ] Implement `TryFromRow<PgRow>` for `NamedModel` with proper field mappings:
  - Use `get_heapless_string()` for `name_l1`
  - Use `get_optional_heapless_string()` for optional name and description fields
- [ ] Implement `TransactionAware` trait (simple version, no cache)

**File**: `create_batch.rs`
- [ ] Implement `create_batch_impl` following the CREATE pattern
- [ ] Hash computation with `hash=0` before hashing
- [ ] Insert into audit table first, then main table, then audit_link
- [ ] Implement `CreateBatch` trait
- [ ] Add test: `test_create_batch` - create 3 entities
- [ ] Add test: `test_create_batch_empty` - handle empty batch

**File**: `load_batch.rs`
- [ ] Implement `load_batch_impl` with ID-based loading
- [ ] Query: `SELECT * FROM named WHERE id = ANY($1)`
- [ ] Implement `LoadBatch` trait
- [ ] Add test: `test_load_batch` - load existing entity
- [ ] Add test: `test_load_batch_not_found` - handle missing entity

**File**: `update_batch.rs`
- [ ] Implement `update_batch_impl` following UPDATE pattern
- [ ] Track antecedent hash and audit_log_id
- [ ] Check if entity actually changed before updating
- [ ] Update audit table first, then main table, then audit_link
- [ ] Implement `UpdateBatch` trait
- [ ] Add test: `test_update_batch` - update entity fields
- [ ] Add test: `test_update_batch_no_change` - verify no update when unchanged

**File**: `delete_batch.rs`
- [ ] Implement `delete_batch_impl` following DELETE pattern
- [ ] Load full entities before deletion
- [ ] Create final audit record before deletion
- [ ] Delete from main table (audit survives)
- [ ] Implement `DeleteBatch` trait
- [ ] Add test: `test_delete_batch` - delete entities
- [ ] Add test: `test_delete_batch_not_found` - handle missing entities

**File**: `exist_by_ids.rs`
- [ ] Implement `exist_by_ids_impl` with direct database query
- [ ] Query: `SELECT id FROM named WHERE id = ANY($1)`
- [ ] Implement `ExistById` trait
- [ ] Add test: `test_exist_by_ids` - check existing and non-existing IDs

**File**: `load_audits.rs`
- [ ] Implement `load_audits_impl` with pagination
- [ ] Count query: `SELECT COUNT(*) FROM named_audit WHERE id = $1`
- [ ] Paginated query with ORDER BY audit_log_id DESC
- [ ] Implement `LoadAudits` trait
- [ ] Add test: `test_load_audits` - verify audit history pagination
- [ ] Add test: `test_load_audits_empty` - handle non-existent entity

**File**: `test_utils.rs`
- [ ] Create `create_test_named()` helper function
- [ ] Create `create_test_named_with_all_languages()` helper function
- [ ] Initialize audit fields to default values (hash=0, audit_log_id=None)

**File**: `mod.rs`
- [ ] Declare all submodules
- [ ] Export `NamedRepositoryImpl`

### 4. Repository Factory Integration

**File**: `business-core-postgres/src/repository/description/factory.rs`
- [ ] Create `DescriptionRepoFactory` struct (no cache fields)
- [ ] Implement `new()` method
- [ ] Implement `build_named_repo()` method
- [ ] Register repository as `TransactionAware`

**File**: `business-core-postgres/src/repository/description/mod.rs`
- [ ] Create if not exists
- [ ] Add `pub mod named_repository;`
- [ ] Add `pub mod factory;`
- [ ] Export factory

**File**: `business-core-postgres/src/repository/mod.rs`
- [ ] Add `pub mod description;` if not present

### 5. Database Schema

**File**: `business-core-postgres/migrations/0XX_initial_schema_named.sql`
- [ ] Create main `named` table with all fields
- [ ] Add audit fields: `hash`, `audit_log_id`, `antecedent_hash`, `antecedent_audit_log_id`
- [ ] Add foreign key to `audit_log(id)` for `audit_log_id`
- [ ] Create `named_audit` table with same structure
- [ ] Use composite primary key `(id, audit_log_id)` for audit table
- [ ] Create index `idx_named_audit_id` on `named_audit(id)`
- [ ] NO foreign key cascade from audit to main table (preserve audit on deletion)

**File**: `business-core-postgres/cleanup/0XX_cleanup_named.sql`
- [ ] Drop `named_audit` table
- [ ] Drop `named` table

### 6. Testing Requirements

Each repository method must have at least one test:
- [ ] `test_create_batch` - Create multiple entities
- [ ] `test_create_batch_empty` - Empty batch handling
- [ ] `test_load_batch` - Load by IDs
- [ ] `test_load_batch_not_found` - Handle missing entities
- [ ] `test_update_batch` - Update entities
- [ ] `test_update_batch_no_change` - No-op when unchanged
- [ ] `test_delete_batch` - Delete entities
- [ ] `test_delete_batch_not_found` - Handle missing entities
- [ ] `test_exist_by_ids` - Check existence
- [ ] `test_load_audits` - Verify audit history with pagination
- [ ] `test_load_audits_empty` - Handle non-existent entity

## Key Implementation Notes

1. **No Indexing**: This entity does not have an index table or hash-based lookups
2. **No Caching**: No in-memory cache infrastructure
3. **Audit Trail**: Complete audit history with hash chain verification
4. **Field Names**: Fix duplicate field names in example - use l1, l2, l3, l4 suffixes
5. **String Sizes**: 50 chars for names, 255 chars for descriptions
6. **Audit First**: Always insert audit record before modifying main entity
7. **Hash Verification**: Set hash=0 before computing hash value
8. **Optional Fields**: All language variants except l1 are optional

## Validation Checklist

After implementation, verify:
- [ ] Model has all required audit fields
- [ ] `Identifiable` and `Auditable` traits implemented
- [ ] No `IndexAware` or `Indexable` traits (not needed)
- [ ] Repository has no cache infrastructure
- [ ] All CRUD operations follow audit-first pattern
- [ ] Hash computation uses entity with hash=0
- [ ] Tests pass for all repository methods
- [ ] Migration creates main and audit tables
- [ ] Audit table has composite PK (id, audit_log_id)
- [ ] No ON DELETE CASCADE from audit to main
- [ ] `EntityType::Named` added and integrated

## Cleanup Tasks

- [ ] Delete `business-core-db/src/models/description/named_example.rs` after implementation is complete
- [ ] Verify no references to the example file remain

## References

- **Skill Documentation**: `docs/skills/entity_template/entity_with_audit.md`
- **Reference Model**: `business-core-db/src/models/reason_and_purpose/reason_reference.rs`
- **Reference Repository**: `business-core-postgres/src/repository/reason_and_purpose/reason_reference_repository/`