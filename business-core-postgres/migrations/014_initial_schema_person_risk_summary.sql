-- Migration: Initial RiskSummary Schema
-- Description: Creates risk_summary-related tables and indexes

-- RiskSummary Table
CREATE TABLE IF NOT EXISTS risk_summary (
    id UUID PRIMARY KEY,
    person_id UUID NOT NULL REFERENCES person(id) ON DELETE CASCADE,
    current_rating risk_rating NOT NULL,
    last_assessment_date TIMESTAMPTZ NOT NULL,
    flags_01 VARCHAR(200) NOT NULL,
    flags_02 VARCHAR(200) NOT NULL,
    flags_03 VARCHAR(200) NOT NULL,
    flags_04 VARCHAR(200) NOT NULL,
    flags_05 VARCHAR(200) NOT NULL
);

-- RiskSummary Index Table
CREATE TABLE IF NOT EXISTS risk_summary_idx (
    id UUID PRIMARY KEY REFERENCES risk_summary(id) ON DELETE CASCADE,
    person_id UUID NOT NULL
);

-- Create trigger for risk_summary_idx table to notify listeners of changes
DROP TRIGGER IF EXISTS risk_summary_idx_notify ON risk_summary_idx;
CREATE TRIGGER risk_summary_idx_notify
    AFTER INSERT OR UPDATE OR DELETE ON risk_summary_idx
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();