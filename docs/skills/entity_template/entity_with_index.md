# Entity Template Skill

## Overview

This skill generates complete database access modules for entities in the business-core architecture. It creates a consistent pattern of models, repositories, cache integration, and tests based on the Country entity template.

## Purpose

Generate production-ready database access layers with:
- ✅ **CRUD operations** (Create, Read, Update, Delete)
- ✅ **Batch operations** for performance
- ✅ **Application-layer indexing** with hash-based lookups
- ✅ **Transaction-aware cache integration** with automatic rollback support
- ✅ **Comprehensive testing** (unit + integration)
- ✅ **Audit logging** support

## Template Reference

The skill is based on the Country entity pattern:
- **Model**: `business-core/business-core-db/src/models/person/country.rs`
- **Repository**: `business-core/business-core-postgres/src/repository/person/country_repository/`

---

## Input Parameters

When invoking this skill, provide the following parameters:

### 1. Entity Definition

```yaml
entity:
  name: "Country"              # Entity name (PascalCase)
  module: "person"             # Module name (snake_case)
  table_name: "country"        # Database table name
  idx_table_name: "country_idx" # Index table name
```

### 2. Fields Specification

```yaml
fields:
  - name: "id"
    type: "Uuid"
    nullable: false
    primary_key: true
    
  - name: "iso2"
    type: "HeaplessString<2>"
    nullable: false
    indexed: true              # Will create hash index
    
  - name: "name_l1"
    type: "HeaplessString<100>"
    nullable: false
    
  - name: "name_l2"
    type: "HeaplessString<100>"
    nullable: true
    
  - name: "name_l3"
    type: "HeaplessString<100>"
    nullable: true
```

### 3. Index Keys

```yaml
index_keys:
  i64_keys:
    - field: "iso2"
      index_name: "iso2_hash"
      hash_function: "hash_as_i64"
  
  uuid_keys: []  # Optional UUID-based indexes
```

### 4. Custom Query Methods

```yaml
custom_queries:
  - name: "find_ids_by_iso2_hash"
    parameters:
      - name: "iso2_hash"
        type: "i64"
    return_type: "Vec<Uuid>"
    cache_based: true
```

---

## Generated Artifacts

### 1. Model File Structure

**Location**: `business-core/business-core-db/src/models/{module}/{entity}.rs`

```rust
use heapless::String as HeaplessString;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use std::collections::HashMap;
use crate::{HasPrimaryKey, IdxModelCache, Indexable};
use crate::models::{IndexAware, Identifiable, Index};
use crate::utils::hash_as_i64;

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
    // ... fields from specification
    // pub enum_field: {EnumName},
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct {Entity}IdxModel {
    pub id: Uuid,
    // ... index keys from specification
}

// Trait implementations
impl HasPrimaryKey for {Entity}IdxModel {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}

impl Identifiable for {Entity}Model {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl IndexAware for {Entity}Model {
    type IndexType = {Entity}IdxModel;
    
    fn to_index(&self) -> Self::IndexType {
        // Calculate hashes for indexed fields
        {Entity}IdxModel {
            id: self.id,
            // ... computed index fields
        }
    }
}

impl Identifiable for {Entity}IdxModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Index for {Entity}IdxModel {}

impl Indexable for {Entity}IdxModel {
    fn i64_keys(&self) -> HashMap<String, Option<i64>> {
        let mut keys = HashMap::new();
        // ... i64 index mappings
        keys
    }

    fn uuid_keys(&self) -> HashMap<String, Option<Uuid>> {
        HashMap::new() // or UUID index mappings
    }
}

pub type {Entity}IdxModelCache = IdxModelCache<{Entity}IdxModel>;
```

### 2. Repository Implementation

**Location**: `business-core/business-core-postgres/src/repository/{module}/{entity}_repository/`

#### Module Structure

```
{entity}_repository/
├── mod.rs
├── repo_impl.rs
├── create_batch.rs
├── load_batch.rs
├── update_batch.rs
├── delete_batch.rs
├── exist_by_ids.rs
└── {custom_query_methods}.rs
```

#### repo_impl.rs Pattern

```rust
use business_core_db::models::{module}::{entity}::{Entity}IdxModel, {Entity}Model};
use crate::utils::{get_heapless_string, get_optional_heapless_string, TryFromRow};
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use postgres_index_cache::TransactionAwareIdxModelCache;
use parking_lot::RwLock as ParkingRwLock;
use tokio::sync::RwLock;
use std::sync::Arc;
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use async_trait::async_trait;

pub struct {Entity}RepositoryImpl {
    pub executor: Executor,
    pub {entity}_idx_cache: Arc<RwLock<TransactionAwareIdxModelCache<{Entity}IdxModel>>>,
}

impl {Entity}RepositoryImpl {
    pub fn new(
        executor: Executor,
        {entity}_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<{Entity}IdxModel>>>,
    ) -> Self {
        Self {
            executor,
            {entity}_idx_cache: Arc::new(RwLock::new(TransactionAwareIdxModelCache::new(
                {entity}_idx_cache,
            ))),
        }
    }

    pub async fn load_all_{entity}_idx(
        executor: &Executor,
    ) -> Result<Vec<{Entity}IdxModel>, sqlx::Error> {
        let query = sqlx::query("SELECT * FROM {table_name}_idx");
        let rows = {
            let mut tx = executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
            query.fetch_all(&mut **transaction).await?
        };
        
        let mut idx_models = Vec::with_capacity(rows.len());
        for row in rows {
            idx_models.push({Entity}IdxModel::try_from_row(&row).map_err(sqlx::Error::Decode)?);
        }
        Ok(idx_models)
    }
}

impl TryFromRow<PgRow> for {Entity}Model {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok({Entity}Model {
            id: row.get("id"),
            // ... field mappings with appropriate helper functions
        })
    }
}

impl TryFromRow<PgRow> for {Entity}IdxModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok({Entity}IdxModel {
            id: row.get("{entity}_id"),
            // ... index field mappings
        })
    }
}

#[async_trait]
impl TransactionAware for {Entity}RepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        self.{entity}_idx_cache.read().await.on_commit().await
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        self.{entity}_idx_cache.read().await.on_rollback().await
    }
}
```

