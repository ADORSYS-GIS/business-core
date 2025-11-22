-- Cleanup: Initial RiskSummary Schema
-- Description: Removes all artifacts created by 014_initial_schema_person_risk_summary.sql

-- Drop trigger first
DROP TRIGGER IF EXISTS risk_summary_idx_notify ON risk_summary_idx;

-- Drop tables (index table first due to foreign key constraint)
DROP TABLE IF EXISTS risk_summary_idx CASCADE;
DROP TABLE IF EXISTS risk_summary CASCADE;