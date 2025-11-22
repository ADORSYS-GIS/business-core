-- Cleanup: Remove ActivityLog Schema
-- Description: Drops person_activity_log-related tables

-- Drop audit table first (no FK constraint to main table)
DROP TABLE IF EXISTS person_activity_log_audit;

-- Drop main table
DROP TABLE IF EXISTS person_activity_log;

-- Note: We don't drop the entity_type enum value as it might be referenced in audit_link