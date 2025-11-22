-- Migration: Initial Reason Schema
-- Description: Creates reason-related tables, enum types, and indexes

-- Enum Types
CREATE TYPE reason_category AS ENUM (
    'LoanPurpose',
    'LoanRejection',
    'AccountClosure',
    'AccountSuspension',
    'AccountReactivation',
    'StatusChange',
    'TransactionRejection',
    'TransactionReversal',
    'HoldReason',
    'Compliance',
    'ComplianceFlag',
    'AuditFinding',
    'AmlAlert',
    'AmlInvestigation',
    'SuspiciousActivity',
    'CtfRiskFlag',
    'SanctionsHit',
    'PepFlag',
    'HighRiskCountry',
    'UnusualPattern',
    'KycMissingDocument',
    'KycDocumentRejection',
    'KycVerificationFailure',
    'KycUpdateRequired',
    'IdentityVerificationIssue',
    'LocationVerificationIssue',
    'SourceOfFundsRequired',
    'ComplaintReason',
    'ServiceRequest',
    'SystemGenerated',
    'MaintenanceReason',
    'Other'
);

CREATE TYPE reason_context AS ENUM (
    'Account',
    'Loan',
    'Transaction',
    'Customer',
    'Compliance',
    'AmlCtf',
    'Kyc',
    'System',
    'General'
);

CREATE TYPE reason_severity AS ENUM (
    'Critical',
    'High',
    'Medium',
    'Low',
    'Informational'
);

-- Reason Table
CREATE TABLE IF NOT EXISTS reason (
    id UUID PRIMARY KEY,
    code VARCHAR(50) NOT NULL,
    category reason_category NOT NULL,
    context reason_context NOT NULL,
    l1_content VARCHAR(100),
    l2_content VARCHAR(100),
    l3_content VARCHAR(100),
    l1_language_code VARCHAR(3),
    l2_language_code VARCHAR(3),
    l3_language_code VARCHAR(3),
    requires_details BOOLEAN NOT NULL DEFAULT false,
    is_active BOOLEAN NOT NULL DEFAULT true,
    severity reason_severity,
    display_order INTEGER NOT NULL DEFAULT 0,
    compliance_metadata UUID REFERENCES compliance_metadata(id) ON DELETE SET NULL
);

-- Reason Index Table
CREATE TABLE IF NOT EXISTS reason_idx (
    id UUID PRIMARY KEY REFERENCES reason(id) ON DELETE CASCADE,
    code_hash BIGINT NOT NULL,
    category_hash BIGINT NOT NULL,
    context_hash BIGINT NOT NULL,
    compliance_metadata UUID
);

-- Create indexes for better query performance
CREATE INDEX IF NOT EXISTS idx_reason_code_hash ON reason_idx(code_hash);
CREATE INDEX IF NOT EXISTS idx_reason_category_hash ON reason_idx(category_hash);
CREATE INDEX IF NOT EXISTS idx_reason_context_hash ON reason_idx(context_hash);
CREATE INDEX IF NOT EXISTS idx_reason_compliance_metadata ON reason_idx(compliance_metadata);

-- Create trigger for reason_idx table to notify listeners of changes
DROP TRIGGER IF EXISTS reason_idx_notify ON reason_idx;
CREATE TRIGGER reason_idx_notify
    AFTER INSERT OR UPDATE OR DELETE ON reason_idx
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();