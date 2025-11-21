# Entity Template Skill: Entities with Index, Audit, and Main Cache

## Overview

This skill extends the [Entity with Index](entity_with_index.md) template to add both **audit trail functionality** and **main entity caching**. It generates complete database access modules for entities that require application-layer indexing, comprehensive audit logging, and full entity caching with eviction policies.

## Purpose

This skill builds upon the base indexable entity template by adding:
- ✅ **Audit table** with complete entity state snapshots stored as tuples
- ✅ **Hash-based verification** using xxHash64 for audit integrity
- ✅ **Audit log integration** with audit_log_id references
- ✅ **Serialization-based hashing** using CBOR encoding
- ✅ **Transaction-level entity tracking** via the `audit_link` table
- ✅ **Main entity cache** with configurable eviction policies (LRU/FIFO)
- ✅ **Transaction-aware cache** for atomic cache updates
- ✅ **Database trigger notifications** for cache synchronization across application nodes
- ✅ **Cache statistics** for monitoring hit rates and performance
- ✅ **TTL support** for automatic cache expiration
- ✅ **Staged changes** that commit or rollback with transactions

## Prerequisites

**You must read and understand** [Entity with Index](entity_with_index.md) first. This document only covers the **additional** patterns for audit and main cache functionality. All base patterns from the indexable entity template still apply.

**CRITICAL PATTERN - Finder Methods**: The finder method pattern for secondary index fields described in the base template applies to auditable entities with main cache as well.

**Rule**: For each secondary index field (field name not equal to `id`) in your `{Entity}IdxModel`, you MUST create a corresponding finder method that returns `Vec<{Entity}IdxModel>`.

