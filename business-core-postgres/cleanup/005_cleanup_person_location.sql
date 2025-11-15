-- Cleanup: Initial Location Schema
-- Description: Removes all artifacts created by 005_initial_schema_person_location.sql

-- Drop trigger first
DROP TRIGGER IF EXISTS location_idx_notify ON location_idx;

-- Drop tables (index table first due to foreign key constraint)
DROP TABLE IF EXISTS location_idx CASCADE;
DROP TABLE IF EXISTS location_audit CASCADE;
DROP TABLE IF EXISTS location CASCADE;

-- Drop the custom type
DROP TYPE IF EXISTS location_type;