-- Cleanup: Initial Audit Schema
-- Description: Removes all artifacts created by 001_initial_schema_audit.sql

-- Drop indexes first
DROP INDEX IF EXISTS idx_audit_log_updated_by_person_id;
DROP INDEX IF EXISTS idx_audit_log_updated_at;

-- Drop tables
DROP TABLE IF EXISTS audit_log CASCADE;