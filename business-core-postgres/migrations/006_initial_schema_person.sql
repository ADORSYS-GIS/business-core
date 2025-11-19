-- Migration: Initial Person Schema with Audit Support
-- Description: Creates person-related tables with audit trail.

CREATE TYPE person_type AS ENUM ('Natural', 'Legal', 'System', 'Integration', 'Unknown');

CREATE TYPE identity_type AS ENUM ('NationalId', 'Passport', 'CompanyRegistration', 'PermanentResidentCard', 'AsylumCard', 'TemporaryResidentPermit', 'Unknown');
 
-- Main Person Table
-- Stores the current state of the entity.
CREATE TABLE IF NOT EXISTS person (
    id UUID PRIMARY KEY,
    person_type person_type NOT NULL,
    display_name VARCHAR(100) NOT NULL,
    external_identifier VARCHAR(50),
    id_type identity_type NOT NULL,
    id_number VARCHAR(50) NOT NULL,
    entity_reference_count INTEGER NOT NULL DEFAULT 0,
    organization_person_id UUID,
    messaging_info1 VARCHAR(50),
    messaging_info2 VARCHAR(50),
    messaging_info3 VARCHAR(50),
    messaging_info4 VARCHAR(50),
    messaging_info5 VARCHAR(50),
    department VARCHAR(50),
    location_id UUID,
    duplicate_of_person_id UUID,
    hash BIGINT NOT NULL DEFAULT 0,
    audit_log_id UUID REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
);

-- Person Index Table
-- Contains fields for application-layer indexing and caching.
CREATE TABLE IF NOT EXISTS person_idx (
    id UUID PRIMARY KEY REFERENCES person(id) ON DELETE CASCADE,
    external_identifier_hash BIGINT,
    organization_person_id UUID,
    duplicate_of_person_id UUID,
    id_number_hash BIGINT
);

-- Person Audit Table
-- Stores a complete, immutable snapshot of the entity at each change.
CREATE TABLE IF NOT EXISTS person_audit (
    -- All entity fields are duplicated here for a complete snapshot.
    id UUID NOT NULL,
    person_type person_type NOT NULL,
    display_name VARCHAR(100) NOT NULL,
    external_identifier VARCHAR(50),
    id_type identity_type NOT NULL,
    id_number VARCHAR(50) NOT NULL,
    entity_reference_count INTEGER NOT NULL DEFAULT 0,
    organization_person_id UUID,
    messaging_info1 VARCHAR(50),
    messaging_info2 VARCHAR(50),
    messaging_info3 VARCHAR(50),
    messaging_info4 VARCHAR(50),
    messaging_info5 VARCHAR(50),
    department VARCHAR(50),
    location_id UUID,
    duplicate_of_person_id UUID,
    
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
CREATE INDEX IF NOT EXISTS idx_person_audit_audit_log_id
    ON person_audit(audit_log_id);

-- Create trigger for person_idx table to notify listeners of changes
DROP TRIGGER IF EXISTS person_idx_notify ON person_idx;
CREATE TRIGGER person_idx_notify
    AFTER INSERT OR UPDATE OR DELETE ON person_idx
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();