-- Migration: Initial Portfolio Schema with Audit Support
-- Description: Creates portfolio-related tables with audit trail.
-- Note: This entity is NOT indexed (no idx table, no cache, no triggers).

-- Main Portfolio Table
-- Stores the current state of the portfolio.
CREATE TABLE IF NOT EXISTS portfolio (
    id UUID PRIMARY KEY,
    person_id UUID NOT NULL,
    total_accounts BIGINT NOT NULL DEFAULT 0,
    total_balance DECIMAL(19, 4) NOT NULL DEFAULT 0,
    total_loan_outstanding_main DECIMAL(19, 4),
    total_loan_outstanding_grantor DECIMAL(19, 4),
    risk_score DECIMAL(5, 2),
    compliance_status UUID NOT NULL,
    hash BIGINT NOT NULL DEFAULT 0,
    audit_log_id UUID REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
);

-- Portfolio Audit Table
-- Stores a complete, immutable snapshot of the entity at each change.
CREATE TABLE IF NOT EXISTS portfolio_audit (
    -- All entity fields are duplicated here for a complete snapshot.
    id UUID NOT NULL,
    person_id UUID NOT NULL,
    total_accounts BIGINT NOT NULL,
    total_balance DECIMAL(19, 4) NOT NULL,
    total_loan_outstanding_main DECIMAL(19, 4),
    total_loan_outstanding_grantor DECIMAL(19, 4),
    risk_score DECIMAL(5, 2),
    compliance_status UUID NOT NULL,
    
    -- Audit-specific fields
    hash BIGINT NOT NULL,
    audit_log_id UUID NOT NULL REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    
    -- Composite primary key ensures one audit entry per entity version.
    PRIMARY KEY (id, audit_log_id)
);

-- Index on audit_log_id for efficient audit log queries.
-- Note: The audit table intentionally lacks a foreign key to the main table
-- with `ON DELETE CASCADE`. This ensures that audit history is preserved
-- even if the main entity record is deleted.
CREATE INDEX IF NOT EXISTS idx_portfolio_audit_audit_log_id
    ON portfolio_audit(audit_log_id);

-- Update entity_type enum to include PORTFOLIO
-- Note: This assumes the entity_type enum exists from the audit schema migration
ALTER TYPE entity_type ADD VALUE IF NOT EXISTS 'PORTFOLIO';