See the [Finder Methods section](entity_with_index.md#finder-methods-for-secondary-index-fields) in the base template for detailed implementation patterns.

---

## Template Reference

This pattern combines:
- **Index Cache**: `postgres-index-cache/src/index_model_cache.rs` (from base template)
- **Main Cache**: `postgres-index-cache/src/main_model_cache.rs`
- **Transaction-Aware Cache**: `postgres-index-cache/src/transaction_aware_main_model_cache.rs`
- **Audit Traits**: `business-core/business-core-db/src/models/auditable.rs`
- **Cache Handler**: Database trigger registration and notification handling

---

## Key Differences from Other Templates

### From Index-Only Template

**Index-Only Template** (base):
- Only caches index models (`{Entity}IdxModel`)
- No audit trail
- Always preloaded at startup

**This Template**:
- Caches both index models AND full entity models
- Full audit trail with hash verification
- Main cache NOT preloaded - entities added on-demand
- Requires eviction policy (limited size)

### From Index + Audit Template

**Index + Audit Template**:
- Audit trail but no main cache
- Only index cache (preloaded)

**This Template**:
- Audit trail AND main cache
- Main cache for performance optimization
- Cache statistics and TTL support

### From Index + Main Cache Template

**Index + Main Cache Template**:
- Main cache but no audit trail
- No hash verification
- Trigger on main table only

**This Template**:
- Main cache AND audit trail
- Hash-based verification
- Trigger on main table (covers both cache and audit)

---

## Architecture Overview

### Cache Architecture

- Caches both index models AND full entity models (`{Entity}Model`)
- Full entity data in cache
- **NOT preloaded** - entities added on-demand
- Requires eviction policy (limited size)
- Cache statistics and TTL support

### Database Triggers

- Trigger on `{table_name}_idx` table for index cache
- Trigger on `{table_name}` table for main cache synchronization
- Notifies all application nodes of entity changes

### Repository Structure

```rust
pub struct {Entity}RepositoryImpl {
    pub executor: Executor,
    pub {entity}_idx_cache: Arc<RwLock<TransactionAwareIdxModelCache<{Entity}IdxModel>>>,
    pub {entity}_cache: Arc<RwLock<TransactionAwareMainModelCache<{Entity}Model>>>,
}
```

---

## Additional Artifacts

### 1. Main Model Modifications

The main entity model must include audit fields and implement cache traits:

```rust
use postgres_index_cache::HasPrimaryKey;

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

impl HasPrimaryKey for {Entity}Model {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}
```

### 2. Auditable Trait Implementation

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

### 3. Repository Implementation with Audit and Main Cache

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
            hash: row.get("hash"),
            audit_log_id: row.get("audit_log_id"),
            antecedent_hash: row.get("antecedent_hash"),
            antecedent_audit_log_id: row.get("antecedent_audit_log_id"),
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
- Repository handles audit fields in all operations

---

## Repository Method Patterns

### CREATE Path Pattern

```rust
use business_core_db::models::index_aware::IndexAware;
use business_core_db::utils::hash_as_i64;
use business_core_db::models::audit::audit_link::{AuditLinkModel, EntityType};

impl {Entity}RepositoryImpl {
    pub(super) async fn create_batch_impl(
        repo: &{Entity}RepositoryImpl,
        items: Vec<{Entity}Model>,
        audit_log_id: Uuid,
    ) -> Result<Vec<{Entity}Model>, Box<dyn Error + Send + Sync>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut saved_items = Vec::new();
        let mut indices = Vec::new();
        
        // Acquire lock once and do all database operations
        let mut tx = repo.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        
        for mut item in items {
            // 1. Compute hash for audit
            let mut entity_for_hashing = item.clone();
            entity_for_hashing.hash = 0;
            entity_for_hashing.audit_log_id = Some(audit_log_id);
            
            let computed_hash = hash_as_i64(&entity_for_hashing)?;
            
            // 2. Update entity with computed hash and audit_log_id
            item.hash = computed_hash;
            item.audit_log_id = Some(audit_log_id);
            
            // 3. Insert into audit table first
            sqlx::query(
                r#"
                INSERT INTO {table_name}_audit ({field_list}, hash, audit_log_id)
                VALUES ({value_placeholders})
                "#,
            )
            // ... bind all fields including hash and audit_log_id
            .execute(&mut **transaction)
            .await?;
            
            // 4. Insert into main table
            sqlx::query(
                r#"
                INSERT INTO {table_name} ({field_list}, hash, audit_log_id)
                VALUES ({value_placeholders})
                "#,
            )
            // ... bind all fields
            .execute(&mut **transaction)
            .await?;
            
            // 5. Insert into index table
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
            
            // 6. Create audit link
            let audit_link = AuditLinkModel {
                audit_log_id,
                entity_id: item.id,
                entity_type: EntityType::{Entity},
            };
            sqlx::query(
                r#"
                INSERT INTO audit_link (audit_log_id, entity_id, entity_type)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(audit_link.audit_log_id)
            .bind(audit_link.entity_id)
            .bind(audit_link.entity_type)
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

**Key Points**:
- Compute hash before insertion
- Insert audit record FIRST (critical order)
- Create audit link for tracking
- Update BOTH index cache AND main cache
- Main cache uses `insert()` method

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
        
        // Try to get from main cache first
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

**Key Points**:
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
        audit_log_id: Uuid,
    ) -> Result<Vec<{Entity}Model>, Box<dyn Error + Send + Sync>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut updated_items = Vec::new();
        let mut indices = Vec::new();
        
        // Acquire lock once and do all database operations
        let mut tx = self.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        
        for mut item in items {
            // 1. Save current hash and audit_log_id for antecedent tracking
            let previous_hash = item.hash;
            let previous_audit_log_id = item.audit_log_id.ok_or("Entity must have audit_log_id for update")?;
            
            // 2. Check if entity has actually changed by recomputing hash
            let mut entity_for_hashing = item.clone();
            entity_for_hashing.hash = 0;
            
            let computed_hash = hash_as_i64(&entity_for_hashing)?;
            
            // 3. Only proceed with update if entity has changed
            if computed_hash == previous_hash {
                updated_items.push(item);
                continue;
            }
            
            // 4. Entity has changed, set antecedent fields
            item.antecedent_hash = previous_hash;
            item.antecedent_audit_log_id = previous_audit_log_id;
            
            // 5. Update with new hash and audit_log_id
            item.audit_log_id = Some(audit_log_id);
            item.hash = 0;
            
            let new_computed_hash = hash_as_i64(&item)?;
            item.hash = new_computed_hash;
            
            // 6. Insert into audit table first
            sqlx::query(
                r#"
                INSERT INTO {table_name}_audit
                ({field_list}, hash, audit_log_id, antecedent_hash, antecedent_audit_log_id)
                VALUES ({value_placeholders})
                "#,
            )
            // ... bind all fields
            .execute(&mut **transaction)
            .await?;
            
            // 7. Update main table
            sqlx::query(
                r#"
                UPDATE {table_name} SET
                    {field_updates},
                    hash = $N,
                    audit_log_id = $N+1,
                    antecedent_hash = $N+2,
                    antecedent_audit_log_id = $N+3
                WHERE id = $1
                  AND hash = $N+2
                  AND audit_log_id = $N+3
                "#,
            )
            // ... bind fields
            .execute(&mut **transaction)
            .await?;
            
            // 8. Create audit link
            let audit_link = AuditLinkModel {
                audit_log_id,
                entity_id: item.id,
                entity_type: EntityType::{Entity},
            };
            sqlx::query(
                r#"
                INSERT INTO audit_link (audit_log_id, entity_id, entity_type)
                VALUES ($1, $2, $3)
                "#,
            )
            .bind(audit_link.audit_log_id)
            .bind(audit_link.entity_id)
            .bind(audit_link.entity_type)
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
                main_cache.update(updated_items.iter().find(|i| i.id == *id).unwrap().clone());
            }
        }

        Ok(updated_items)
    }
}
```

**Key Points**:
- Check for actual changes using hash comparison
- Update antecedent fields for audit chain
- Insert audit record FIRST
- Create audit link
- Update BOTH index cache AND main cache

### DELETE Path Pattern

```rust
impl {Entity}RepositoryImpl {
    pub(super) async fn delete_batch_impl(
        repo: &{Entity}RepositoryImpl,
        ids: &[Uuid],
        audit_log_id: Uuid,
    ) -> Result<usize, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(0);
        }

        // 1. Load entities to get their final state for auditing
        let entities_to_delete = repo.load_batch(ids).await?;
        let mut rows_affected = 0;

        let mut tx = repo.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;

        for entity_option in &entities_to_delete {
            if let Some(entity) = entity_option {
                // 2. Create final audit record
                let mut final_audit_entity = entity.clone();
                
                final_audit_entity.antecedent_hash = entity.hash;
                final_audit_entity.antecedent_audit_log_id = entity.audit_log_id.ok_or("Entity must have audit_log_id for deletion")?;
                
                final_audit_entity.audit_log_id = Some(audit_log_id);
                final_audit_entity.hash = 0;
                
                let final_hash = hash_as_i64(&final_audit_entity)?;
                final_audit_entity.hash = final_hash;
                
                // 3. Insert final audit record
                sqlx::query(
                    r#"
                    INSERT INTO {table_name}_audit
                    ({field_list}, hash, audit_log_id, antecedent_hash, antecedent_audit_log_id)
                    VALUES ({value_placeholders})
                    "#,
                )
                // ... bind all fields
                .execute(&mut **transaction)
                .await?;
                
                // 4. Delete from main table (index deleted via CASCADE)
                let result = sqlx::query(
                    r#"
                    DELETE FROM {table_name} WHERE id = $1
                    "#,
                )
                .bind(entity.id)
                .execute(&mut **transaction)
                .await?;
                
                rows_affected += result.rows_affected() as usize;
                
                // 5. Create audit link
                let audit_link = AuditLinkModel {
                    audit_log_id,
                    entity_id: entity.id,
                    entity_type: EntityType::{Entity},
                };
                sqlx::query(
                    r#"
                    INSERT INTO audit_link (audit_log_id, entity_id, entity_type)
                    VALUES ($1, $2, $3)
                    "#,
                )
                .bind(audit_link.audit_log_id)
                .bind(audit_link.entity_id)
                .bind(audit_link.entity_type)
                .execute(&mut **transaction)
                .await?;
            }
        }
        
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

**Key Points**:
- Load entities first to get final state
- Create final audit record with antecedent tracking
- Insert audit record FIRST
- Create audit link
- Remove from BOTH index cache AND main cache

---

## Database Schema

```sql
-- Migration: Initial {Entity} Schema with Audit and Main Cache Support
-- Description: Creates {entity}-related tables with audit trail and cache notification triggers

-- Enum Types (if any)
CREATE TYPE IF NOT EXISTS {enum_name} AS ENUM ('Variant1', 'Variant2');

-- Enum for auditable entity types
CREATE TYPE entity_type AS ENUM ('LOCATION', ...);

-- Main {Entity} Table
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

-- {Entity} Audit Table
CREATE TABLE IF NOT EXISTS {table_name}_audit (
    -- All entity fields are duplicated here for a complete snapshot
    id UUID NOT NULL,
    field1 VARCHAR(...),
    field2 VARCHAR(...),
    -- ... all other entity fields
    
    -- Audit-specific fields
    hash BIGINT NOT NULL,
    audit_log_id UUID NOT NULL REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    
    -- Composite primary key ensures one audit entry per entity version
    PRIMARY KEY (id, audit_log_id)
);

-- Index on id for efficient audit queries by entity ID
CREATE INDEX IF NOT EXISTS idx_{table_name}_audit_id
    ON {table_name}_audit(id);

-- Audit Link Table
CREATE TABLE IF NOT EXISTS audit_link (
    audit_log_id UUID NOT NULL REFERENCES audit_log(id),
    entity_id UUID NOT NULL,
    entity_type entity_type NOT NULL,
    PRIMARY KEY (audit_log_id, entity_id)
);

-- Index for audit_link table
CREATE INDEX IF NOT EXISTS idx_audit_link_audit_log_id ON audit_link(audit_log_id);
```

**Key Points**:
- TWO triggers: one for index cache, one for main cache
- Index trigger on `{table_name}_idx` table
- Main trigger on `{table_name}` table (covers both cache and audit)
- Audit table has composite primary key `(id, audit_log_id)`
- Audit table does NOT have `ON DELETE CASCADE`

---

## Repository Factory Pattern

### Factory Implementation

```rust
use std::sync::Arc;
use parking_lot::RwLock as ParkingRwLock;
use postgres_unit_of_work::UnitOfWorkSession;
use postgres_index_cache::{CacheNotificationListener, IndexCacheHandler, MainModelCacheHandler, MainModelCache, CacheConfig, EvictionPolicy};
use std::time::Duration;
use business_core_db::models::{module}::{entity}::{Entity}IdxModel, {Entity}Model};
use super::{Entity}RepositoryImpl;

