-- Cleanup: Initial Reason Schema
-- Description: Removes all artifacts created by 009_initial_schema_reason_and_purpose_reason.sql

-- Drop trigger first
DROP TRIGGER IF EXISTS reason_idx_notify ON reason_idx;

-- Drop indexes
DROP INDEX IF EXISTS idx_reason_code_hash;
DROP INDEX IF EXISTS idx_reason_category_hash;
DROP INDEX IF EXISTS idx_reason_context_hash;
DROP INDEX IF EXISTS idx_reason_compliance_metadata;

-- Drop tables (index table first due to foreign key constraint)
DROP TABLE IF EXISTS reason_idx CASCADE;
DROP TABLE IF EXISTS reason CASCADE;

-- Drop enum types
DROP TYPE IF EXISTS reason_severity;
DROP TYPE IF EXISTS reason_context;
DROP TYPE IF EXISTS reason_category;