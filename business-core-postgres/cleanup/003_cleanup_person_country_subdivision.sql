-- Cleanup: Initial CountrySubdivision Schema
-- Description: Removes all artifacts created by 003_initial_schema_person_country_subdivision.sql

-- Drop trigger first
DROP TRIGGER IF EXISTS country_subdivision_idx_notify ON country_subdivision_idx;

-- Drop tables (country_subdivision_idx first due to foreign key constraint)
DROP TABLE IF EXISTS country_subdivision_idx CASCADE;
DROP TABLE IF EXISTS country_subdivision CASCADE;