-- Migration: Initial Reason Reference Schema with Audit Support
-- Description: Creates reason_reference-related tables with audit trail.
-- Note: This entity is NOT indexed (no idx table, no cache, no triggers).

-- Main Reason Reference Table
-- Stores the current state of the reason reference.
CREATE TABLE IF NOT EXISTS reason_reference (
    id UUID PRIMARY KEY,
    reason_id UUID NOT NULL,
    entity_id UUID NOT NULL,
    entity_type entity_type NOT NULL,
    additional_details TEXT,
    hash BIGINT NOT NULL DEFAULT 0,
    audit_log_id UUID REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
);

-- Reason Reference Audit Table
-- Stores a complete, immutable snapshot of the entity at each change.
CREATE TABLE IF NOT EXISTS reason_reference_audit (
    -- All entity fields are duplicated here for a complete snapshot.
    id UUID NOT NULL,
    reason_id UUID NOT NULL,
    entity_id UUID NOT NULL,
    entity_type entity_type NOT NULL,
    additional_details TEXT,
    
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
CREATE INDEX IF NOT EXISTS idx_reason_reference_audit_audit_log_id
    ON reason_reference_audit(audit_log_id);

-- Update entity_type enum to include REASON_REFERENCE
-- Note: This assumes the entity_type enum exists from the audit schema migration
ALTER TYPE entity_type ADD VALUE IF NOT EXISTS 'REASON_REFERENCE';