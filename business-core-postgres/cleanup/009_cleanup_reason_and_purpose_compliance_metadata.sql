-- Cleanup: Initial ComplianceMetadata Schema
-- Description: Removes all artifacts created by 008_initial_schema_reason_and_purpose_compliance_metadata.sql

-- Drop trigger first
DROP TRIGGER IF EXISTS compliance_metadata_idx_notify ON compliance_metadata_idx;

-- Drop tables (index table first due to foreign key constraint)
DROP TABLE IF EXISTS compliance_metadata_idx CASCADE;
DROP TABLE IF EXISTS compliance_metadata CASCADE;