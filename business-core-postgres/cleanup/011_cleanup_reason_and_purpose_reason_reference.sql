-- Cleanup: Initial Reason Reference Schema
-- Description: Removes all artifacts created by 010_initial_schema_reason_and_purpose_reason_reference.sql

-- Drop tables (audit table first, then main table)
DROP TABLE IF EXISTS reason_reference_audit CASCADE;
DROP TABLE IF EXISTS reason_reference CASCADE;