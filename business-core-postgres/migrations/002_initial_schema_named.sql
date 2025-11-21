-- Migration: Initial Named Schema with Audit Support
-- Description: Creates named-related tables with audit trail.
-- Note: This entity is NOT indexed (no idx table, no cache, no triggers).

-- Main Named Table
-- Stores the current state of the named entity with multilingual support.
CREATE TABLE IF NOT EXISTS named (
    id UUID PRIMARY KEY,
    entity_type entity_type NOT NULL,
    name_l1 VARCHAR(50) NOT NULL,
    name_l2 VARCHAR(50),
    name_l3 VARCHAR(50),
    name_l4 VARCHAR(50),
    description_l1 VARCHAR(255),
    description_l2 VARCHAR(255),
    description_l3 VARCHAR(255),
    description_l4 VARCHAR(255),
    hash BIGINT NOT NULL DEFAULT 0,
    audit_log_id UUID REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
);

-- Named Audit Table
-- Stores a complete, immutable snapshot of the entity at each change.
CREATE TABLE IF NOT EXISTS named_audit (
    -- All entity fields are duplicated here for a complete snapshot.
    id UUID NOT NULL,
    entity_type entity_type NOT NULL,
    name_l1 VARCHAR(50) NOT NULL,
    name_l2 VARCHAR(50),
    name_l3 VARCHAR(50),
    name_l4 VARCHAR(50),
    description_l1 VARCHAR(255),
    description_l2 VARCHAR(255),
    description_l3 VARCHAR(255),
    description_l4 VARCHAR(255),
    
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
CREATE INDEX IF NOT EXISTS idx_named_audit_id
    ON named_audit(id);

-- Update entity_type enum to include NAMED
-- Note: This assumes the entity_type enum exists from the audit schema migration
ALTER TYPE entity_type ADD VALUE IF NOT EXISTS 'NAMED';