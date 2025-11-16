# Generate Locality Entity

Apply the entity template skill from `business-core/docs/skills/entity_template` to generate a complete database access module for the Locality entity in business-core.

## Entity Specification

### Entity Definition

```yaml
entity:
  name: "Locality"
  module: "person"
  table_name: "locality"
  idx_table_name: "locality_idx"
```

### Fields Specification

```yaml
fields:
  - name: "id"
    type: "Uuid"
    nullable: false
    primary_key: true
    description: "Primary identifier for locality"
    
  - name: "country_subdivision_id"
    type: "Uuid"
    nullable: false
    indexed: true
    description: "Foreign key to CountrySubdivisionModel.id"
    constraint: "exists(CountrySubdivisionModel.id)"
    
  - name: "code"
    type: "HeaplessString<50>"
    nullable: false
    indexed: true
    unique: true
    description: "Locality code - if non-existent, use country subdivision code '_' the first 10 chars of the name_l1"
    
  - name: "name_l1"
    type: "HeaplessString<50>"
    nullable: false
    description: "Primary language name"
    
  - name: "name_l2"
    type: "HeaplessString<50>"
    nullable: true
    description: "Secondary language name (optional)"
    
  - name: "name_l3"
    type: "HeaplessString<50>"
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
      description: "Hash-based unique index for locality code"
  
  uuid_keys:
    - field: "country_subdivision_id"
      index_name: "country_subdivision_id"
      description: "Foreign key index to country subdivision"
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
    description: "Find locality IDs by code hash (should return single result due to uniqueness)"
    
  - name: "find_ids_by_country_subdivision_id"
    parameters:
      - name: "country_subdivision_id"
        type: "Uuid"
    return_type: "Vec<Uuid>"
    cache_based: true
    description: "Find all locality IDs for a given country subdivision"
```

---

## Database Schema Files

### Migration File

**Path**: `business-core/business-core-postgres/migrations/004_initial_schema_person_locality.sql`

```sql
-- Migration: Initial Locality Schema
-- Description: Creates locality-related tables and indexes

-- Locality Table
CREATE TABLE IF NOT EXISTS locality (
    id UUID PRIMARY KEY,
    country_subdivision_id UUID NOT NULL REFERENCES country_subdivision(id) ON DELETE CASCADE,
    code VARCHAR(50) NOT NULL UNIQUE,
    name_l1 VARCHAR(50) NOT NULL,
    name_l2 VARCHAR(50),
    name_l3 VARCHAR(50)
);

-- Locality Index Table
CREATE TABLE IF NOT EXISTS locality_idx (
    locality_id UUID PRIMARY KEY REFERENCES locality(id) ON DELETE CASCADE,
    country_subdivision_id UUID NOT NULL,
    code_hash BIGINT NOT NULL UNIQUE
);

-- Create trigger for locality_idx table to notify listeners of changes
DROP TRIGGER IF EXISTS locality_idx_notify ON locality_idx;
CREATE TRIGGER locality_idx_notify
    AFTER INSERT OR UPDATE OR DELETE ON locality_idx
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();
```

### Cleanup File

**Path**: `business-core/business-core-postgres/cleanup/004_cleanup_person_locality.sql`

```sql
-- Cleanup: Initial Locality Schema
-- Description: Removes all artifacts created by 004_initial_schema_person_locality.sql

-- Drop trigger first
DROP TRIGGER IF EXISTS locality_idx_notify ON locality_idx;

-- Drop tables (locality_idx first due to foreign key constraint)
DROP TABLE IF EXISTS locality_idx CASCADE;
DROP TABLE IF EXISTS locality CASCADE;
```

---

## Generation Instructions

Using the entity template skill at `business-core/docs/skills/entity_template`, generate the following artifacts:

### 1. Model File

**Path**: `business-core/business-core-db/src/models/person/locality.rs`

**Requirements**:
- `LocalityModel` struct with all fields
- `LocalityIdxModel` struct with `locality_id`, `country_subdivision_id`, and `code_hash`
- Implement `Identifiable` trait for both models
- Implement `IndexAware` trait for `LocalityModel`
- Implement `Index` trait for `LocalityIdxModel`
- Implement `HasPrimaryKey` trait for `LocalityIdxModel`
- Implement `Indexable` trait for `LocalityIdxModel` with:
  - `i64_keys()`: return `code_hash`
  - `uuid_keys()`: return `country_subdivision_id`
- Define `LocalityIdxModelCache` type alias

**Index Computation in `to_index()`**:
```rust
fn to_index(&self) -> Self::IndexType {
    let code_hash = hash_as_i64(&self.code.as_str());
    
    LocalityIdxModel {
        locality_id: self.id,
        country_subdivision_id: self.country_subdivision_id,
        code_hash,
    }
}
```

### 2. Repository Module

**Path**: `business-core/business-core-postgres/src/repository/person/locality_repository/`