/// Factory for creating {module} module repositories with audit and main cache
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

## Testing Patterns

### Standard Test Cases

```rust
#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_create_batch_updates_caches_and_audit() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let {entity}_repo = &ctx.{module}_repos().{entity}_repository;

        // Create audit log
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        let items = vec![/* test data */];
        let saved = {entity}_repo.create_batch(items, audit_log.id).await?;

        // Verify entities are in main cache
        let main_cache = {entity}_repo.{entity}_cache.read().await;
        for item in &saved {
            assert!(main_cache.contains(&item.id), "Entity should be in main cache");
            assert!(item.hash != 0, "Entity should have computed hash");
            assert!(item.audit_log_id.is_some(), "Entity should have audit_log_id");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_load_batch_uses_cache() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let {entity}_repo = &ctx.{module}_repos().{entity}_repository;

        // Create entities
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        
        let items = vec![/* test data */];
        let saved = {entity}_repo.create_batch(items, audit_log.id).await?;
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
    async fn test_update_batch_updates_caches_and_audit() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let {entity}_repo = &ctx.{module}_repos().{entity}_repository;

        // Create entities
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        
        let items = vec![/* test data */];
        let mut saved = {entity}_repo.create_batch(items, audit_log.id).await?;
        
        // Store original hash
        let original_hash = saved[0].hash;
        
        // Update entity
        let update_audit_log = create_test_audit_log();
        audit_log_repo.create(&update_audit_log).await?;
        
        saved[0].field1 = /* updated value */;
        let updated = {entity}_repo.update_batch(saved, update_audit_log.id).await?;

        // Verify updated entity in cache with new hash
        let main_cache = {entity}_repo.{entity}_cache.read().await;
        let cached = main_cache.get(&updated[0].id);
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().field1, updated[0].field1);
        assert_ne!(cached.unwrap().hash, original_hash, "Hash should change after update");
        assert_eq!(cached.unwrap().antecedent_hash, original_hash, "Antecedent hash should match original");

        Ok(())
    }

    #[tokio::test]
    async fn test_delete_batch_removes_from_caches_and_creates_audit() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let {entity}_repo = &ctx.{module}_repos().{entity}_repository;

        // Create entities
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        
        let items = vec![/* test data */];
        let saved = {entity}_repo.create_batch(items, audit_log.id).await?;
        let ids: Vec<Uuid> = saved.iter().map(|i| i.id).collect();

        // Delete entities
        let delete_audit_log = create_test_audit_log();
        audit_log_repo.create(&delete_audit_log).await?;
        
        let deleted_count = {entity}_repo.delete_batch(&ids, delete_audit_log.id).await?;
        assert_eq!(deleted_count, ids.len());

        // Verify removed from both caches
        let idx_cache = {entity}_repo.{entity}_idx_cache.read().await;
        let main_cache = {entity}_repo.{entity}_cache.read().await;
        for id in &ids {
            assert!(!idx_cache.contains_primary(id), "Entity should be removed from index cache");
            assert!(!main_cache.contains(id), "Entity should be removed from main cache");
        }

        Ok(())
    }

    #[tokio::test]
    async fn test_load_audits() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let {entity}_repo = &ctx.{module}_repos().{entity}_repository;

        // Create initial entity
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;
        
        let {entity} = create_test_{entity}(/* params */);
        let {entity}_id = {entity}.id;
        let mut saved = {entity}_repo.create_batch(vec![{entity}], audit_log.id).await?;

        // Update entity multiple times to create audit history
        for i in 1..=3 {
            let update_audit_log = create_test_audit_log();
            audit_log_repo.create(&update_audit_log).await?;
            
            let mut updated = saved[0].clone();
            updated.field_name = /* modify field */;
            saved = {entity}_repo.update_batch(vec![updated], update_audit_log.id).await?;
        }

        // Load audit history with pagination
        let page = {entity}_repo.load_audits({entity}_id, PageRequest::new(2, 0)).await?;

        assert_eq!(page.total, 4); // 1 create + 3 updates
        assert_eq!(page.items.len(), 2);
        assert!(page.has_more());

        Ok(())
    }
}
```

