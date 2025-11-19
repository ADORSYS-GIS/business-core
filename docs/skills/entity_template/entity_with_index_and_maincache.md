# Entity Template Skill: Entities with Index and Main Cache

## Overview

This skill extends the [Entity with Index](entity_with_index.md) template to add **main entity caching** functionality. It generates complete database access modules for entities that require both application-layer indexing and full entity caching with eviction policies.

## Purpose

This skill builds upon the base indexable entity template by adding:
- ✅ **Main entity cache** with configurable eviction policies (LRU/FIFO)
- ✅ **Transaction-aware cache** for atomic cache updates
- ✅ **Database trigger notifications** for cache synchronization across application nodes
- ✅ **Cache statistics** for monitoring hit rates and performance
- ✅ **TTL support** for automatic cache expiration
- ✅ **Staged changes** that commit or rollback with transactions

## Prerequisites

**You must read and understand** [Entity with Index](entity_with_index.md) first. This document only covers the **additional** patterns for main cache functionality. All base patterns from the indexable entity template still apply.

**CRITICAL PATTERN - Finder Methods**: The finder method pattern for secondary index fields described in the base template applies to cacheable entities as well.

**Rule**: For each secondary index field (field name not equal to `id`) in your `{Entity}IdxModel`, you MUST create a corresponding finder method that returns `Vec<{Entity}IdxModel>`.

