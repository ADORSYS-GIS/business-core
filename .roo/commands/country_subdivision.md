# Generate CountrySubdivision Entity

Apply the entity template skill from `business-core/docs/skills/entity_template` to generate a complete database access module for the CountrySubdivision entity in business-core.

## Entity Specification

### Entity Definition

```yaml
entity:
  name: "CountrySubdivision"
  module: "person"
  table_name: "country_subdivision"
  idx_table_name: "country_subdivision_idx"
```

### Fields Specification

```yaml
fields:
  - name: "id"
    type: "Uuid"
    nullable: false
    primary_key: true
    description: "Primary identifier for country subdivision"
    
  - name: "country_id"
    type: "Uuid"
    nullable: false
    indexed: true
    description: "Foreign key to CountryModel.id"
    constraint: "exists(CountryModel.id)"
    
  - name: "code"
    type: "HeaplessString<10>"
    nullable: false
    indexed: true
    unique: true
    description: "Subdivision code - if non-existent, use first 10 chars of name_l1"
    
  - name: "name_l1"
    type: "HeaplessString<100>"
    nullable: false
    description: "Primary language name"
    
  - name: "name_l2"
    type: "HeaplessString<100>"
    nullable: true
    description: "Secondary language name (optional)"
    
  - name: "name_l3"
    type: "HeaplessString<100>"
    nullable: true
    description: "Tertiary language name (optional)"
```

### Index Keys

```yaml
index_keys:
  i64_keys:
    - field: "code"
      index_name: "code_hash"
      hash_function: "hash_as_i64"
      unique: true
      description: "Hash-based unique index for subdivision code"
  
  uuid_keys:
    - field: "country_id"
      index_name: "country_id"
      description: "Foreign key index to country"
```

### Custom Query Methods

```yaml
custom_queries:
  - name: "find_ids_by_code_hash"
    parameters:
      - name: "code_hash"
        type: "i64"
    return_type: "Vec<Uuid>"
    cache_based: true
    description: "Find subdivision IDs by code hash (should return single result due to uniqueness)"
    
  - name: "find_ids_by_country_id"
    parameters:
      - name: "country_id"
        type: "Uuid"
    return_type: "Vec<Uuid>"
    cache_based: true
    description: "Find all subdivision IDs for a given country"
```

---

## Database Schema Files

### Migration File

**Path**: `business-core/business-core-postgres/migrations/003_initial_schema_person_country_subdivision.sql`

```sql
-- Migration: Initial CountrySubdivision Schema
-- Description: Creates country_subdivision-related tables and indexes

-- CountrySubdivision Table
CREATE TABLE IF NOT EXISTS country_subdivision (
    id UUID PRIMARY KEY,
    country_id UUID NOT NULL REFERENCES country(id) ON DELETE CASCADE,
    code VARCHAR(10) NOT NULL UNIQUE,
    name_l1 VARCHAR(100) NOT NULL,
    name_l2 VARCHAR(100),
    name_l3 VARCHAR(100)
);

-- CountrySubdivision Index Table
CREATE TABLE IF NOT EXISTS country_subdivision_idx (
    id UUID PRIMARY KEY REFERENCES country_subdivision(id) ON DELETE CASCADE,
    country_id UUID NOT NULL,
    code_hash BIGINT NOT NULL UNIQUE
);

-- Create trigger for country_subdivision_idx table to notify listeners of changes
DROP TRIGGER IF EXISTS country_subdivision_idx_notify ON country_subdivision_idx;
CREATE TRIGGER country_subdivision_idx_notify
    AFTER INSERT OR UPDATE OR DELETE ON country_subdivision_idx
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();
```

### Cleanup File

**Path**: `business-core/business-core-postgres/cleanup/003_cleanup_person_country_subdivision.sql`

```sql
-- Cleanup: Initial CountrySubdivision Schema
-- Description: Removes all artifacts created by 003_initial_schema_person_country_subdivision.sql

-- Drop trigger first
DROP TRIGGER IF EXISTS country_subdivision_idx_notify ON country_subdivision_idx;

-- Drop tables (country_subdivision_idx first due to foreign key constraint)
DROP TABLE IF EXISTS country_subdivision_idx CASCADE;
DROP TABLE IF EXISTS country_subdivision CASCADE;
```

---

## Generation Instructions

