-- Migration: Initial ActivityLog Schema with Audit Support
-- Description: Creates person_activity_log-related tables with audit trail.
-- Note: This entity is NOT indexed (no idx table, no cache, no triggers).

-- Main ActivityLog Table
-- Stores the current state of the activity log.
CREATE TABLE IF NOT EXISTS person_activity_log (
    id UUID PRIMARY KEY,
    person_id UUID NOT NULL,
    activity_summary TEXT,
    predecessor_1 UUID,
    predecessor_2 UUID,
    predecessor_3 UUID,
    hash BIGINT NOT NULL DEFAULT 0,
    audit_log_id UUID REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
);

-- ActivityLog Audit Table
-- Stores a complete, immutable snapshot of the entity at each change.
CREATE TABLE IF NOT EXISTS person_activity_log_audit (
    -- All entity fields are duplicated here for a complete snapshot.
    id UUID NOT NULL,
    person_id UUID NOT NULL,
    activity_summary TEXT,
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
CREATE INDEX IF NOT EXISTS idx_person_activity_log_audit_id
    ON person_activity_log_audit(id);

-- Update entity_type enum to include ACTIVITY_LOG
-- Note: This assumes the entity_type enum exists from the audit schema migration
ALTER TYPE entity_type ADD VALUE IF NOT EXISTS 'ACTIVITY_LOG';