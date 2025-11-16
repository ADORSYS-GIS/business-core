# Create EntityReference Module

**Skill:** `docs/skills/entity_template/entity_with_index_and_audit.md`

**Module:** `entity_reference`

**Project:** `business-core`

**Core Data:** `business-core/business-core-db/src/models/person/entity_reference.rs`

**Description:**

This command creates the `entity_reference` module within the `business-core` project. It utilizes the `entity_with_index_and_audit` skill to generate the necessary components for the entity, including indexing and auditing features. The data model for the entity_reference entity is sourced from `business-core/business-core-db/src/models/person/entity_reference.rs`.

The entity_reference entity manages person-to-entity relationships with the following key features:
- **Indexable**: Supports hash-based lookups for person_id and reference_external_id_hash
- **Auditable**: Includes full audit trail with hash-based verification
- **Enum Support**: Uses RelationshipRole enum for entity_role field

**Entity Fields:**
- `id`: UUID (primary key)
- `person_id`: UUID (indexed, references PersonModel.id)
- `entity_role`: RelationshipRole enum (type of entity relationship)
- `reference_external_id`: HeaplessString<50> (indexed via hash)
- `reference_details_l1`: Option<HeaplessString<50>>
- `reference_details_l2`: Option<HeaplessString<50>>
- `reference_details_l3`: Option<HeaplessString<50>>
- `last_audit_log_id`: Option<UUID> (audit reference)

**Index Keys:**
- `person_id`: UUID (direct UUID index)
- `reference_external_id_hash`: i64 (hash of reference_external_id field)

**Custom Query Methods:**
- `find_ids_by_person_id(person_id: Uuid) -> Vec<Uuid>`
- `find_ids_by_reference_external_id_hash(reference_external_id_hash: i64) -> Vec<Uuid>`

**Audit Support:**
- The entity is auditable and requires audit_log_id for all create, update, and delete operations
- Implements hash-based verification for audit chain integrity
- Uses the same audit pattern as LocationModel

**Reference Template:**
Use `business-core-postgres/src/repository/person/location_repository` as the template for repository implementation since both entities are indexable and auditable.

**Testing Requirements:**
Comprehensive tests must be generated for each repository method implemented, following the same testing logic and patterns used in `business-core-postgres/src/repository/person/location_repository`. This includes:
- **create_batch**: Test entity creation with audit trail
- **load_batch**: Test entity loading by IDs
- **update_batch**: Test entity updates with audit chain verification
- **delete_batch**: Test entity deletion with final audit record
- **exist_by_ids**: Test existence checks via cache
- **Custom finder methods**: Test cache-based lookups for person_id and reference_external_id_hash
- **Cache notification test**: Test that direct database inserts/deletes trigger cache updates
- Each test should verify correct cache synchronization and audit trail integrity