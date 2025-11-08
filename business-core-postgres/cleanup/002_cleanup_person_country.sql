-- Cleanup: Initial Country Schema
-- Description: Removes all artifacts created by 002_initial_schema_person_country.sql

-- Drop trigger first
DROP TRIGGER IF EXISTS country_idx_notify ON country_idx;

-- Drop tables (country_idx first due to foreign key constraint)
DROP TABLE IF EXISTS country_idx CASCADE;
DROP TABLE IF EXISTS country CASCADE;