-- Cleanup: Initial Named Schema
-- Description: Removes all artifacts created by 019_initial_schema_named.sql

-- Drop tables (audit table first, then main table)
DROP TABLE IF EXISTS named_audit CASCADE;
DROP TABLE IF EXISTS named CASCADE;