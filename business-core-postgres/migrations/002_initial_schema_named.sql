-- Migration: Initial Named Schema with Audit Support
-- Description: Creates named-related tables with audit trail.

-- Named Entity Type Enum
CREATE TYPE named_entity_type AS ENUM ('Location', 'Person');

-- Main Named Table
-- Stores the current state of the named entity with multilingual support.
CREATE TABLE IF NOT EXISTS named (
    id UUID PRIMARY KEY,
    entity_type named_entity_type NOT NULL,
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

-- Named Index Table
-- Contains fields for application-layer indexing and caching.
CREATE TABLE IF NOT EXISTS named_idx (
    id UUID PRIMARY KEY REFERENCES named(id) ON DELETE CASCADE,
    entity_type named_entity_type NOT NULL
);

-- Named Audit Table
-- Stores a complete, immutable snapshot of the entity at each change.
CREATE TABLE IF NOT EXISTS named_audit (
    -- All entity fields are duplicated here for a complete snapshot.
    id UUID NOT NULL,
    entity_type named_entity_type NOT NULL,
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

-- Create trigger for named_idx table to notify listeners of changes
DROP TRIGGER IF EXISTS named_idx_notify ON named_idx;
CREATE TRIGGER named_idx_notify
    AFTER INSERT OR UPDATE OR DELETE ON named_idx
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();

-- Update audit_entity_type enum to include Named
-- Note: This assumes the audit_entity_type enum exists from the audit schema migration
ALTER TYPE audit_entity_type ADD VALUE IF NOT EXISTS 'Named';