### Cache Notification Test (REQUIRED)

```rust
#[tokio::test]
async fn test_{entity}_insert_triggers_cache_notifications() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
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

    // Create audit log
    let audit_log = create_test_audit_log();
    sqlx::query("INSERT INTO audit_log (id, updated_at, updated_by_person_id) VALUES ($1, $2, $3)")
        .bind(audit_log.id)
        .bind(audit_log.updated_at)
        .bind(audit_log.updated_by_person_id)
        .execute(&**pool)
        .await
        .expect("Failed to insert audit log");

    // Prepare entity with hash
    let mut test_entity_for_hashing = test_entity.clone();
    test_entity_for_hashing.hash = 0;
    test_entity_for_hashing.audit_log_id = Some(audit_log.id);
    let computed_hash = business_core_db::utils::hash_as_i64(&test_entity_for_hashing).unwrap();
    
    let final_entity = {Entity}Model {
        hash: computed_hash,
        audit_log_id: Some(audit_log.id),
        ..test_entity
    };

    // Insert the entity record directly into database
    sqlx::query("INSERT INTO {table_name} ({field_list}, hash, audit_log_id) VALUES ({placeholders})")
        // ... bind all fields from final_entity
        .execute(&**pool)
        .await
        .expect("Failed to insert {entity}");

    // Insert the index record directly into database
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
        main_cache.contains(&final_entity.id),
        "{Entity} should be in main cache after insert"
    );
    drop(main_cache);

    // Delete the record from database
    sqlx::query("DELETE FROM {table_name} WHERE id = $1")
        .bind(final_entity.id)
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
        !main_cache.contains(&final_entity.id),
        "{Entity} should be removed from main cache after delete"
    );

    Ok(())
}
```