#### create_batch.rs Pattern

```rust
use async_trait::async_trait;
use business_core_db::models::{module}::{entity}::{Entity}Model;
use business_core_db::repository::create_batch::CreateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;
use business_core_db::models::index_aware::IndexAware;

use super::repo_impl::{Entity}RepositoryImpl;

impl {Entity}RepositoryImpl {
    pub(super) async fn create_batch_impl(
        repo: &{Entity}RepositoryImpl,
        items: Vec<{Entity}Model>,
    ) -> Result<Vec<{Entity}Model>, Box<dyn Error + Send + Sync>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut saved_items = Vec::new();
        let mut indices = Vec::new();
        
        // Acquire lock once and do all database operations
        let mut tx = repo.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        
        for item in items {
            // Execute main insert
            sqlx::query(
                r#"
                INSERT INTO {table_name} ({field_list})
                VALUES ({value_placeholders})
                "#,
            )
            // ... bind all fields
            .execute(&mut **transaction)
            .await?;
            // Insert into index table
            let idx = item.to_index();
            sqlx::query(
                r#"
                INSERT INTO {table_name}_idx ({index_field_list})
                VALUES ({index_value_placeholders})
                "#,
            )
            // ... bind index fields
            .execute(&mut **transaction)
            .await?;
            indices.push(idx);
            saved_items.push(item);
        }
        
        // Update cache after releasing transaction lock
        {
            let cache = repo.{entity}_idx_cache.read().await;
            for idx in indices {
                cache.add(idx);
            }
        }

        Ok(saved_items)
    }
}

#[async_trait]
impl CreateBatch<Postgres, {Entity}Model> for {Entity}RepositoryImpl {
    async fn create_batch(
        &self,
        items: Vec<{Entity}Model>,
        _audit_log_id: Option<Uuid>,
    ) -> Result<Vec<{Entity}Model>, Box<dyn Error + Send + Sync>> {
        Self::create_batch_impl(self, items).await
    }
}

#[cfg(test)]
mod tests {
    // ... comprehensive tests
}
```

#### load_batch.rs Pattern

```rust
use async_trait::async_trait;
use business_core_db::models::{module}::{entity}::{Entity}Model;
use business_core_db::repository::load_batch::LoadBatch;
use crate::utils::TryFromRow;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::{Entity}RepositoryImpl;

impl {Entity}RepositoryImpl {
    pub(super) async fn load_batch_impl(
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
}

#[async_trait]
impl LoadBatch<Postgres, {Entity}Model> for {Entity}RepositoryImpl {
    async fn load_batch(&self, ids: &[Uuid]) -> Result<Vec<Option<{Entity}Model>>, Box<dyn Error + Send + Sync>> {
        Self::load_batch_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    // ... tests for load_batch
}
```

#### update_batch.rs Pattern

```rust
use async_trait::async_trait;
use business_core_db::models::{module}::{entity}::{Entity}Model;
use business_core_db::models::index_aware::IndexAware;
use business_core_db::repository::update_batch::UpdateBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::{Entity}RepositoryImpl;

impl {Entity}RepositoryImpl {
    pub(super) async fn update_batch_impl(
        &self,
        items: Vec<{Entity}Model>,
    ) -> Result<Vec<{Entity}Model>, Box<dyn Error + Send + Sync>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut updated_items = Vec::new();
        let mut indices = Vec::new();
        
        // Acquire lock once and do all database operations
        let mut tx = self.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        
        for item in items {
            // Execute update
            sqlx::query(
                r#"
                UPDATE {table_name}
                SET {field_updates}
                WHERE id = $1
                "#,
            )
            // ... bind fields
            .execute(&mut **transaction)
            .await?;
            indices.push((item.id, item.to_index()));
            updated_items.push(item);
        }
        
        // Update cache after releasing transaction lock
        {
            let cache = self.{entity}_idx_cache.read().await;
            for (id, idx) in indices {
                cache.remove(&id);
                cache.add(idx);
            }
        }

        Ok(updated_items)
    }
}

#[async_trait]
impl UpdateBatch<Postgres, {Entity}Model> for {Entity}RepositoryImpl {
    async fn update_batch(
        &self,
        items: Vec<{Entity}Model>,
        _audit_log_id: Option<Uuid>,
    ) -> Result<Vec<{Entity}Model>, Box<dyn Error + Send + Sync>> {
        Self::update_batch_impl(self, items).await
    }
}

#[cfg(test)]
mod tests {
    // ... tests for update_batch
}
```

#### delete_batch.rs Pattern

