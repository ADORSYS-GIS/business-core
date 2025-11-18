# Business Day Entity with Index and Main Cache

Generate a complete implementation of the BusinessDay entity following the skill documented in `docs/skills/entity_template/entity_with_index_and_maincache.md`.

## Entity Details

- **Entity Name**: BusinessDay
- **Module**: calendar
- **Table Name**: calendar_business_day
- **Description**: Manages business day configurations per country/subdivision with date tracking and holiday information

## Index Fields

The entity has the following index fields (in addition to `id`):

```rust
pub country_id: Option<Uuid>,
pub country_subdivision_id: Option<Uuid>,
pub date_hash: i64,
```

## Entity Model Fields

Sample data is taken from `business-core-db/src/models/calendar/business_day_example.rs`:

```rust
pub struct BusinessDayModel {
    pub id: Uuid,
    pub country_id: Option<Uuid>,
    pub country_subdivision_id: Option<Uuid>,
    pub date: NaiveDate,
    pub weekday: Weekday,
    pub is_business_day: bool,
    pub is_weekend: bool,
    pub weekend_day_01: Option<Uuid>,
    pub is_holiday: bool,
    pub holiday_type: Option<HolidayType>,
    pub holiday_name: Option<HeaplessString<50>>,
    pub day_scope: DayScope,
}
```

## Required Artifacts

Following the skill `docs/skills/entity_template/entity_with_index_and_maincache.md`, generate:

### 1. Database Model
- Location: `business-core-db/src/models/calendar/business_day.rs`
- Create from: `business-core-db/src/models/calendar/business_day_example.rs`
- Status: BusinessDayModel, Weekday enum, HolidayType enum, and DayScope enum to be moved from example file

### 2. Index Model
- Location: `business-core-db/src/models/calendar/business_day.rs`
- Add `BusinessDayIdxModel` with fields:
  - `id: Uuid`
  - `country_id: Option<Uuid>`
  - `country_subdivision_id: Option<Uuid>`
  - `date_hash: i64`
- Implement `IndexAware` trait for `BusinessDayModel`
- Implement `HasPrimaryKey` trait for `BusinessDayModel`
- Implement `HasPrimaryKey` trait for `BusinessDayIdxModel`

### 3. Repository Implementation
- Location: `business-core-postgres/src/repository/calendar/business_day_repository/`
- Files needed:
  - `repo_impl.rs` - Repository struct with index and main cache
  - `create_batch.rs` - Create implementation updating both caches
  - `load_batch.rs` - Load implementation checking main cache first
  - `update_batch.rs` - Update implementation updating both caches
  - `delete_batch.rs` - Delete implementation removing from both caches
  - `exist_by_ids.rs` - Check existence using index cache
  - `find_by_country_id.rs` - Finder method for country_id index
  - `find_by_country_subdivision_id.rs` - Finder method for country_subdivision_id index
  - `find_by_date_hash.rs` - Finder method for date_hash index
  - `mod.rs` - Module exports
  - `test_utils.rs` - Test helper functions

### 4. Factory Implementation
- Location: `business-core-postgres/src/repository/calendar/factory.rs`
- Update CalendarRepoFactory to include:
  - business_day_idx_cache field
  - business_day_cache field (with CacheConfig: max 1000 entities, LRU eviction, 1 hour TTL)
  - Register IndexCacheHandler for "calendar_business_day_idx"
  - Register MainModelCacheHandler for "calendar_business_day"
  - build_business_day_repo method

### 5. Database Migration
- Location: `business-core-postgres/migrations/0XX_initial_schema_calendar_business_day.sql`
- Create tables:
  - `calendar_business_day` - Main entity table with all fields
  - `calendar_business_day_idx` - Index table with id, country_id, country_subdivision_id, date_hash
- Create TWO triggers:
  - Trigger on `calendar_business_day_idx` for index cache notifications
  - Trigger on `calendar_business_day` for main cache notifications

### 6. Cleanup Script
- Location: `business-core-postgres/cleanup/0XX_cleanup_calendar_business_day.sql`
- Drop both tables and triggers

### 7. Module Integration
- Update `business-core-db/src/models/calendar/mod.rs` to export BusinessDayModel, BusinessDayIdxModel, DayScope, HolidayType, and Weekday
- Update `business-core-postgres/src/repository/calendar/mod.rs` to export business_day_repository

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

1. **find_by_country_id** - Returns `Vec<BusinessDayIdxModel>` for a given country_id
2. **find_by_country_subdivision_id** - Returns `Vec<BusinessDayIdxModel>` for a given country_subdivision_id
3. **find_by_date_hash** - Returns `Vec<BusinessDayIdxModel>` for a given date_hash

## Testing Requirements

Create comprehensive tests in `business-core-postgres/src/repository/calendar/business_day_repository/mod.rs`:

- `test_create_batch_updates_main_cache` - Verify entities are added to main cache
- `test_load_batch_uses_cache` - Verify cache hits on subsequent loads
- `test_update_batch_updates_main_cache` - Verify cache updates on entity updates
- `test_delete_batch_removes_from_main_cache` - Verify cache removal on delete
- `test_business_day_insert_triggers_main_cache_notification` - Verify both cache notifications work
- `test_find_by_country_id` - Test country_id finder method
- `test_find_by_country_subdivision_id` - Test country_subdivision_id finder method
- `test_find_by_date_hash` - Test date_hash finder method

## Implementation Notes

1. The BusinessDayModel structure exists in the example file, extract and enhance it
2. Follow the exact patterns from the skill document for cache management
3. Ensure both index cache and main cache are updated atomically in all operations
4. Release transaction lock before updating caches
5. Check main cache before querying database in load operations
6. Register both IndexCacheHandler and MainModelCacheHandler with correct table names
7. Create triggers on both the main table and the index table
8. The `date_hash` field should be calculated from the `date` field for indexing purposes

## Database Field Mappings

### Main Table Fields
- `id` - UUID PRIMARY KEY
- `country_id` - UUID (nullable)
- `country_subdivision_id` - UUID (nullable)
- `date` - DATE
- `weekday` - weekday enum
- `is_business_day` - BOOLEAN
- `is_weekend` - BOOLEAN
- `weekend_day_01` - UUID (nullable)
- `is_holiday` - BOOLEAN
- `holiday_type` - holiday_type enum (nullable)
- `holiday_name` - VARCHAR(50) (nullable)
- `day_scope` - day_scope enum

### Index Table Fields
- `id` - UUID PRIMARY KEY (references main table)
- `country_id` - UUID (nullable)
- `country_subdivision_id` - UUID (nullable)
- `date_hash` - BIGINT

## Success Criteria

- [ ] All CRUD operations work correctly
- [ ] Both caches are synchronized after all operations
- [ ] Database triggers notify both caches correctly
- [ ] Cache hit rates are measurable and logged
- [ ] Finder methods return correct results using index cache
- [ ] All tests pass
- [ ] Transaction rollback clears staged cache changes
- [ ] Date hash calculation is consistent and correct

## Post-Generation Cleanup

After successful generation of all artifacts:
- Delete the example file: `business-core-db/src/models/calendar/business_day_example.rs`