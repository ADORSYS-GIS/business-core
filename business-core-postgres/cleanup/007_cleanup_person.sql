-- Cleanup: Initial Person Schema
-- Description: Removes all artifacts created by 006_initial_schema_person.sql

-- Drop trigger first
DROP TRIGGER IF EXISTS person_idx_notify ON person_idx;

-- Drop tables (index table first due to foreign key constraint)
DROP TABLE IF EXISTS person_idx CASCADE;
DROP TABLE IF EXISTS person_audit CASCADE;
DROP TABLE IF EXISTS person CASCADE;

-- Drop the custom types
DROP TYPE IF EXISTS person_type;
DROP TYPE IF EXISTS identity_type;
DROP TYPE IF EXISTS risk_rating;
DROP TYPE IF EXISTS person_status;