```rust
use async_trait::async_trait;
use business_core_db::repository::delete_batch::DeleteBatch;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::{Entity}RepositoryImpl;

impl {Entity}RepositoryImpl {
    pub(super) async fn delete_batch_impl(
        repo: &{Entity}RepositoryImpl,
        ids: &[Uuid],
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(0);
        }

        // Delete from index table first
        let delete_idx_query = r#"DELETE FROM {table_name}_idx WHERE id = ANY($1)"#;
        let delete_query = r#"DELETE FROM {table_name} WHERE id = ANY($1)"#;

        let mut tx = repo.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        
        sqlx::query(delete_idx_query)
            .bind(ids)
            .execute(&mut **transaction)
            .await?;
        let result = sqlx::query(delete_query)
            .bind(ids)
            .execute(&mut **transaction)
            .await?;
        let rows_affected = result.rows_affected() as usize;
        
        // Update cache after releasing transaction lock
        {
            let cache = repo.{entity}_idx_cache.read().await;
            for id in ids {
                cache.remove(id);
            }
        }
        
        Ok(rows_affected)
    }
}

#[async_trait]
impl DeleteBatch<Postgres> for {Entity}RepositoryImpl {
    async fn delete_batch(
        &self,
        ids: &[Uuid],
        _audit_log_id: Option<Uuid>,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        Self::delete_batch_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    // ... tests for delete_batch
}
```

#### exist_by_ids.rs Pattern

```rust
use async_trait::async_trait;
use business_core_db::repository::exist_by_ids::ExistByIds;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::{Entity}RepositoryImpl;

impl {Entity}RepositoryImpl {
    pub(super) async fn exist_by_ids_impl(
        repo: &{Entity}RepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        let mut result = Vec::new();
        let cache = repo.{entity}_idx_cache.read().await;
        for &id in ids {
            result.push((id, cache.contains_primary(&id)));
        }
        Ok(result)
    }
}

#[async_trait]
impl ExistByIds<Postgres> for {Entity}RepositoryImpl {
    async fn exist_by_ids(&self, ids: &[Uuid]) -> Result<Vec<(Uuid, bool)>, Box<dyn Error + Send + Sync>> {
        Self::exist_by_ids_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    // ... tests for exist_by_ids
}
```

#### Custom Query Methods Pattern

For cache-based lookups (e.g., `find_ids_by_iso2_hash.rs`):

```rust
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::{Entity}RepositoryImpl;

impl {Entity}RepositoryImpl {
    pub async fn find_ids_by_{index_name}(
        &self,
        {index_name}: i64,
    ) -> Result<Vec<Uuid>, Box<dyn Error + Send + Sync>> {
        let cache = self.{entity}_idx_cache.read().await;
        let items = cache.get_by_i64_index("{index_name}", &{index_name});
        let result = items.into_iter().map(|item| item.id).collect();
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    // ... tests for custom query method
}
```

---

## Critical Patterns

### 1. Transaction-Aware Cache Architecture

The cache implementation uses a two-tier architecture:

1. **Shared Cache**: `Arc<parking_lot::RwLock<IdxModelCache<T>>>` - Global, persistent cache
2. **Transaction Layer**: `Arc<tokio::sync::RwLock<TransactionAwareIdxModelCache<T>>>` - Transaction-local staging

**Key Concepts:**

- **Staging**: Cache modifications are staged locally during a transaction
- **Commit**: On successful commit, staged changes are applied to the shared cache via `on_commit()`
- **Rollback**: On rollback, staged changes are discarded via `on_rollback()`
- **Read-through**: Reads consider both staged changes and the shared cache

### 2. Transaction Lock Management

**ALWAYS follow this pattern:**

```rust
// 1. Acquire transaction lock
let mut tx = self.executor.tx.lock().await;
let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;

// 2. Perform all database operations
// ...

// 3. The lock is released automatically when `tx` goes out of scope.

// 4. Update cache AFTER lock is released (stages changes locally)
{
    let cache = self.{entity}_idx_cache.read().await;
    // ... cache operations (add, remove, update)
}
```

**Why?** Prevents deadlocks and improves concurrency. The transaction-aware cache automatically handles commit/rollback.

### 3. Cache Synchronization

**Pattern for each operation (uses read lock - changes are staged):**

- **Create**: `cache.add(idx)` - Stages item for addition
- **Update**: `cache.remove(&id); cache.add(idx)` - Stages removal and re-addition
- **Delete**: `cache.remove(&id)` - Stages item for removal
- **Read**: Use `cache.contains_primary()` or `cache.get_by_*_index()` - Considers staged changes

**Transaction Lifecycle:**

1. During transaction: All cache operations stage changes locally
2. On commit: `on_commit()` applies all staged changes to shared cache
3. On rollback: `on_rollback()` discards all staged changes

**Important**: Always use `.await` when accessing the transaction-aware cache:
```rust
let cache = self.{entity}_idx_cache.read().await;  // Note the .await
```

### 4. Error Handling

```rust
// Use Box<dyn Error + Send + Sync> for all repository methods
Result<T, Box<dyn Error + Send + Sync>>

// Check for transaction consumption
let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
```

### 5. TransactionAware Implementation

All repository implementations must implement the `TransactionAware` trait:

```rust
#[async_trait]
impl TransactionAware for {Entity}RepositoryImpl {
    async fn on_commit(&self) -> TransactionResult<()> {
        self.{entity}_idx_cache.read().await.on_commit().await
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        self.{entity}_idx_cache.read().await.on_rollback().await
    }
}
```

