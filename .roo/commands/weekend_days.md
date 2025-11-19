# Weekend Days Entity with Index and Main Cache

Generate a complete implementation of the WeekendDays entity following the skill documented in `docs/skills/entity_template/entity_with_index_and_maincache.md`.

## Entity Details

- **Entity Name**: WeekendDays
- **Module**: calendar
- **Table Name**: calendar_weekend_days
- **Description**: Manages weekend day configurations per country/subdivision with effective date ranges

## Index Fields

The entity has the following index fields (in addition to `id`):

```rust
pub country_id: Option<Uuid>,
pub country_subdivision_id: Option<Uuid>,
```

## Entity Model Fields

```rust
pub struct WeekendDaysModel {
    pub id: Uuid,
    pub country_id: Option<Uuid>,
    pub country_subdivision_id: Option<Uuid>,
    pub weekend_day_01: Option<Weekday>,
    pub weekend_day_02: Option<Weekday>,
    pub weekend_day_03: Option<Weekday>,
    pub weekend_day_04: Option<Weekday>,
    pub weekend_day_05: Option<Weekday>,
    pub weekend_day_06: Option<Weekday>,
    pub weekend_day_07: Option<Weekday>,
    pub effective_date: NaiveDate,
    pub expiry_date: Option<NaiveDate>,
}
```

## Required Artifacts

Following the skill `docs/skills/entity_template/entity_with_index_and_maincache.md`, generate:

### 1. Database Model
- Location: `business-core-db/src/models/calendar/weekend_days.rs`
- Create from: `business-core-db/src/models/calendar/weekend_days_example.rs` (will be deleted after)
- Status: WeekendDaysModel and Weekday enum to be moved from example file

### 2. Index Model
- Location: `business-core-db/src/models/calendar/weekend_days.rs`
- Add `WeekendDaysIdxModel` with fields:
  - `id: Uuid`
  - `country_id: Option<Uuid>`
  - `country_subdivision_id: Option<Uuid>`
- Implement `IndexAware` trait for `WeekendDaysModel`
- Implement `HasPrimaryKey` trait for `WeekendDaysModel`
- Implement `HasPrimaryKey` trait for `WeekendDaysIdxModel`

### 3. Repository Implementation
- Location: `business-core-postgres/src/repository/calendar/weekend_days_repository/`
- Files needed:
  - `repo_impl.rs` - Repository struct with index and main cache
  - `create_batch.rs` - Create implementation updating both caches
  - `load_batch.rs` - Load implementation checking main cache first
  - `update_batch.rs` - Update implementation updating both caches
  - `delete_batch.rs` - Delete implementation removing from both caches
  - `exist_by_ids.rs` - Check existence using index cache
  - `find_by_country_id.rs` - Finder method for country_id index
  - `find_by_country_subdivision_id.rs` - Finder method for country_subdivision_id index
  - `mod.rs` - Module exports
  - `test_utils.rs` - Test helper functions

### 4. Factory Implementation
- Location: `business-core-postgres/src/repository/calendar/factory.rs`
- Update CalendarRepoFactory to include:
  - weekend_days_idx_cache field
  - weekend_days_cache field (with CacheConfig: max 1000 entities, LRU eviction, 1 hour TTL)
  - Register IndexCacheHandler for "calendar_weekend_days_idx"
  - Register MainModelCacheHandler for "calendar_weekend_days"
  - build_weekend_days_repo method

### 5. Database Migration
- Location: `business-core-postgres/migrations/0XX_initial_schema_calendar_weekend_days.sql`
- Create tables:
  - `calendar_weekend_days` - Main entity table with all fields
  - `calendar_weekend_days_idx` - Index table with id, country_id, country_subdivision_id
- Create TWO triggers:
  - Trigger on `calendar_weekend_days_idx` for index cache notifications
  - Trigger on `calendar_weekend_days` for main cache notifications

### 6. Cleanup Script
- Location: `business-core-postgres/cleanup/0XX_cleanup_calendar_weekend_days.sql`
- Drop both tables and triggers

### 7. Module Integration
- Update `business-core-db/src/models/calendar/mod.rs` to export WeekendDaysModel, WeekendDaysIdxModel, and Weekday
- Update `business-core-postgres/src/repository/calendar/mod.rs` to export weekend_days_repository

## Cache Configuration

```rust
let cache_config = CacheConfig::new(
    1000,  // Max 1000 entities in cache
    EvictionPolicy::LRU,  // Least Recently Used eviction
)
.with_ttl(Duration::from_secs(3600)); // 1 hour TTL
```

## Finder Methods Required

Per the skill documentation, create finder methods for each secondary index field:

1. **find_by_country_id** - Returns `Vec<WeekendDaysIdxModel>` for a given country_id
2. **find_by_country_subdivision_id** - Returns `Vec<WeekendDaysIdxModel>` for a given country_subdivision_id

## Testing Requirements

Create comprehensive tests in `business-core-postgres/src/repository/calendar/weekend_days_repository/mod.rs`:

- `test_create_batch_updates_main_cache` - Verify entities are added to main cache
- `test_load_batch_uses_cache` - Verify cache hits on subsequent loads
- `test_update_batch_updates_main_cache` - Verify cache updates on entity updates
- `test_delete_batch_removes_from_main_cache` - Verify cache removal on delete
- `test_weekend_days_insert_triggers_main_cache_notification` - Verify both cache notifications work
- `test_find_by_country_id` - Test country_id finder method
- `test_find_by_country_subdivision_id` - Test country_subdivision_id finder method

## Implementation Notes

1. The WeekendDaysModel already exists, so focus on adding the index model and repository implementation
2. Follow the exact patterns from the skill document for cache management
3. Ensure both index cache and main cache are updated atomically in all operations
4. Release transaction lock before updating caches
5. Check main cache before querying database in load operations
6. Register both IndexCacheHandler and MainModelCacheHandler with correct table names
7. Create triggers on both the main table and the index table

## Database Field Mappings

### Main Table Fields
- `id` - UUID PRIMARY KEY
- `country_id` - UUID (nullable)
- `country_subdivision_id` - UUID (nullable)
- `weekend_day_01` through `weekend_day_07` - weekday enum (nullable)
- `effective_date` - DATE
- `expiry_date` - DATE (nullable)

### Index Table Fields
- `id` - UUID PRIMARY KEY (references main table)
- `country_id` - UUID (nullable)
- `country_subdivision_id` - UUID (nullable)

## Success Criteria

- [ ] All CRUD operations work correctly
- [ ] Both caches are synchronized after all operations
- [ ] Database triggers notify both caches correctly
- [ ] Cache hit rates are measurable and logged
- [ ] Finder methods return correct results using index cache
- [ ] All tests pass
- [ ] Transaction rollback clears staged cache changes

## Post-Generation Cleanup

After successful generation of all artifacts:
- Delete the example file: `business-core-db/src/models/calendar/weekend_days_example.rs`