-- Cleanup: Initial Audit Schema
-- Description: Removes all artifacts created by 001_initial_schema_audit.sql

-- Drop indexes first
DROP INDEX IF EXISTS idx_audit_log_updated_by_person_id;
DROP INDEX IF EXISTS idx_audit_log_updated_at;

-- Drop tables
DROP TABLE IF EXISTS audit_log CASCADE;

-- Drop audit_link table and its index
DROP INDEX IF EXISTS idx_audit_link_audit_log_id;
DROP TABLE IF EXISTS audit_link CASCADE;

-- Drop audit_entity_type enum
DROP TYPE IF EXISTS audit_entity_type;