This delegates to the transaction-aware cache, which handles applying or discarding staged changes.

### 6. Field Mapping Helpers

For HeaplessString fields:

```rust
use crate::utils::{get_heapless_string, get_optional_heapless_string};

// Non-nullable
field_name: get_heapless_string(row, "field_name")?,

// Nullable
optional_field: get_optional_heapless_string(row, "optional_field")?,
```

### 7. Index Hash Computation

```rust
use crate::utils::hash_as_i64;

fn to_index(&self) -> Self::IndexType {
    let field_hash = hash_as_i64(&self.field.as_str());
    
    {Entity}IdxModel {
        id: self.id,
        field_hash,
    }
}
```

---

## Testing Patterns

### Test Utility Module

**Location**: `business-core/business-core-postgres/src/repository/{module}/{entity}_repository/test_utils.rs`

Test utilities should be organized in a dedicated module with the `#[cfg(test)]` attribute to ensure they're only compiled during testing:

```rust
#[cfg(test)]
pub mod test_utils {
    use business_core_db::models::{module}::{entity}::{Entity}Model;
    use heapless::String as HeaplessString;
    use uuid::Uuid;

    /// Creates a test {entity} with the specified parameters
    pub fn create_test_{entity}(/* relevant fields */) -> {Entity}Model {
        {Entity}Model {
            id: Uuid::new_v4(),
            // ... initialize fields with HeaplessString::try_from().unwrap()
        }
    }
}
```

**Example from Country entity:**

```rust
#[cfg(test)]
pub mod test_utils {
    use business_core_db::models::person::country::CountryModel;
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
}
```

**Example from CountrySubdivision entity (with foreign key):**

```rust
#[cfg(test)]
pub mod test_utils {
    use business_core_db::models::person::country::CountryModel;
    use business_core_db::models::person::country_subdivision::CountrySubdivisionModel;
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
}
```

### Test Helper Best Practices

1. **Use `#[cfg(test)]`**: Ensures test utilities are only compiled during testing
2. **Create factory functions**: Provide simple constructors that accept only the essential parameters
3. **Use sensible defaults**: Optional fields should default to `None` unless testing requires them
4. **Accept string slices**: Use `&str` parameters and convert to `HeaplessString` internally
5. **Use `.unwrap()`**: Safe in test code since test data is controlled
6. **Include dependency helpers**: For entities with foreign keys, include helper functions to create parent entities
7. **Keep it simple**: Each helper should create a single valid entity with minimal complexity

### Standard Test Cases

For **each operation**, include tests for:

1. **Happy path**: Normal operation succeeds
2. **Empty batch**: Handles empty input gracefully
3. **Non-existent entities**: Handles missing entities correctly
4. **Cache validation**: Verifies cache is updated correctly
5. **Cache notification**: Verifies database triggers update cache correctly (required for entities with in-memory caches)

### Cache Notification Test Pattern

**REQUIRED** for all entities with in-memory caches. This test verifies that database triggers correctly notify and update the cache when index records are inserted or deleted directly via SQL.

```rust
#[cfg(test)]
mod tests {
    use crate::test_helper::{random, setup_test_context, setup_test_context_and_listen};
    use business_core_db::models::index_aware::IndexAware;
    use business_core_db::repository::create_batch::CreateBatch;
    use tokio::time::{sleep, Duration};
    use uuid::Uuid;
    use super::super::test_utils::test_utils::create_test_{entity};

    #[tokio::test]
    async fn test_{entity}_insert_triggers_cache_notification() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        
        // Setup test context with the notification listener
        let ctx = setup_test_context_and_listen().await?;
        let pool = ctx.pool();

        // Create a test entity with unique identifiable fields to avoid conflicts
        let unique_field = random(5);
        let test_entity = create_test_{entity}(&unique_field, "Test Entity");
        let entity_idx = test_entity.to_index();
    
        // Give listener time to start and establish connection
        sleep(Duration::from_millis(2000)).await;
    
        // Insert the entity record directly into database
        sqlx::query("INSERT INTO {table_name} ({field_list}) VALUES ({placeholders})")
            // ... bind all fields
            .execute(&**pool)
            .await
            .expect("Failed to insert {entity}");
    
        // Insert the index record directly into database (triggers notification)
        sqlx::query("INSERT INTO {table_name}_idx ({index_field_list}) VALUES ({index_placeholders})")
            // ... bind index fields
            .execute(&**pool)
            .await
            .expect("Failed to insert {entity} index");

        // Give time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        let entity_repo = &ctx.{module}_repos().{entity}_repository;

        // Verify the cache was updated via the trigger
        let cache = entity_repo.{entity}_idx_cache.read().await;
        assert!(
            cache.contains_primary(&entity_idx.id),
            "{Entity} should be in cache after insert"
        );
    
        let cached_entity = cache.get_by_primary(&entity_idx.id);
        assert!(cached_entity.is_some(), "{Entity} should be retrievable from cache");
        
        // Verify the cached data matches
        let cached_entity = cached_entity.unwrap();
        assert_eq!(cached_entity.id, entity_idx.id);
        // ... assert other index fields
        
        // Drop the read lock before proceeding
        drop(cache);

        // Delete the record from database (triggers notification)
        sqlx::query("DELETE FROM {table_name} WHERE id = $1")
            .bind(entity_idx.id)
            .execute(&**pool)
            .await
            .expect("Failed to delete {entity}");

        // Give time for notification to be processed
        sleep(Duration::from_millis(500)).await;

        // Verify the cache entry was removed
        let cache = entity_repo.{entity}_idx_cache.read().await;
        assert!(
            !cache.contains_primary(&entity_idx.id),
            "{Entity} should be removed from cache after delete"
        );
        
        Ok(())
    }
}
```