Using the entity template skill at `business-core/docs/skills/entity_template`, generate the following artifacts:

### 1. Model File

**Path**: `business-core/business-core-db/src/models/person/country_subdivision.rs`

**Requirements**:
- `CountrySubdivisionModel` struct with all fields
- `CountrySubdivisionIdxModel` struct with `id`, `country_id`, and `code_hash`
- Implement `Identifiable` trait for both models
- Implement `IndexAware` trait for `CountrySubdivisionModel`
- Implement `Index` trait for `CountrySubdivisionIdxModel`
- Implement `HasPrimaryKey` trait for `CountrySubdivisionIdxModel`
- Implement `Indexable` trait for `CountrySubdivisionIdxModel` with:
  - `i64_keys()`: return `code_hash`
  - `uuid_keys()`: return `country_id`
- Define `CountrySubdivisionIdxModelCache` type alias

**Index Computation in `to_index()`**:
```rust
fn to_index(&self) -> Self::IndexType {
    let code_hash = hash_as_i64(&self.code.as_str());
    
    CountrySubdivisionIdxModel {
        id: self.id,
        country_id: self.country_id,
        code_hash,
    }
}
```

### 2. Repository Module

**Path**: `business-core/business-core-postgres/src/repository/person/country_subdivision_repository/`

**Module Structure**:
```
country_subdivision_repository/
├── mod.rs
├── repo_impl.rs
├── create_batch.rs
├── load_batch.rs
├── update_batch.rs
├── delete_batch.rs
├── exist_by_ids.rs
├── find_ids_by_code_hash.rs
└── find_ids_by_country_id.rs
```

#### mod.rs
```rust
pub mod repo_impl;
pub mod create_batch;
pub mod delete_batch;
pub mod exist_by_ids;
pub mod load_batch;
pub mod update_batch;
pub mod find_ids_by_code_hash;
pub mod find_ids_by_country_id;

pub use repo_impl::CountrySubdivisionRepositoryImpl;
```

#### repo_impl.rs

**Requirements**:
- `CountrySubdivisionRepositoryImpl` struct with `executor` and `country_subdivision_idx_cache`
- Constructor `new(executor, country_subdivision_idx_cache)`
- `load_all_country_subdivision_idx()` method for cache initialization
- `TryFromRow` implementation for `CountrySubdivisionModel`:
  - Use `get_heapless_string(row, "code")?`
  - Use `get_heapless_string(row, "name_l1")?`
  - Use `get_optional_heapless_string(row, "name_l2")?`
  - Use `get_optional_heapless_string(row, "name_l3")?`
- `TryFromRow` implementation for `CountrySubdivisionIdxModel`

#### create_batch.rs

**SQL Insert Statements**:
```sql
-- Main table insert
INSERT INTO country_subdivision (id, country_id, code, name_l1, name_l2, name_l3)
VALUES ($1, $2, $3, $4, $5, $6)

-- Index table insert
INSERT INTO country_subdivision_idx (id, country_id, code_hash)
VALUES ($1, $2, $3)
```

**Test Cases**:
1. `test_create_batch()` - Create 5 subdivisions
2. `test_create_batch_empty()` - Handle empty batch

#### load_batch.rs

**SQL Query**:
```sql
SELECT * FROM country_subdivision WHERE id = ANY($1)
```

**Test Cases**:
1. `test_load_batch()` - Load multiple subdivisions
2. `test_load_batch_with_non_existing()` - Handle mix of existing/non-existing IDs

#### update_batch.rs

**SQL Update Statement**:
```sql
UPDATE country_subdivision
SET country_id = $2, code = $3, name_l1 = $4, name_l2 = $5, name_l3 = $6
WHERE id = $1
```

**Cache Update**: Remove old index, add new index

**Test Cases**:
1. `test_update_batch()` - Update multiple subdivisions
2. `test_update_batch_empty()` - Handle empty batch

#### delete_batch.rs

**SQL Delete Statements**:
```sql
-- Delete index first
DELETE FROM country_subdivision_idx WHERE id = ANY($1)

-- Delete main table
DELETE FROM country_subdivision WHERE id = ANY($1)
```

**Test Cases**:
1. `test_delete_batch()` - Delete multiple subdivisions
2. `test_delete_batch_with_non_existing()` - Handle non-existing IDs

#### exist_by_ids.rs

**Implementation**: Check cache using `cache.contains_primary(&id)`