See the [Finder Methods section](entity_with_index.md#finder-methods-for-secondary-index-fields) in the base template for detailed implementation patterns.

---

## Template Reference

The main cache pattern is based on:
- **Index Cache**: `postgres-index-cache/src/index_model_cache.rs` (from base template)
- **Main Cache**: `postgres-index-cache/src/main_model_cache.rs`
- **Transaction-Aware Cache**: `postgres-index-cache/src/transaction_aware_main_model_cache.rs`
- **Cache Handler**: Database trigger registration and notification handling

---

## Key Differences from Index-Only Template

### 1. Cache Architecture

**Index-Only Template** (base):
- Only caches index models (`{Entity}IdxModel`)
- Lightweight - only ID + index fields
- Always preloaded at startup
- No eviction needed (small size)

**Index + Main Cache Template** (this):
- Caches both index models AND full entity models (`{Entity}Model`)
- Full entity data in cache
- **NOT preloaded** - entities added on-demand
- Requires eviction policy (limited size)
- Cache statistics and TTL support

### 2. Database Triggers

**Index-Only Template**:
- Trigger on `{table_name}_idx` table only

**Index + Main Cache Template**:
- Trigger on `{table_name}` table (the main entity table)
- Notifies all application nodes of entity changes
- Keeps main cache synchronized across the cluster

### 3. Repository Structure

**Index-Only Template**:
```rust
pub struct {Entity}RepositoryImpl {
    pub executor: Executor,
    pub {entity}_idx_cache: Arc<RwLock<TransactionAwareIdxModelCache<{Entity}IdxModel>>>,
}
```

**Index + Main Cache Template**:
```rust
pub struct {Entity}RepositoryImpl {
    pub executor: Executor,
    pub {entity}_idx_cache: Arc<RwLock<TransactionAwareIdxModelCache<{Entity}IdxModel>>>,
    pub {entity}_cache: Arc<RwLock<TransactionAwareMainModelCache<{Entity}Model>>>,
}
```

---

## Additional Artifacts for Main Cache

### 1. Main Model Traits

The main entity model must implement `HasPrimaryKey`:

```rust
use postgres_index_cache::HasPrimaryKey;
use uuid::Uuid;

impl HasPrimaryKey for {Entity}Model {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}
```

### 2. Repository Implementation with Main Cache

**Location**: `business-core/business-core-postgres/src/repository/{module}/{entity}_repository/repo_impl.rs`

```rust
use business_core_db::models::{module}::{entity}::{Entity}IdxModel, {Entity}Model};
use crate::utils::{get_heapless_string, get_optional_heapless_string, TryFromRow};
use postgres_unit_of_work::{Executor, TransactionAware, TransactionResult};
use postgres_index_cache::{TransactionAwareIdxModelCache, TransactionAwareMainModelCache};
use parking_lot::RwLock as ParkingRwLock;
use tokio::sync::RwLock;
use std::sync::Arc;
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use async_trait::async_trait;

pub struct {Entity}RepositoryImpl {
    pub executor: Executor,
    pub {entity}_idx_cache: Arc<RwLock<TransactionAwareIdxModelCache<{Entity}IdxModel>>>,
    pub {entity}_cache: Arc<RwLock<TransactionAwareMainModelCache<{Entity}Model>>>,
}

impl {Entity}RepositoryImpl {
    pub fn new(
        executor: Executor,
        {entity}_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<{Entity}IdxModel>>>,
        {entity}_cache: Arc<ParkingRwLock<postgres_index_cache::MainModelCache<{Entity}Model>>>,
    ) -> Self {
        Self {
            executor,
            {entity}_idx_cache: Arc::new(RwLock::new(TransactionAwareIdxModelCache::new(
                {entity}_idx_cache,
            ))),
            {entity}_cache: Arc::new(RwLock::new(TransactionAwareMainModelCache::new(
                {entity}_cache,
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
        self.{entity}_idx_cache.read().await.on_commit().await?;
        self.{entity}_cache.read().await.on_commit().await?;
        Ok(())
    }

    async fn on_rollback(&self) -> TransactionResult<()> {
        self.{entity}_idx_cache.read().await.on_rollback().await?;
        self.{entity}_cache.read().await.on_rollback().await?;
        Ok(())
    }
}
```

**Key Points**:
- Two cache fields: index cache and main cache
- Both caches are transaction-aware
- `TransactionAware` commits/rolls back both caches
- Main cache NOT preloaded (added on-demand)

---

## Repository Method Patterns with Main Cache

### CREATE Path Pattern

```rust
use business_core_db::models::index_aware::IndexAware;

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
        
        // Release transaction lock before updating caches
        drop(tx);
        
        // Update BOTH caches after releasing transaction lock
        {
            let idx_cache = repo.{entity}_idx_cache.read().await;
            let main_cache = repo.{entity}_cache.read().await;
            
            for (idx, item) in indices.iter().zip(saved_items.iter()) {
                idx_cache.add(idx.clone());
                main_cache.insert(item.clone());
            }
        }

        Ok(saved_items)
    }
}
```

**Key Changes**:
- Update BOTH index cache AND main cache
- Main cache uses `insert()` method
- Both caches staged in transaction-aware layer

### LOAD Path Pattern

```rust
impl {Entity}RepositoryImpl {
    pub(super) async fn load_batch_impl(
        repo: &{Entity}RepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<Option<{Entity}Model>>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        
        // Try to get from cache first
        let main_cache = repo.{entity}_cache.read().await;
        let mut result = Vec::with_capacity(ids.len());
        let mut missing_ids = Vec::new();
        
        for &id in ids {
            match main_cache.get(&id) {
                Some(item) => result.push(Some(item)),
                None => {
                    result.push(None);
                    missing_ids.push(id);
                }
            }
        }
        
        drop(main_cache); // Release read lock
        
        // If all found in cache, return early
        if missing_ids.is_empty() {
            return Ok(result);
        }
        
        // Load missing items from database
        let query = r#"SELECT * FROM {table_name} WHERE id = ANY($1)"#;
        let rows = {
            let mut tx = repo.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
            sqlx::query(query).bind(&missing_ids).fetch_all(&mut **transaction).await?
        };
        
        let mut loaded_map = std::collections::HashMap::new();
        for row in rows {
            let item = {Entity}Model::try_from_row(&row)?;
            loaded_map.insert(item.id, item);
        }
        
        // Update result and add to cache
        let main_cache = repo.{entity}_cache.read().await;
        for (i, &id) in ids.iter().enumerate() {
            if result[i].is_none() {
                if let Some(item) = loaded_map.remove(&id) {
                    main_cache.insert(item.clone());
                    result[i] = Some(item);
                }
            }
        }
        
        Ok(result)
    }
}
```

**Key Changes**:
- First check main cache for entities
- Only query database for cache misses
- Add loaded entities to cache
- Maintains order of requested IDs

### UPDATE Path Pattern

```rust
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
        
        drop(tx); // Release transaction lock
        
        // Update BOTH caches after releasing transaction lock
        {
            let idx_cache = self.{entity}_idx_cache.read().await;
            let main_cache = self.{entity}_cache.read().await;
            
            for (id, idx) in indices.iter() {
                idx_cache.remove(id);
                idx_cache.add(idx.clone());
                // Main cache update replaces the entire entity
                main_cache.update(updated_items.iter().find(|i| i.id == *id).unwrap().clone());
            }
        }

        Ok(updated_items)
    }
}
```

**Key Changes**:
- Update BOTH index cache AND main cache
- Main cache uses `update()` method (or remove + insert)

### DELETE Path Pattern

```rust
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
        
        drop(tx); // Release transaction lock
        
        // Update BOTH caches after releasing transaction lock
        {
            let idx_cache = repo.{entity}_idx_cache.read().await;
            let main_cache = repo.{entity}_cache.read().await;
            
            for id in ids {
                idx_cache.remove(id);
                main_cache.remove(id);
            }
        }
        
        Ok(rows_affected)
    }
}
```

**Key Changes**:
- Remove from BOTH index cache AND main cache

---

## Database Schema with Main Cache

```sql
-- Migration: Initial {Entity} Schema with Index and Main Cache
-- Description: Creates {entity}-related tables with index and cache notification triggers

-- {Entity} Table
CREATE TABLE IF NOT EXISTS {table_name} (
    id UUID PRIMARY KEY,
    field1 VARCHAR(...),
    field2 VARCHAR(...),
    -- ... other fields
);

-- {Entity} Index Table
CREATE TABLE IF NOT EXISTS {table_name}_idx (
    id UUID PRIMARY KEY REFERENCES {table_name}(id) ON DELETE CASCADE,
    index_field1 BIGINT,
    index_field2 UUID,
    -- ... other index fields
);

-- Create trigger for {table_name}_idx table to notify index cache changes
DROP TRIGGER IF EXISTS {table_name}_idx_notify ON {table_name}_idx;
CREATE TRIGGER {table_name}_idx_notify
    AFTER INSERT OR UPDATE OR DELETE ON {table_name}_idx
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();

-- Create trigger for {table_name} table to notify main cache changes
DROP TRIGGER IF EXISTS {table_name}_notify ON {table_name};
CREATE TRIGGER {table_name}_notify
    AFTER INSERT OR UPDATE OR DELETE ON {table_name}
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();
```

**Key Points**:
- TWO triggers: one for index cache, one for main cache
- Index trigger on `{table_name}_idx` table
- **Main trigger on `{table_name}` table** (the main entity table)
- Both use same `notify_cache_change()` function

---

## Repository Factory Pattern with Main Cache

### Factory Implementation

```rust
use std::sync::Arc;
use parking_lot::RwLock as ParkingRwLock;
use postgres_unit_of_work::UnitOfWorkSession;
use postgres_index_cache::{CacheNotificationListener, IndexCacheHandler, MainModelCacheHandler, MainModelCache, CacheConfig, EvictionPolicy};
use std::time::Duration;
use business_core_db::models::{module}::{entity}::{Entity}IdxModel, {Entity}Model};
use super::{Entity}RepositoryImpl;

/// Factory for creating {module} module repositories with main cache
pub struct {Module}RepoFactory {
    {entity}_idx_cache: Arc<ParkingRwLock<business_core_db::IdxModelCache<{Entity}IdxModel>>>,
    {entity}_cache: Arc<ParkingRwLock<MainModelCache<{Entity}Model>>>,
}

impl {Module}RepoFactory {
    /// Create a new {Module}RepoFactory singleton with cache configuration
    ///
    /// Optionally register cache handlers with a notification listener
    pub fn new(listener: Option<&mut CacheNotificationListener>) -> Arc<Self> {
        // Initialize index cache (as in base template)
        let {entity}_idx_cache = Arc::new(ParkingRwLock::new(
            business_core_db::IdxModelCache::new(vec![]).unwrap()
        ));
        
        // Initialize main cache with configuration
        let cache_config = CacheConfig::new(
            1000,  // Max 1000 entities in cache
            EvictionPolicy::LRU,  // Least Recently Used eviction
        )
        .with_ttl(Duration::from_secs(3600)); // 1 hour TTL
        
        let {entity}_cache = Arc::new(ParkingRwLock::new(
            MainModelCache::new(cache_config)
        ));
        
        // Register handlers with listener if provided
        if let Some(listener) = listener {
            // Register index cache handler
            let idx_handler = Arc::new(IndexCacheHandler::new(
                "{entity}_idx".to_string(),
                {entity}_idx_cache.clone(),
            ));
            listener.register_handler(idx_handler);
            
            // Register main cache handler
            let main_handler = Arc::new(MainModelCacheHandler::new(
                "{entity}".to_string(),  // Note: main table name, not _idx
                {entity}_cache.clone(),
            ));
            listener.register_handler(main_handler);
        }
        
        Arc::new(Self {
            {entity}_idx_cache,
            {entity}_cache,
        })
    }

    /// Build a {Entity}Repository with the given executor
    pub fn build_{entity}_repo(&self, session: &impl UnitOfWorkSession) -> Arc<{Entity}RepositoryImpl> {
        let repo = Arc::new({Entity}RepositoryImpl::new(
            session.executor().clone(),
            self.{entity}_idx_cache.clone(),
            self.{entity}_cache.clone(),
        ));
        session.register_transaction_aware(repo.clone());
        repo
    }

    /// Build all {module} repositories with the given executor
    pub fn build_all_repos(&self, session: &impl UnitOfWorkSession) -> {Module}Repositories {
        {Module}Repositories {
            {entity}_repository: self.build_{entity}_repo(session),
        }
    }
}

/// Container for all {module} module repositories
pub struct {Module}Repositories {
    pub {entity}_repository: Arc<{Entity}RepositoryImpl>,
}
```

**Key Points**:
- Factory holds BOTH index cache AND main cache
- Main cache configured with eviction policy and TTL
- TWO handlers registered: `IndexCacheHandler` and `MainModelCacheHandler`
- Main handler uses main table name (not `_idx`)
- Repository receives both caches in constructor

---

## Cache Configuration Options

### Eviction Policies

```rust
use postgres_index_cache::{CacheConfig, EvictionPolicy};
use std::time::Duration;

// LRU (Least Recently Used) - recommended for most cases
let lru_config = CacheConfig::new(1000, EvictionPolicy::LRU)
    .with_ttl(Duration::from_secs(3600));

// FIFO (First In First Out) - simpler, no access tracking
let fifo_config = CacheConfig::new(1000, EvictionPolicy::FIFO)
    .with_ttl(Duration::from_secs(1800));
```

### Cache Statistics

```rust
// Access cache statistics
let main_cache = repo.{entity}_cache.read().await;
let stats = main_cache.statistics();

println!("Cache hits: {}", stats.hits());
println!("Cache misses: {}", stats.misses());
println!("Hit rate: {:.2}%", stats.hit_rate() * 100.0);
println!("Evictions: {}", stats.evictions());
println!("Invalidations: {}", stats.invalidations());
```

---

## Testing Patterns with Main Cache

### Standard Test Cases (extends base template)

```rust
#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_batch_updates_main_cache() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let {entity}_repo = &ctx.{module}_repos().{entity}_repository;

        let items = vec![/* test data */];
        let saved = {entity}_repo.create_batch(items, None).await?;

        // Verify entities are in main cache
        let main_cache = {entity}_repo.{entity}_cache.read().await;
        for item in &saved {
            assert!(main_cache.contains(&item.id), "Entity should be in main cache");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_load_batch_uses_cache() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let {entity}_repo = &ctx.{module}_repos().{entity}_repository;

        // Create entities
        let items = vec![/* test data */];
        let saved = {entity}_repo.create_batch(items, None).await?;
        let ids: Vec<Uuid> = saved.iter().map(|i| i.id).collect();

        // First load - should populate cache
        let loaded1 = {entity}_repo.load_batch(&ids).await?;
        
        // Second load - should hit cache
        let loaded2 = {entity}_repo.load_batch(&ids).await?;
        
        assert_eq!(loaded1.len(), loaded2.len());
        
        // Verify cache statistics
        let main_cache = {entity}_repo.{entity}_cache.read().await;
        let stats = main_cache.statistics();
        assert!(stats.hits() > 0, "Should have cache hits on second load");

        Ok(())
    }

    #[tokio::test]
    async fn test_update_batch_updates_main_cache() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let {entity}_repo = &ctx.{module}_repos().{entity}_repository;

        // Create and update
        let items = vec![/* test data */];
        let mut saved = {entity}_repo.create_batch(items, None).await?;
        
        saved[0].field1 = /* updated value */;
        let updated = {entity}_repo.update_batch(saved, None).await?;

        // Verify updated entity in cache
        let main_cache = {entity}_repo.{entity}_cache.read().await;
        let cached = main_cache.get(&updated[0].id);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().field1, updated[0].field1);

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_batch_removes_from_main_cache() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let {entity}_repo = &ctx.{module}_repos().{entity}_repository;

        // Create entities
        let items = vec![/* test data */];
        let saved = {entity}_repo.create_batch(items, None).await?;
        let ids: Vec<Uuid> = saved.iter().map(|i| i.id).collect();

        // Delete entities
        let deleted_count = {entity}_repo.delete_batch(&ids, None).await?;
        assert_eq!(deleted_count, ids.len());

        // Verify removed from main cache
        let main_cache = {entity}_repo.{entity}_cache.read().await;
        for id in &ids {
            assert!(!main_cache.contains(id), "Entity should be removed from main cache");
        }

        Ok(())
    }
}
```

### Cache Notification Test (REQUIRED)

```rust
#[tokio::test]
async fn test_{entity}_insert_triggers_main_cache_notification() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    use crate::test_helper::{random, setup_test_context_and_listen};
    use business_core_db::models::index_aware::IndexAware;
    use tokio::time::{sleep, Duration};

    // Setup test context with the notification listener
    let ctx = setup_test_context_and_listen().await?;
    let pool = ctx.pool();

    // Create a test entity
    let unique_field = random(5);
    let test_entity = create_test_{entity}(&unique_field, "Test Entity");
    let entity_idx = test_entity.to_index();

    // Give listener time to start
    sleep(Duration::from_millis(2000)).await;

    // Insert the entity record directly into database (triggers main cache notification)
    sqlx::query("INSERT INTO {table_name} ({field_list}) VALUES ({placeholders})")
        // ... bind all fields
        .execute(&**pool)
        .await
        .expect("Failed to insert {entity}");

    // Insert the index record directly into database (triggers index cache notification)
    sqlx::query("INSERT INTO {table_name}_idx ({index_field_list}) VALUES ({index_placeholders})")
        // ... bind index fields
        .execute(&**pool)
        .await
        .expect("Failed to insert {entity} index");

    // Give time for notification to be processed
    sleep(Duration::from_millis(500)).await;

    let entity_repo = &ctx.{module}_repos().{entity}_repository;

    // Verify the INDEX cache was updated
    let idx_cache = entity_repo.{entity}_idx_cache.read().await;
    assert!(
        idx_cache.contains_primary(&entity_idx.id),
        "{Entity} should be in index cache after insert"
    );
    drop(idx_cache);

    // Verify the MAIN cache was updated
    let main_cache = entity_repo.{entity}_cache.read().await;
    assert!(
        main_cache.contains(&test_entity.id),
        "{Entity} should be in main cache after insert"
    );
    drop(main_cache);

    // Delete the record from database (triggers both cache notifications)
    sqlx::query("DELETE FROM {table_name} WHERE id = $1")
        .bind(test_entity.id)
        .execute(&**pool)
        .await
        .expect("Failed to delete {entity}");

    // Give time for notification to be processed
    sleep(Duration::from_millis(500)).await;

    // Verify removed from both caches
    let idx_cache = entity_repo.{entity}_idx_cache.read().await;
    assert!(
        !idx_cache.contains_primary(&entity_idx.id),
        "{Entity} should be removed from index cache after delete"
    );
    drop(idx_cache);

    let main_cache = entity_repo.{entity}_cache.read().await;
    assert!(
        !main_cache.contains(&test_entity.id),
        "{Entity} should be removed from main cache after delete"
    );

    Ok(())
}
```

**Key Points**:
- Tests BOTH index cache AND main cache notifications
- Verifies direct database operations trigger cache updates
- Tests both INSERT and DELETE operations

---

## Validation Checklist

Extends the base template checklist with:

- [ ] All base template validations (from entity_with_index.md)
- [ ] Main entity model implements `HasPrimaryKey` trait
- [ ] Repository has both `{entity}_idx_cache` and `{entity}_cache` fields
- [ ] Repository constructor accepts both caches
- [ ] `TransactionAware` commits/rollbacks both caches
- [ ] CREATE updates both index and main cache
- [ ] LOAD checks main cache first before database query
- [ ] LOAD adds cache misses to main cache
- [ ] UPDATE updates both index and main cache
- [ ] DELETE removes from both index and main cache
- [ ] Factory initializes both caches with configuration
- [ ] Factory registers TWO handlers: index and main
- [ ] Main cache handler uses main table name (not `_idx`)
- [ ] Database has TWO triggers: one on `{table_name}_idx`, one on `{table_name}`
- [ ] Tests verify main cache synchronization
- [ ] Cache notification test covers both caches
- [ ] Cache statistics are accessible for monitoring

---

## Best Practices

### ✅ DO:

- Configure appropriate cache size based on expected data volume
- Use LRU eviction policy for most use cases
- Set reasonable TTL values to prevent stale data
- Monitor cache statistics in production
- Test cache hit rates to validate configuration
- Update both caches atomically in repository methods
- Release transaction lock before updating caches
- Check main cache before querying database in load operations
- Register both index and main cache handlers
- Create triggers on both `{table_name}_idx` and `{table_name}` tables

### ❌ DON'T:

- Preload main cache at startup (entities added on-demand)
- Use excessively large cache sizes (consider memory constraints)
- Skip cache updates after database operations
- Hold transaction and cache locks simultaneously
- Query database without checking cache first
- Forget to configure eviction policy
- Use same handler for both index and main cache
- Create trigger only on index table (need both)

---

## When to Use This Template

Use this template for entities that:
- ✅ Need secondary index lookups (hash-based or UUID-based)
- ✅ Are frequently accessed by ID after initial lookup
- ✅ Benefit from full entity caching for performance
- ✅ Have moderate data volume (can fit in configured cache size)
- ✅ Require cross-node cache synchronization

**Do NOT use** this template if:
- ❌ Entities are too large to cache efficiently
- ❌ Data volume exceeds reasonable cache size limits
- ❌ Access patterns don't benefit from caching (always query fresh)
- ❌ Only need index lookups without full entity data

For simpler cases, use [Entity with Index](entity_with_index.md) instead.

---

## Performance Considerations

### Cache Size Tuning

```rust
// Small, frequently accessed entities
CacheConfig::new(5000, EvictionPolicy::LRU)

// Medium entities, moderate access
CacheConfig::new(1000, EvictionPolicy::LRU)

// Large entities, limited cache
CacheConfig::new(100, EvictionPolicy::LRU)
```

### TTL Configuration

```rust
// Frequently changing data - short TTL
.with_ttl(Duration::from_secs(300))  // 5 minutes

// Stable data - longer TTL
.with_ttl(Duration::from_secs(3600))  // 1 hour

// Very stable data - very long TTL
.with_ttl(Duration::from_secs(86400))  // 24 hours
```

### Monitoring

```rust
// Periodically log cache statistics
async fn log_cache_stats(repo: &{Entity}RepositoryImpl) {
    let main_cache = repo.{entity}_cache.read().await;
    let stats = main_cache.statistics();
    
    tracing::info!(
        "{} cache - Size: {}, Hit rate: {:.2}%, Evictions: {}",
        "{Entity}",
        main_cache.len(),
        stats.hit_rate() * 100.0,
        stats.evictions()
    );
}
```

---

## References

- **Base Template**: [Entity with Index](entity_with_index.md)
- **Main Cache Implementation**: `postgres-index-cache/src/main_model_cache.rs`
- **Transaction-Aware Cache**: `postgres-index-cache/src/transaction_aware_main_model_cache.rs`
- **Cache Traits**: `postgres-index-cache/src/traits.rs`
- **Complete Example**: Reference implementations in business-core

---

## License

Same as business-core project