**Key aspects of cache notification tests:**

1. **Use `setup_test_context_and_listen()`**: This sets up the PostgreSQL notification listener
2. **Insert via raw SQL**: Direct database inserts trigger the PostgreSQL `NOTIFY` mechanism
3. **Allow processing time**: Use `sleep()` to give the notification handler time to update the cache
4. **Test both INSERT and DELETE**: Verify cache is updated on both operations
5. **Generate unique test data**: Avoid conflicts with existing data or concurrent tests
6. **Verify cache state**: Check both presence and content of cached items
7. **Handle foreign keys**: For entities with foreign keys, insert parent records first

### Example Test Structure

```rust
#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::models::{module}::{entity}::{Entity}Model;
    use business_core_db::repository::create_batch::CreateBatch;
    use uuid::Uuid;

    fn create_test_{entity}(/* params */) -> {Entity}Model {
        // ... implementation
    }

    #[tokio::test]
    async fn test_create_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let {entity}_repo = &ctx.{module}_repos().{entity}_repository;

        let items = vec![/* test data */];
        let audit_log_id = Uuid::new_v4();
        let saved = {entity}_repo.create_batch(items, audit_log_id).await?;

        assert_eq!(saved.len(), /* expected */);
        // ... additional assertions

        Ok(())
    }

    #[tokio::test]
    async fn test_create_batch_empty() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // ... test empty batch
        Ok(())
    }

    #[tokio::test]
    async fn test_{entity}_insert_triggers_cache_notification() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        // ... cache notification test (see pattern above)
        Ok(())
    }

    // ... more tests
}
```

---

## Repository Factory Pattern

### Factory Structure

**Location**: `business-core/business-core-postgres/src/repository/{module}/factory.rs`

Each module requires a factory that manages repository instantiation and cache lifecycle. The factory serves as a singleton that holds all caches for the module and provides methods to build repositories with the appropriate executor.

### Factory Implementation Pattern

```rust
use std::sync::Arc;
use parking_lot::RwLock as ParkingRwLock;
use postgres_unit_of_work::UnitOfWorkSession;
use postgres_index_cache::{CacheNotificationListener, IndexCacheHandler};
use business_core_db::models::{module}::{
    {entity1}::{Entity1}IdxModel,
    {entity2}::{Entity2}IdxModel,
};
use super::{Entity1}RepositoryImpl, {Entity2}RepositoryImpl};

/// Factory for creating {module} module repositories
///
/// This factory holds all caches for the {module} module and provides
/// methods to build repositories with the appropriate executor.
/// This should be used as a singleton throughout the application.
pub struct {Module}RepoFactory {
    {entity1}_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<{Entity1}IdxModel>>>,
    {entity2}_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<{Entity2}IdxModel>>>,
}

impl {Module}RepoFactory {
    /// Create a new {Module}RepoFactory singleton
    ///
    /// Optionally register cache handlers with a notification listener
    pub fn new(listener: Option<&mut CacheNotificationListener>) -> Arc<Self> {
        let {entity1}_idx_cache = Arc::new(ParkingRwLock::new(
            business_core_db::IdxModelCache::new(vec![]).unwrap()
        ));
        
        let {entity2}_idx_cache = Arc::new(ParkingRwLock::new(
            business_core_db::IdxModelCache::new(vec![]).unwrap()
        ));
        
        // Register handlers with listener if provided
        if let Some(listener) = listener {
            let handler = Arc::new(IndexCacheHandler::new(
                "{entity1}_idx".to_string(),
                {entity1}_idx_cache.clone(),
            ));
            listener.register_handler(handler);
            
            let handler2 = Arc::new(IndexCacheHandler::new(
                "{entity2}_idx".to_string(),
                {entity2}_idx_cache.clone(),
            ));
            listener.register_handler(handler2);
        }
        
        Arc::new(Self {
            {entity1}_idx_cache,
            {entity2}_idx_cache,
        })
    }

    /// Build a {Entity1}Repository with the given executor
    pub fn build_{entity1}_repo(&self, session: &impl UnitOfWorkSession) -> Arc<{Entity1}RepositoryImpl> {
        let repo = Arc::new({Entity1}RepositoryImpl::new(
            session.executor().clone(),
            self.{entity1}_idx_cache.clone(),
        ));
        session.register_transaction_aware(repo.clone());
        repo
    }

    /// Build a {Entity2}Repository with the given executor
    pub fn build_{entity2}_repo(&self, session: &impl UnitOfWorkSession) -> Arc<{Entity2}RepositoryImpl> {
        let repo = Arc::new({Entity2}RepositoryImpl::new(
            session.executor().clone(),
            self.{entity2}_idx_cache.clone(),
        ));
        session.register_transaction_aware(repo.clone());
        repo
    }

    /// Build all {module} repositories with the given executor
    pub fn build_all_repos(&self, session: &impl UnitOfWorkSession) -> {Module}Repositories {
        {Module}Repositories {
            {entity1}_repository: self.build_{entity1}_repo(session),
            {entity2}_repository: self.build_{entity2}_repo(session),
        }
    }
}

/// Container for all {module} module repositories
pub struct {Module}Repositories {
    pub {entity1}_repository: Arc<{Entity1}RepositoryImpl>,
    pub {entity2}_repository: Arc<{Entity2}RepositoryImpl>,
}
```

