# Date Calculation Rules Entity with Index and Main Cache

Generate a complete implementation of the DateCalculationRules entity following the skill documented in `docs/skills/entity_template/entity_with_index_and_maincache.md`.

## Entity Details

- **Entity Name**: DateCalculationRules
- **Module**: calendar
- **Table Name**: calendar_date_calculation_rules
- **Description**: Manages date calculation rules per country/subdivision with rule-based date shifting and maturity calculations

## Index Fields

The entity has the following index fields (in addition to `id`):

```rust
pub country_id: Option<Uuid>,
pub country_subdivision_id: Option<Uuid>,
pub rule_name_hash: i64,
```

## Entity Model Fields

Sample data is taken from `business-core-db/src/models/calendar/date_calculation_rules_example.rs`:

```rust
pub struct DateCalculationRulesModel {
    pub id: Uuid,
    pub country_id: Uuid,
    pub country_subdivision_id: Option<Uuid>,
    pub rule_name: HeaplessString<50>,
    pub rule_purpose: DateRulePurpose,
    pub default_shift_rule: DateShiftRule,
    pub weekend_days_id: Option<Uuid>,
    pub priority: i32,
    pub is_active: bool,
    pub effective_date: NaiveDate,
    pub expiry_date: Option<NaiveDate>,
}
```

## Required Artifacts

Following the skill `docs/skills/entity_template/entity_with_index_and_maincache.md`, generate:

### 1. Database Model
- Location: `business-core-db/src/models/calendar/date_calculation_rules.rs`
- Create from: `business-core-db/src/models/calendar/date_calculation_rules_example.rs`
- Status: DateCalculationRulesModel, DateRulePurpose enum, and DateShiftRule enum to be moved from example file

### 2. Index Model
- Location: `business-core-db/src/models/calendar/date_calculation_rules.rs`
- Add `DateCalculationRulesIdxModel` with fields:
  - `id: Uuid`
  - `country_id: Option<Uuid>`
  - `country_subdivision_id: Option<Uuid>`
  - `rule_name_hash: i64`
- Implement `IndexAware` trait for `DateCalculationRulesModel`
- Implement `HasPrimaryKey` trait for `DateCalculationRulesModel`
- Implement `HasPrimaryKey` trait for `DateCalculationRulesIdxModel`

### 3. Repository Implementation
- Location: `business-core-postgres/src/repository/calendar/date_calculation_rules_repository/`
- Files needed:
  - `repo_impl.rs` - Repository struct with index and main cache
  - `create_batch.rs` - Create implementation updating both caches
  - `load_batch.rs` - Load implementation checking main cache first
  - `update_batch.rs` - Update implementation updating both caches
  - `delete_batch.rs` - Delete implementation removing from both caches
  - `exist_by_ids.rs` - Check existence using index cache
  - `find_by_country_id.rs` - Finder method for country_id index
  - `find_by_country_subdivision_id.rs` - Finder method for country_subdivision_id index
  - `find_by_rule_name_hash.rs` - Finder method for rule_name_hash index
  - `mod.rs` - Module exports
  - `test_utils.rs` - Test helper functions

### 4. Factory Implementation
- Location: `business-core-postgres/src/repository/calendar/factory.rs`
- Update CalendarRepoFactory to include:
  - date_calculation_rules_idx_cache field
  - date_calculation_rules_cache field (with CacheConfig: max 1000 entities, LRU eviction, 1 hour TTL)
  - Register IndexCacheHandler for "calendar_date_calculation_rules_idx"
  - Register MainModelCacheHandler for "calendar_date_calculation_rules"
  - build_date_calculation_rules_repo method

### 5. Database Migration
- Location: `business-core-postgres/migrations/0XX_initial_schema_calendar_date_calculation_rules.sql`
- Create tables:
  - `calendar_date_calculation_rules` - Main entity table with all fields
  - `calendar_date_calculation_rules_idx` - Index table with id, country_id, country_subdivision_id, rule_name_hash
- Create TWO triggers:
  - Trigger on `calendar_date_calculation_rules_idx` for index cache notifications
  - Trigger on `calendar_date_calculation_rules` for main cache notifications

### 6. Cleanup Script
- Location: `business-core-postgres/cleanup/0XX_cleanup_calendar_date_calculation_rules.sql`
- Drop both tables and triggers

### 7. Module Integration
- Update `business-core-db/src/models/calendar/mod.rs` to export DateCalculationRulesModel, DateCalculationRulesIdxModel, DateRulePurpose, and DateShiftRule
- Update `business-core-postgres/src/repository/calendar/mod.rs` to export date_calculation_rules_repository

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

1. **find_by_country_id** - Returns `Vec<DateCalculationRulesIdxModel>` for a given country_id
2. **find_by_country_subdivision_id** - Returns `Vec<DateCalculationRulesIdxModel>` for a given country_subdivision_id
3. **find_by_rule_name_hash** - Returns `Vec<DateCalculationRulesIdxModel>` for a given rule_name_hash

## Testing Requirements

Create comprehensive tests in `business-core-postgres/src/repository/calendar/date_calculation_rules_repository/mod.rs`:

- `test_create_batch_updates_main_cache` - Verify entities are added to main cache
- `test_load_batch_uses_cache` - Verify cache hits on subsequent loads
- `test_update_batch_updates_main_cache` - Verify cache updates on entity updates
- `test_delete_batch_removes_from_main_cache` - Verify cache removal on delete
- `test_date_calculation_rules_insert_triggers_main_cache_notification` - Verify both cache notifications work
- `test_find_by_country_id` - Test country_id finder method
- `test_find_by_country_subdivision_id` - Test country_subdivision_id finder method
- `test_find_by_rule_name_hash` - Test rule_name_hash finder method

## Implementation Notes

1. The DateCalculationRulesModel structure exists in the example file, extract and enhance it
2. Follow the exact patterns from the skill document for cache management
3. Ensure both index cache and main cache are updated atomically in all operations
4. Release transaction lock before updating caches
5. Check main cache before querying database in load operations
6. Register both IndexCacheHandler and MainModelCacheHandler with correct table names
7. Create triggers on both the main table and the index table
8. The `rule_name_hash` field should be calculated from the `rule_name` field for indexing purposes

## Database Field Mappings

### Main Table Fields
- `id` - UUID PRIMARY KEY
- `country_id` - UUID
- `country_subdivision_id` - UUID (nullable)
- `rule_name` - VARCHAR(50)
- `rule_purpose` - date_rule_purpose enum
- `default_shift_rule` - date_shift_rule enum
- `weekend_days_id` - UUID (nullable)
- `priority` - INTEGER
- `is_active` - BOOLEAN
- `effective_date` - DATE
- `expiry_date` - DATE (nullable)

### Index Table Fields
- `id` - UUID PRIMARY KEY (references main table)
- `country_id` - UUID (nullable)
- `country_subdivision_id` - UUID (nullable)
- `rule_name_hash` - BIGINT

## Success Criteria

- [ ] All CRUD operations work correctly
- [ ] Both caches are synchronized after all operations
- [ ] Database triggers notify both caches correctly
- [ ] Cache hit rates are measurable and logged
- [ ] Finder methods return correct results using index cache
- [ ] All tests pass
- [ ] Transaction rollback clears staged cache changes
- [ ] Rule name hash calculation is consistent and correct

## Post-Generation Cleanup

After successful generation of all artifacts:
- Delete the example file: `business-core-db/src/models/calendar/date_calculation_rules_example.rs`