**Test Cases**:
1. `test_exist_by_ids()` - Check mix of existing/non-existing IDs

#### find_ids_by_code_hash.rs

**Implementation**:
```rust
pub async fn find_ids_by_code_hash(
    &self,
    code_hash: i64,
) -> Result<Vec<Uuid>, Box<dyn Error + Send + Sync>> {
    let cache = self.country_subdivision_idx_cache.read();
    let result = cache
        .get_by_i64_index("code_hash", &code_hash)
        .cloned()
        .unwrap_or_default();
    Ok(result)
}
```

**Test Cases**:
1. `test_find_ids_by_code_hash()` - Find by unique code
2. `test_find_ids_by_code_hash_non_existing()` - Handle non-existing code

#### find_ids_by_country_id.rs

**Implementation**:
```rust
pub async fn find_ids_by_country_id(
    &self,
    country_id: Uuid,
) -> Result<Vec<Uuid>, Box<dyn Error + Send + Sync>> {
    let cache = self.country_subdivision_idx_cache.read();
    let result = cache
        .get_by_uuid_index("country_id", &country_id)
        .cloned()
        .unwrap_or_default();
    Ok(result)
}
```

**Test Cases**:
1. `test_find_ids_by_country_id()` - Find all subdivisions for a country
2. `test_find_ids_by_country_id_non_existing()` - Handle country with no subdivisions

---

## Module Registration

### Update `business-core/business-core-db/src/models/person/mod.rs`

Add:
```rust
pub mod country_subdivision;
pub use country_subdivision::*;
```

### Update `business-core/business-core-postgres/src/repository/person/mod.rs`

Add:
```rust
pub mod country_subdivision_repository;
pub use country_subdivision_repository::CountrySubdivisionRepositoryImpl;
```

---

## Test Helper Function

```rust
fn create_test_country_subdivision(
    country_id: Uuid,
    code: &str,
    name: &str,
) -> CountrySubdivisionModel {
    CountrySubdivisionModel {
        id: Uuid::new_v4(),
        country_id,
        code: HeaplessString::try_from(code).unwrap(),
        name_l1: HeaplessString::try_from(name).unwrap(),
        name_l2: None,
        name_l3: None,
    }
}
```

---

## Validation Checklist

After generation, verify:

- [ ] `CountrySubdivisionModel` has all 6 fields with correct types
- [ ] `CountrySubdivisionIdxModel` has `id`, `country_id`, and `code_hash`
- [ ] All trait implementations are present and correct
- [ ] `to_index()` computes `code_hash` using `hash_as_i64`
- [ ] `Indexable` returns both i64 and UUID keys correctly
- [ ] Repository has all CRUD operations (create, load, update, delete, exist)
- [ ] Two custom query methods are implemented (find_ids_by_code_hash, find_ids_by_country_id)
- [ ] Database migration file created with proper schema
- [ ] Database cleanup file created for rollback
- [ ] Foreign key constraint on country_id
- [ ] Unique constraint on code field
- [ ] Database trigger for cache notifications
- [ ] Transaction lock pattern is followed in all operations
- [ ] Cache is updated after database operations
- [ ] All tests compile and include edge cases
- [ ] Module registration is complete
- [ ] Code follows the Country entity pattern exactly

---

## Notes

- **Unique Constraint**: The `code` field should be unique per subdivision. The `code_hash` index enforces this at the application layer.
- **Foreign Key**: `country_id` references `CountryModel.id`. Consider adding foreign key constraint in database schema.
- **Cache Indexes**: Two indexes are maintained:
  - `code_hash` (i64) for unique code lookups
  - `country_id` (UUID) for finding all subdivisions of a country
- **Composite Queries**: For operations that need to combine index lookup with batch loading, implement at the service/application layer:
  ```rust
  let ids = repo.find_ids_by_country_id(country_id).await?;
  let results = repo.load_batch(&ids).await?;
  Ok(results.into_iter().flatten().collect())
  ```

---

## Reference

- **Skill Documentation**: `business-core/docs/skills/entity_template/README.md`
- **Template Model**: `business-core/business-core-db/src/models/person/country.rs`
- **Template Repository**: `business-core/business-core-postgres/src/repository/person/country_repository/`
- **Source Reference**: `ledger-banking-rust/banking-db/src/models/person/country_subdivision.rs`