### Real-World Example: PersonRepoFactory

```rust
use std::sync::Arc;
use parking_lot::RwLock as ParkingRwLock;
use postgres_unit_of_work::UnitOfWorkSession;
use postgres_index_cache::{CacheNotificationListener, IndexCacheHandler};
use business_core_db::models::person::{
    country::CountryIdxModel,
    country_subdivision::CountrySubdivisionIdxModel,
};
use super::{CountryRepositoryImpl, CountrySubdivisionRepositoryImpl};

/// Factory for creating person module repositories
///
/// This factory holds all caches for the person module and provides
/// methods to build repositories with the appropriate executor.
/// This should be used as a singleton throughout the application.
pub struct PersonRepoFactory {
    country_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<CountryIdxModel>>>,
    country_subdivision_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<CountrySubdivisionIdxModel>>>,
}

impl PersonRepoFactory {
    /// Create a new PersonRepoFactory singleton
    ///
    /// Optionally register cache handlers with a notification listener
    pub fn new(listener: Option<&mut CacheNotificationListener>) -> Arc<Self> {
        let country_idx_cache = Arc::new(ParkingRwLock::new(
            business_core_db::IdxModelCache::new(vec![]).unwrap()
        ));
        
        let country_subdivision_idx_cache = Arc::new(ParkingRwLock::new(
            business_core_db::IdxModelCache::new(vec![]).unwrap()
        ));
        
        // Register handlers with listener if provided
        if let Some(listener) = listener {
            let handler = Arc::new(IndexCacheHandler::new(
                "country_idx".to_string(),
                country_idx_cache.clone(),
            ));
            listener.register_handler(handler);
            
            let subdivision_handler = Arc::new(IndexCacheHandler::new(
                "country_subdivision_idx".to_string(),
                country_subdivision_idx_cache.clone(),
            ));
            listener.register_handler(subdivision_handler);
        }
        
        Arc::new(Self {
            country_idx_cache,
            country_subdivision_idx_cache,
        })
    }

    /// Build a CountryRepository with the given executor
    pub fn build_country_repo(&self, session: &impl UnitOfWorkSession) -> Arc<CountryRepositoryImpl> {
        let repo = Arc::new(CountryRepositoryImpl::new(
            session.executor().clone(),
            self.country_idx_cache.clone(),
        ));
        session.register_transaction_aware(repo.clone());
        repo
    }

    /// Build a CountrySubdivisionRepository with the given executor
    pub fn build_country_subdivision_repo(&self, session: &impl UnitOfWorkSession) -> Arc<CountrySubdivisionRepositoryImpl> {
        let repo = Arc::new(CountrySubdivisionRepositoryImpl::new(
            session.executor().clone(),
            self.country_subdivision_idx_cache.clone(),
        ));
        session.register_transaction_aware(repo.clone());
        repo
    }

    /// Build all person repositories with the given executor
    pub fn build_all_repos(&self, session: &impl UnitOfWorkSession) -> PersonRepositories {
        PersonRepositories {
            country_repository: self.build_country_repo(session),
            country_subdivision_repository: self.build_country_subdivision_repo(session),
        }
    }
}

/// Container for all person module repositories
pub struct PersonRepositories {
    pub country_repository: Arc<CountryRepositoryImpl>,
    pub country_subdivision_repository: Arc<CountrySubdivisionRepositoryImpl>,
}
```

### Factory Key Concepts

1. **Singleton Pattern**: The factory should be created once and shared across the application
2. **Cache Management**: The factory owns all module-level caches (wrapped in `Arc<ParkingRwLock<...>>`)
3. **Cache Notification**: Optionally register cache handlers with a notification listener for automatic cache updates
4. **Repository Building**: Provides methods to instantiate repositories with a given session/executor
5. **Transaction Awareness**: Automatically registers repositories as transaction-aware with the session
6. **Convenience Methods**: Provides `build_all_repos()` for creating all repositories at once

### Factory Usage Pattern

```rust
// Application initialization (once)
let mut listener = CacheNotificationListener::new(pool.clone());
let person_factory = PersonRepoFactory::new(Some(&mut listener));

// Per-transaction usage
let session = unit_of_work.start_session().await?;
let person_repos = person_factory.build_all_repos(&session);

// Use repositories
let countries = person_repos.country_repository
    .load_batch(&country_ids)
    .await?;

// Commit or rollback - repositories automatically handle cache updates
session.commit().await?;
```

### Adding New Entities to Existing Factory

When adding a new entity to an existing module:

1. **Add cache field** to the factory struct:
   ```rust
   {new_entity}_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<{NewEntity}IdxModel>>>,
   ```

2. **Initialize cache** in `new()`:
   ```rust
   let {new_entity}_idx_cache = Arc::new(ParkingRwLock::new(
       business_core_db::IdxModelCache::new(vec![]).unwrap()
   ));
   ```