**Key Points**:
- Tests BOTH index cache AND main cache notifications
- Verifies direct database operations trigger cache updates
- Tests both INSERT and DELETE operations
- Includes audit hash computation for realistic testing

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

- [ ] All base template validations (from entity_with_index.md)
- [ ] Main entity model has `hash`, `audit_log_id`, `antecedent_hash`, and `antecedent_audit_log_id` fields
- [ ] Main entity model implements `HasPrimaryKey` trait
- [ ] Auditable trait is implemented
- [ ] Repository has both `{entity}_idx_cache` and `{entity}_cache` fields
- [ ] Repository constructor accepts both caches
- [ ] `TransactionAware` commits/rollbacks both caches
- [ ] Hash computation uses the entity (with `hash=0`) pattern
- [ ] Audit record is inserted **before** entity modification
- [ ] CREATE updates both index and main cache
- [ ] CREATE computes hash and creates audit record
- [ ] LOAD checks main cache first before database query
- [ ] LOAD adds cache misses to main cache
- [ ] UPDATE checks for actual changes using hash comparison
- [ ] UPDATE updates antecedent fields for audit chain
- [ ] UPDATE updates both index and main cache
- [ ] DELETE creates final audit record before deletion
- [ ] DELETE removes from both index and main cache
- [ ] Factory initializes both caches with configuration
- [ ] Factory registers TWO handlers: index and main
- [ ] Main cache handler uses main table name (not `_idx`)
- [ ] Database has TWO triggers: one on `{table_name}_idx`, one on `{table_name}`
- [ ] Audit table has composite primary key `(id, audit_log_id)`
- [ ] Audit table does NOT have `ON DELETE CASCADE`
- [ ] Audit table has index on `id` column
- [ ] LoadAudits trait is implemented with pagination support
- [ ] Tests verify main cache synchronization
- [ ] Tests verify audit trail integrity
- [ ] Cache notification test covers both caches
- [ ] Cache statistics are accessible for monitoring
- [ ] **One finder method is created for each secondary index field**
- [ ] **Comprehensive tests are implemented for ALL repository methods**