**Module Structure**:
```
locality_repository/
├── mod.rs
├── repo_impl.rs
├── create_batch.rs
├── load_batch.rs
├── update_batch.rs
├── delete_batch.rs
├── exist_by_ids.rs
├── find_ids_by_code_hash.rs
└── find_ids_by_country_subdivision_id.rs
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
pub mod find_ids_by_country_subdivision_id;

pub use repo_impl::LocalityRepositoryImpl;
```

#### repo_impl.rs

**Requirements**:
- `LocalityRepositoryImpl` struct with `executor` and `locality_idx_cache`
- Constructor `new(executor, locality_idx_cache)`
- `load_all_locality_idx()` method for cache initialization
- `TryFromRow` implementation for `LocalityModel`:
  - Use `get_heapless_string(row, "code")?`
  - Use `get_heapless_string(row, "name_l1")?`
  - Use `get_optional_heapless_string(row, "name_l2")?`
  - Use `get_optional_heapless_string(row, "name_l3")?`
- `TryFromRow` implementation for `LocalityIdxModel`
- Implement `TransactionAware` trait with `on_commit()` and `on_rollback()`

#### create_batch.rs

**SQL Insert Statements**:
```sql
-- Main table insert
INSERT INTO locality (id, country_subdivision_id, code, name_l1, name_l2, name_l3)
VALUES ($1, $2, $3, $4, $5, $6)

-- Index table insert
INSERT INTO locality_idx (locality_id, country_subdivision_id, code_hash)
VALUES ($1, $2, $3)
```

**Test Cases**:
1. `test_create_batch()` - Create 5 localities
2. `test_create_batch_empty()` - Handle empty batch
3. `test_locality_insert_triggers_cache_notification()` - Verify cache notification

#### load_batch.rs

**SQL Query**:
```sql
SELECT * FROM locality WHERE id = ANY($1)
```

**Test Cases**:
1. `test_load_batch()` - Load multiple localities
2. `test_load_batch_with_non_existing()` - Handle mix of existing/non-existing IDs

#### update_batch.rs

**SQL Update Statement**:
```sql
UPDATE locality
SET country_subdivision_id = $2, code = $3, name_l1 = $4, name_l2 = $5, name_l3 = $6
WHERE id = $1
```

**Cache Update**: Remove old index, add new index

**Test Cases**:
1. `test_update_batch()` - Update multiple localities
2. `test_update_batch_empty()` - Handle empty batch

#### delete_batch.rs

**SQL Delete Statements**:
```sql
-- Delete index first
DELETE FROM locality_idx WHERE locality_id = ANY($1)

-- Delete main table
DELETE FROM locality WHERE id = ANY($1)
```

**Test Cases**:
1. `test_delete_batch()` - Delete multiple localities
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
    let cache = self.locality_idx_cache.read().await;
    let items = cache.get_by_i64_index("code_hash", &code_hash);
    let result = items.into_iter().map(|item| item.locality_id).collect();
    Ok(result)
}
```

**Test Cases**:
1. `test_find_ids_by_code_hash()` - Find by unique code
2. `test_find_ids_by_code_hash_non_existing()` - Handle non-existing code

#### find_ids_by_country_subdivision_id.rs

**Implementation**:
```rust
pub async fn find_ids_by_country_subdivision_id(
    &self,
    country_subdivision_id: Uuid,
) -> Result<Vec<Uuid>, Box<dyn Error + Send + Sync>> {
    let cache = self.locality_idx_cache.read().await;
    let items = cache.get_by_uuid_index("country_subdivision_id", &country_subdivision_id);
    let result = items.into_iter().map(|item| item.locality_id).collect();
    Ok(result)
}
```

**Test Cases**:
1. `test_find_ids_by_country_subdivision_id()` - Find all localities for a country subdivision
2. `test_find_ids_by_country_subdivision_id_non_existing()` - Handle subdivision with no localities

---

## Module Registration

### Update `business-core/business-core-db/src/models/person/mod.rs`

Add:
```rust
pub mod locality;
pub use locality::*;
```

### Update `business-core/business-core-postgres/src/repository/person/mod.rs`

Add:
```rust
pub mod locality_repository;
pub use locality_repository::LocalityRepositoryImpl;
```

### Update PersonRepoFactory

**Path**: `business-core/business-core-postgres/src/repository/person/factory.rs`

Add to the factory:

1. **Add cache field**:
```rust
locality_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<LocalityIdxModel>>>,
```

2. **Initialize cache in `new()`**:
```rust
let locality_idx_cache = Arc::new(ParkingRwLock::new(
    business_core_db::IdxModelCache::new(vec![]).unwrap()
));
```

3. **Register handler in `new()`**:
```rust
if let Some(listener) = listener {
    // ... existing handlers
    let locality_handler = Arc::new(IndexCacheHandler::new(
        "locality_idx".to_string(),
        locality_idx_cache.clone(),
    ));
    listener.register_handler(locality_handler);
}
```

4. **Add build method**:
```rust
pub fn build_locality_repo(&self, session: &impl UnitOfWorkSession) -> Arc<LocalityRepositoryImpl> {
    let repo = Arc::new(LocalityRepositoryImpl::new(
        session.executor().clone(),
        self.locality_idx_cache.clone(),
    ));
    session.register_transaction_aware(repo.clone());
    repo
}
```

5. **Add to PersonRepositories struct**:
```rust
pub struct PersonRepositories {
    pub country_repository: Arc<CountryRepositoryImpl>,
    pub country_subdivision_repository: Arc<CountrySubdivisionRepositoryImpl>,
    pub locality_repository: Arc<LocalityRepositoryImpl>,
}
```

6. **Update `build_all_repos()`**:
```rust
pub fn build_all_repos(&self, session: &impl UnitOfWorkSession) -> PersonRepositories {
    PersonRepositories {
        country_repository: self.build_country_repo(session),
        country_subdivision_repository: self.build_country_subdivision_repo(session),
        locality_repository: self.build_locality_repo(session),
    }
}
```

---

## Test Helper Function

**Location**: `business-core/business-core-postgres/src/repository/person/locality_repository/mod.rs`

```rust
#[cfg(test)]
pub mod test_utils {
    use business_core_db::models::person::country::CountryModel;
    use business_core_db::models::person::country_subdivision::CountrySubdivisionModel;
    use business_core_db::models::person::locality::LocalityModel;
    use heapless::String as HeaplessString;
    use uuid::Uuid;