3. **Register handler** in `new()`:
   ```rust
   if let Some(listener) = listener {
       let handler = Arc::new(IndexCacheHandler::new(
           "{new_entity}_idx".to_string(),
           {new_entity}_idx_cache.clone(),
       ));
       listener.register_handler(handler);
   }
   ```

4. **Add build method**:
   ```rust
   pub fn build_{new_entity}_repo(&self, session: &impl UnitOfWorkSession) -> Arc<{NewEntity}RepositoryImpl> {
       let repo = Arc::new({NewEntity}RepositoryImpl::new(
           session.executor().clone(),
           self.{new_entity}_idx_cache.clone(),
       ));
       session.register_transaction_aware(repo.clone());
       repo
   }
   ```

5. **Add field to repositories container**:
   ```rust
   pub struct {Module}Repositories {
       // ... existing fields
       pub {new_entity}_repository: Arc<{NewEntity}RepositoryImpl>,
   }
   ```

6. **Update `build_all_repos()`**:
   ```rust
   pub fn build_all_repos(&self, session: &impl UnitOfWorkSession) -> {Module}Repositories {
       {Module}Repositories {
           // ... existing repositories
           {new_entity}_repository: self.build_{new_entity}_repo(session),
       }
   }
   ```

---

## Module Registration

### 1. Update models/mod.rs

```rust
pub mod {module};

// In the file where entity is defined:
pub mod {entity};
pub use {entity}::*;
```

### 2. Update repository mod.rs

```rust
pub mod {entity}_repository;
pub use {entity}_repository::{Entity}RepositoryImpl;
```

---

## Type Mapping Guide

### Rust Types → SQL Types

| Rust Type | SQL Type | Nullable | Helper Function |
|-----------|----------|----------|----------------|
| `Uuid` | `UUID` | No | `row.get("field")` |
| `HeaplessString<N>` | `VARCHAR(N)` | No | `get_heapless_string(row, "field")?` |
| `Option<HeaplessString<N>>` | `VARCHAR(N)` | Yes | `get_optional_heapless_string(row, "field")?` |
| `i64` | `BIGINT` | No | `row.get("field")` |
| `Option<i64>` | `BIGINT` | Yes | `row.try_get("field").ok()` |
| `String` | `TEXT` | No | `row.get("field")` |
| `Option<String>` | `TEXT` | Yes | `row.get("field")` |
| `{EnumName}` | `{enum_name}` (custom) | No | `row.get("field")` |
| `Option<{EnumName}>` | `{enum_name}` (custom) | Yes | `row.get("field")` |

### Index Key Types

- **i64 keys**: Use for hash-based lookups (strings, enums)
- **UUID keys**: Use for foreign key relationships

---

## Database Schema Files

### Migration File Pattern

**Path**: `business-core/business-core-postgres/migrations/{number}_initial_schema_{module}_{entity}.sql`

```sql
-- Migration: Initial {Entity} Schema
-- Description: Creates {entity}-related tables and indexes

-- Enum Types (if any)
CREATE TYPE IF NOT EXISTS {enum_name} AS ENUM ('Variant1', 'Variant2');

-- {Entity} Table
CREATE TABLE IF NOT EXISTS {table_name} (
    id UUID PRIMARY KEY,
    -- ... other fields
    -- enum_field {enum_name} NOT NULL
);

-- {Entity} Index Table
CREATE TABLE IF NOT EXISTS {table_name}_idx (
    id UUID PRIMARY KEY REFERENCES {table_name}(id) ON DELETE CASCADE,
    -- ... index fields
);

-- Create trigger for {table_name}_idx table to notify listeners of changes
DROP TRIGGER IF EXISTS {table_name}_idx_notify ON {table_name}_idx;
CREATE TRIGGER {table_name}_idx_notify
    AFTER INSERT OR UPDATE OR DELETE ON {table_name}_idx
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();
```

### Cleanup File Pattern

**Path**: `business-core/business-core-postgres/cleanup/{number}_cleanup_{module}_{entity}.sql`

```sql
-- Cleanup: Initial {Entity} Schema
-- Description: Removes all artifacts created by {number}_initial_schema_{module}_{entity}.sql

-- Drop trigger first
DROP TRIGGER IF EXISTS {table_name}_idx_notify ON {table_name}_idx;

-- Drop tables (index table first due to foreign key constraint)
DROP TABLE IF EXISTS {table_name}_idx CASCADE;
DROP TABLE IF EXISTS {table_name} CASCADE;

-- Drop enum types (if any)
DROP TYPE IF EXISTS {enum_name};
```

---

## Usage Example

### Complete Generation Request

```
Generate a database access module for a "Locality" entity with the following specification:

Entity: Locality
Module: person
Table: locality
Index Table: locality_idx

Fields:
- id: Uuid (primary key)
- code: HeaplessString<20> (indexed)
- name: HeaplessString<100>
- country_subdivision_id: Uuid (indexed)
- description: Option<HeaplessString<500>>

Index Keys:
- code_hash: i64 (hash of code field)
- country_subdivision_id: Uuid (direct UUID index)

Custom Query Methods:
- find_ids_by_code_hash(code_hash: i64) -> Vec<Uuid>
- find_ids_by_country_subdivision_id(country_subdivision_id: Uuid) -> Vec<Uuid>

Database Files:
- Migration: migrations/004_initial_schema_person_locality.sql
- Cleanup: cleanup/004_cleanup_person_locality.sql
```

### Expected Output

The skill should generate:

