-- Migration: Initial ComplianceMetadata Schema
-- Description: Creates compliance_metadata-related tables and indexes

-- ComplianceMetadata Table
CREATE TABLE IF NOT EXISTS compliance_metadata (
    id UUID PRIMARY KEY,
    regulatory_code VARCHAR(20),
    reportable BOOLEAN NOT NULL,
    requires_sar BOOLEAN NOT NULL,
    requires_ctr BOOLEAN NOT NULL,
    retention_years SMALLINT NOT NULL,
    escalation_required BOOLEAN NOT NULL,
    risk_score_impact SMALLINT,
    no_tipping_off BOOLEAN NOT NULL,
    jurisdictions1 VARCHAR(2) NOT NULL,
    jurisdictions2 VARCHAR(2) NOT NULL,
    jurisdictions3 VARCHAR(2) NOT NULL,
    jurisdictions4 VARCHAR(2) NOT NULL,
    jurisdictions5 VARCHAR(2) NOT NULL
);

-- ComplianceMetadata Index Table
CREATE TABLE IF NOT EXISTS compliance_metadata_idx (
    id UUID PRIMARY KEY REFERENCES compliance_metadata(id) ON DELETE CASCADE,
    regulatory_code_hash BIGINT
);

-- Create trigger for compliance_metadata_idx table to notify listeners of changes
DROP TRIGGER IF EXISTS compliance_metadata_idx_notify ON compliance_metadata_idx;
CREATE TRIGGER compliance_metadata_idx_notify
    AFTER INSERT OR UPDATE OR DELETE ON compliance_metadata_idx
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();