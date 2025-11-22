-- Cleanup: Initial Locality Schema
-- Description: Removes all artifacts created by 004_initial_schema_person_locality.sql

-- Drop trigger first
DROP TRIGGER IF EXISTS locality_idx_notify ON locality_idx;

-- Drop tables (locality_idx first due to foreign key constraint)
DROP TABLE IF EXISTS locality_idx CASCADE;
DROP TABLE IF EXISTS locality CASCADE;