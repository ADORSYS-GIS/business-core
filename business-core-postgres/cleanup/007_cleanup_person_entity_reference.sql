-- Cleanup: Initial Entity Reference Schema
-- Description: Removes all artifacts created by 007_initial_schema_person_entity_reference.sql

-- Drop trigger first
DROP TRIGGER IF EXISTS entity_reference_idx_notify ON entity_reference_idx;

-- Drop tables (index table first due to foreign key constraint)
DROP TABLE IF EXISTS entity_reference_idx CASCADE;
DROP TABLE IF EXISTS entity_reference_audit CASCADE;
DROP TABLE IF EXISTS entity_reference CASCADE;

-- Drop the custom type
DROP TYPE IF EXISTS person_entity_type;