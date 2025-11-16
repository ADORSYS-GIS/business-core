-- Migration: Initial Entity Reference Schema with Audit Support
-- Description: Creates entity_reference-related tables with audit trail.

CREATE TYPE person_entity_type AS ENUM ('Customer', 'Employee', 'Shareholder', 'Director', 'BeneficialOwner', 'Agent', 'Vendor', 'Partner', 'RegulatoryContact', 'EmergencyContact', 'SystemAdmin', 'Other');
 
-- Main Entity Reference Table
-- Stores the current state of the entity.
CREATE TABLE IF NOT EXISTS entity_reference (
    id UUID PRIMARY KEY,
    person_id UUID NOT NULL,
    entity_role person_entity_type NOT NULL,
    reference_external_id VARCHAR(50) NOT NULL,
    reference_details_l1 VARCHAR(50),
    reference_details_l2 VARCHAR(50),
    reference_details_l3 VARCHAR(50),
    hash BIGINT NOT NULL DEFAULT 0,
    audit_log_id UUID REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
);

-- Entity Reference Index Table
-- Contains fields for application-layer indexing and caching.
CREATE TABLE IF NOT EXISTS entity_reference_idx (
    id UUID PRIMARY KEY REFERENCES entity_reference(id) ON DELETE CASCADE,
    person_id UUID NOT NULL,
    reference_external_id_hash BIGINT NOT NULL
);

-- Entity Reference Audit Table
-- Stores a complete, immutable snapshot of the entity at each change.
CREATE TABLE IF NOT EXISTS entity_reference_audit (
    -- All entity fields are duplicated here for a complete snapshot.
    id UUID NOT NULL,
    person_id UUID NOT NULL,
    entity_role person_entity_type NOT NULL,
    reference_external_id VARCHAR(50) NOT NULL,
    reference_details_l1 VARCHAR(50),
    reference_details_l2 VARCHAR(50),
    reference_details_l3 VARCHAR(50),
    
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
CREATE INDEX IF NOT EXISTS idx_entity_reference_audit_audit_log_id
    ON entity_reference_audit(audit_log_id);

-- Create trigger for entity_reference_idx table to notify listeners of changes
DROP TRIGGER IF EXISTS entity_reference_idx_notify ON entity_reference_idx;
CREATE TRIGGER entity_reference_idx_notify
    AFTER INSERT OR UPDATE OR DELETE ON entity_reference_idx
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();

-- Update entity_type enum to include ENTITY_REFERENCE
-- Note: This assumes the entity_type enum exists from the audit schema migration
ALTER TYPE entity_type ADD VALUE IF NOT EXISTS 'ENTITY_REFERENCE';