---

## Best Practices

### ✅ DO:

- Always create audit log entry before calling create/update operations
- Insert audit record before modifying the main entity
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
- Use the full entity model for audit records
- Set `entity.hash` to zero before computing the hash
- Use CBOR serialization for hash computation
- Store antecedent hash and audit_log_id for chain verification
- Test audit trail integrity and hash verification
- **Generate comprehensive tests for EVERY repository method implemented**
- **Include cache notification tests to verify database triggers work correctly**
- **Verify cache synchronization in all tests**

### ❌ DON'T:

- Preload main cache at startup (entities added on-demand)
- Use excessively large cache sizes (consider memory constraints)
- Skip cache updates after database operations
- Hold transaction and cache locks simultaneously
- Query database without checking cache first
- Forget to configure eviction policy
- Use same handler for both index and main cache
- Create trigger only on index table (need both)
- Modify main entity before creating audit record
- Skip hash computation or forget to set hash to zero before hashing
- Use different serialization for hashing
- Forget to update audit_log_id in the entity
- Create a dedicated audit model (all fields are in the main entity model)
- Use version numbers (audit records are keyed by id and audit_log_id)
- **Skip test generation for any repository method**
- **Forget to test cache notification mechanism**

---

## When to Use This Template

Use this template for entities that:
- ✅ Need secondary index lookups (hash-based or UUID-based)
- ✅ Are frequently accessed by ID after initial lookup
- ✅ Benefit from full entity caching for performance
- ✅ Require complete audit trail with cryptographic verification
- ✅ Have moderate data volume (can fit in configured cache size)
- ✅ Require cross-node cache synchronization
- ✅ Need immutable history tracking

**Do NOT use** this template if:
- ❌ Entities are too large to cache efficiently
- ❌ Data volume exceeds reasonable cache size limits
- ❌ Access patterns don't benefit from caching
- ❌ Audit trail is not required
- ❌ Only need index lookups without full entity data

For simpler cases, use:
- [Entity with Index](entity_with_index.md) - no audit, no main cache
- [Entity with Index and Audit](entity_with_index_and_audit.md) - audit but no main cache
- [Entity with Index and Main Cache](entity_with_index_and_maincache.md) - main cache but no audit

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

## Audit Chain Verification

The entity's audit fields enable cryptographic audit chain verification:

```rust
// Entity Model contains audit fields
pub struct {Entity}Model {
    pub hash: i64,                       // Hash of the entity (with hash=0)
    pub audit_log_id: Option<Uuid>,     // Current audit log reference
    pub antecedent_hash: i64,           // Previous record's hash
    pub antecedent_audit_log_id: Uuid,  // Previous record's audit_log_id
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
- **Audit Template**: [Entity with Index and Audit](entity_with_index_and_audit.md)
- **Main Cache Template**: [Entity with Index and Main Cache](entity_with_index_and_maincache.md)
- **Main Cache Implementation**: `postgres-index-cache/src/main_model_cache.rs`
- **Transaction-Aware Cache**: `postgres-index-cache/src/transaction_aware_main_model_cache.rs`
- **Audit Traits**: `business-core/business-core-db/src/models/auditable.rs`
- **Hash Library**: [twox-hash](https://docs.rs/twox-hash/)
- **CBOR Library**: [ciborium](https://docs.rs/ciborium/)

---

## License

Same as business-core project