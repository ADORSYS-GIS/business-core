-- Cleanup: Initial Person Compliance Status Schema
-- Description: Removes all artifacts created by 017_initial_schema_person_compliance_status.sql

-- Drop tables (audit table first, then main table)
DROP TABLE IF EXISTS person_compliance_status_audit CASCADE;
DROP TABLE IF EXISTS person_compliance_status CASCADE;

-- Drop kyc_status ENUM type
DROP TYPE IF EXISTS kyc_status CASCADE;