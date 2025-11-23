-- Cleanup: Initial Named Schema
-- Description: Removes all artifacts created by 002_initial_schema_named.sql

-- Drop trigger first
DROP TRIGGER IF EXISTS named_idx_notify ON named_idx;

-- Drop tables (index table first due to foreign key constraint, then audit, then main table)
DROP TABLE IF EXISTS named_idx CASCADE;
DROP TABLE IF EXISTS named_audit CASCADE;
DROP TABLE IF EXISTS named CASCADE;

-- Drop named_entity_type enum
DROP TYPE IF EXISTS named_entity_type;