1. `business-core/business-core-db/src/models/person/locality.rs` (model + index model)
2. `business-core/business-core-postgres/src/repository/person/locality_repository/` (full repository module)
3. All standard CRUD operations with tests
4. Custom query methods with tests
5. Database migration SQL file
6. Database cleanup SQL file

---

## Best Practices

### ✅ DO:

- Always use the same transaction lock pattern
- Update cache after releasing transaction lock
- Handle empty batches gracefully
- Use type-appropriate helper functions for field mapping
- Include comprehensive tests for all operations
- Use `pub(super)` for implementation methods
- Follow the module structure exactly

### ❌ DON'T:

- Hold transaction and cache locks simultaneously
- Modify cache before database operations complete
- Skip empty batch checks
- Use direct string parsing without helper functions
- Forget to implement all trait methods
- Mix sync and async operations incorrectly
- Ignore test coverage for edge cases

---

## Validation Checklist

After generating code, verify:

- [ ] All trait implementations are present (including TransactionAware)
- [ ] Transaction lock pattern is correct
- [ ] Cache is updated after database operations using read lock
- [ ] All cache accesses use `.await` for async operations
- [ ] Empty batch handling is included
- [ ] All fields are mapped correctly with proper helpers
- [ ] Index computation in `to_index()` is correct
- [ ] Custom query methods use cache correctly
- [ ] Tests cover happy path, empty, and error cases
- [ ] **Cache notification test is included** (required for entities with in-memory caches)
- [ ] Module registration is complete
- [ ] Code compiles without errors
- [ ] All imports are correct
- [ ] Database migration file is created
- [ ] Database cleanup file is created
- [ ] Database trigger for cache notification is included

---

## Advanced Features

### Multi-field Indexes

For entities with multiple indexed fields:

```rust
impl Indexable for {Entity}IdxModel {
    fn i64_keys(&self) -> HashMap<String, Option<i64>> {
        let mut keys = HashMap::new();
        keys.insert("field1_hash".to_string(), Some(self.field1_hash));
        keys.insert("field2_hash".to_string(), Some(self.field2_hash));
        keys
    }

    fn uuid_keys(&self) -> HashMap<String, Option<Uuid>> {
        let mut keys = HashMap::new();
        keys.insert("foreign_key_id".to_string(), Some(self.foreign_key_id));
        keys
    }
}
```

### Composite Custom Queries

For queries combining multiple indexes:

```rust
pub async fn find_by_field1_and_field2(
    &self,
    field1_hash: i64,
    field2_value: Uuid,
) -> Result<Vec<Uuid>, Box<dyn Error + Send + Sync>> {
    let cache = self.{entity}_idx_cache.read();
    
    // Get candidates from first index
    let candidates = cache
        .get_by_i64_index("field1_hash", &field1_hash)
        .cloned()
        .unwrap_or_default();
    
    // Filter by second index
    let result: Vec<Uuid> = candidates
        .into_iter()
        .filter(|id| {
            cache.get_by_primary(id)
                .map(|idx| idx.field2_value == field2_value)
                .unwrap_or(false)
        })
        .collect();
    
    Ok(result)
}
```

---

## Important Notes

### Repository Layer vs Application Layer

**Repository methods should:**
- Perform single, atomic data operations
- Use cache for index lookups
- Return IDs or simple models
- Be composable building blocks

**DO implement in repository:**
```rust
// Simple cache-based ID lookups
pub async fn find_ids_by_{index}(&self, value: T) -> Result<Vec<Uuid>, Error>

// Batch loading by IDs
pub async fn load_batch(&self, ids: &[Uuid]) -> Result<Vec<Option<Model>>, Error>
```

**DON'T implement in repository:**
```rust
// Composite operations combining multiple repository methods
pub async fn find_by_{index}(&self, value: T) -> Result<Vec<Model>, Error> {
    let ids = self.find_ids_by_{index}(value).await?;
    let results = self.load_batch(&ids).await?;
    Ok(results.into_iter().flatten().collect())
}
```

**Instead, compose at service/application layer:**
```rust
// In your service or application code
let ids = repo.find_ids_by_country_id(country_id).await?;
let subdivisions = repo.load_batch(&ids).await?;
let valid_subdivisions: Vec<_> = subdivisions.into_iter().flatten().collect();
```

This keeps repositories focused on atomic operations and allows flexible composition at higher layers.

---

## Troubleshooting

### Common Issues

**Issue**: Transaction consumed error
- **Cause**: Transaction used after being consumed
- **Fix**: Ensure transaction is only used once per operation

**Issue**: Deadlock or slow performance
- **Cause**: Holding transaction lock while updating cache
- **Fix**: Release transaction lock before cache operations

**Issue**: Cache out of sync
- **Cause**: Cache updated before database operation completes
- **Fix**: Update cache only after successful database operation

**Issue**: HeaplessString conversion fails
- **Cause**: String exceeds capacity
- **Fix**: Verify field size matches HeaplessString capacity

---

## References

- **Trait Definitions**: `business-core/business-core-db/src/models/`
- **Repository Traits**: `business-core/business-core-db/src/repository/`
- **Cache Library**: `postgres-index-cache` crate
- **Transaction Management**: `postgres-unit-of-work` crate
- **Complete Example**: Country entity implementation

---

## Version History

- **v1.0** - Initial skill based on Country entity pattern
- Supports: CRUD, batch operations, indexing, caching, testing

---

## License

Same as business-core project