    pub fn create_test_country(iso2: &str, name: &str) -> CountryModel {
        CountryModel {
            id: Uuid::new_v4(),
            iso2: HeaplessString::try_from(iso2).unwrap(),
            name_l1: HeaplessString::try_from(name).unwrap(),
            name_l2: None,
            name_l3: None,
        }
    }

    pub fn create_test_country_subdivision(
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

    pub fn create_test_locality(
        country_subdivision_id: Uuid,
        code: &str,
        name: &str,
    ) -> LocalityModel {
        LocalityModel {
            id: Uuid::new_v4(),
            country_subdivision_id,
            code: HeaplessString::try_from(code).unwrap(),
            name_l1: HeaplessString::try_from(name).unwrap(),
            name_l2: None,
            name_l3: None,
        }
    }
}
```

---

## Validation Checklist

After generation, verify:

- [ ] `LocalityModel` has all 6 fields with correct types
- [ ] `LocalityIdxModel` has `locality_id`, `country_subdivision_id`, and `code_hash`
- [ ] All trait implementations are present and correct
- [ ] `to_index()` computes `code_hash` using `hash_as_i64`
- [ ] `Indexable` returns both i64 and UUID keys correctly
- [ ] `HasPrimaryKey` returns `locality_id` (not `id`)
- [ ] Repository has all CRUD operations (create, load, update, delete, exist)
- [ ] Two custom query methods are implemented (find_ids_by_code_hash, find_ids_by_country_subdivision_id)
- [ ] `TransactionAware` trait is implemented with `on_commit()` and `on_rollback()`
- [ ] Database migration file created with proper schema
- [ ] Database cleanup file created for rollback
- [ ] Foreign key constraint on country_subdivision_id
- [ ] Unique constraint on code field
- [ ] Database trigger for cache notifications
- [ ] Transaction lock pattern is followed in all operations
- [ ] Cache is updated after database operations using read lock
- [ ] Cache notification test is included
- [ ] All tests compile and include edge cases
- [ ] Module registration is complete
- [ ] PersonRepoFactory is updated with locality repository
- [ ] Code follows the CountrySubdivision entity pattern exactly

---

## Notes

- **Primary Key Field Name**: The `LocalityIdxModel` uses `locality_id` as the primary key field name (not just `id`), following the pattern from the source model.
- **Unique Constraint**: The `code` field should be unique per locality. The `code_hash` index enforces this at the application layer.
- **Foreign Key**: `country_subdivision_id` references `CountrySubdivisionModel.id`. The database schema includes a foreign key constraint.
- **Cache Indexes**: Two indexes are maintained:
  - `code_hash` (i64) for unique code lookups
  - `country_subdivision_id` (UUID) for finding all localities of a country subdivision
- **Composite Queries**: For operations that need to combine index lookup with batch loading, implement at the service/application layer:
  ```rust
  let ids = repo.find_ids_by_country_subdivision_id(subdivision_id).await?;
  let results = repo.load_batch(&ids).await?;
  Ok(results.into_iter().flatten().collect())
  ```
- **Test Dependencies**: When testing, remember to create parent records (Country and CountrySubdivision) before creating Locality records to satisfy foreign key constraints.

---

## Reference

- **Skill Documentation**: `business-core/docs/skills/entity_template/README.md`
- **Template Model**: `business-core/business-core-db/src/models/person/country.rs`
- **Template Repository**: `business-core/business-core-postgres/src/repository/person/country_repository/`
- **Similar Entity**: `business-core/business-core-db/src/models/person/country_subdivision.rs`
- **Source Reference**: `ledger-banking-rust/banking-db/src/models/person/locality.rs`