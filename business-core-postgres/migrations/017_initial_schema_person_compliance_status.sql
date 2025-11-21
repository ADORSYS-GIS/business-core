-- Migration: Initial Person Compliance Status Schema with Audit Support
-- Description: Creates person_compliance_status-related tables with audit trail.
-- Note: This entity is NOT indexed (no idx table, no cache, no triggers).

-- Update entity_type enum to include COMPLIANCE_STATUS
-- Note: This assumes the entity_type enum exists from the audit schema migration
ALTER TYPE entity_type ADD VALUE IF NOT EXISTS 'COMPLIANCE_STATUS';

-- Create kyc_status ENUM type
CREATE TYPE kyc_status AS ENUM (
    'NotStarted',
    'InProgress',
    'Pending',
    'Complete',
    'Approved',
    'Rejected',
    'RequiresUpdate',
    'Failed'
);

-- Main Person Compliance Status Table
-- Stores the current state of the compliance status.
CREATE TABLE IF NOT EXISTS person_compliance_status (
    id UUID PRIMARY KEY,
    person_id UUID NOT NULL,
    kyc_status kyc_status NOT NULL,
    sanctions_checked BOOLEAN NOT NULL DEFAULT false,
    last_screening_date TIMESTAMPTZ,
    predecessor_1 UUID,
    predecessor_2 UUID,
    predecessor_3 UUID,
    hash BIGINT NOT NULL DEFAULT 0,
    audit_log_id UUID REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
);

-- Person Compliance Status Audit Table
-- Stores a complete, immutable snapshot of the entity at each change.
CREATE TABLE IF NOT EXISTS person_compliance_status_audit (
    -- All entity fields are duplicated here for a complete snapshot.
    id UUID NOT NULL,
    person_id UUID NOT NULL,
    kyc_status kyc_status NOT NULL,
    sanctions_checked BOOLEAN NOT NULL,
    last_screening_date TIMESTAMPTZ,
    predecessor_1 UUID,
    predecessor_2 UUID,
    predecessor_3 UUID,
    
    -- Audit-specific fields
    hash BIGINT NOT NULL,
    audit_log_id UUID NOT NULL REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    
    -- Composite primary key ensures one audit entry per entity version.
    PRIMARY KEY (id, audit_log_id)
);

-- Index on id for efficient audit queries by entity ID.
-- Note: The audit table intentionally lacks a foreign key to the main table
-- with `ON DELETE CASCADE`. This ensures that audit history is preserved
-- even if the main entity record is deleted.
CREATE INDEX IF NOT EXISTS idx_person_compliance_status_audit_id
    ON person_compliance_status_audit(id);