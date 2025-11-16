-- Migration: Initial Location Schema with Audit Support
-- Description: Creates location-related tables with audit trail.

CREATE TYPE location_type AS ENUM ('Residential', 'Business', 'Mailing', 'Temporary', 'Branch', 'Community', 'Other');
 
-- Main Location Table
-- Stores the current state of the entity.
CREATE TABLE IF NOT EXISTS location (
    id UUID PRIMARY KEY,
    street_line1 VARCHAR(50) NOT NULL,
    street_line2 VARCHAR(50),
    street_line3 VARCHAR(50),
    street_line4 VARCHAR(50),
    locality_id UUID NOT NULL,
    postal_code VARCHAR(20),
    latitude DECIMAL,
    longitude DECIMAL,
    accuracy_meters REAL,
    location_type location_type NOT NULL,
    hash BIGINT NOT NULL DEFAULT 0,
    audit_log_id UUID REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
);

-- Location Index Table
-- Contains fields for application-layer indexing and caching.
CREATE TABLE IF NOT EXISTS location_idx (
    id UUID PRIMARY KEY REFERENCES location(id) ON DELETE CASCADE,
    locality_id UUID NOT NULL
);

-- Location Audit Table
-- Stores a complete, immutable snapshot of the entity at each change.
CREATE TABLE IF NOT EXISTS location_audit (
    -- All entity fields are duplicated here for a complete snapshot.
    id UUID NOT NULL,
    street_line1 VARCHAR(50) NOT NULL,
    street_line2 VARCHAR(50),
    street_line3 VARCHAR(50),
    street_line4 VARCHAR(50),
    locality_id UUID NOT NULL,
    postal_code VARCHAR(20),
    latitude DECIMAL,
    longitude DECIMAL,
    accuracy_meters REAL,
    location_type location_type NOT NULL,
    
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
CREATE INDEX IF NOT EXISTS idx_location_audit_audit_log_id
    ON location_audit(audit_log_id);

-- Create trigger for location_idx table to notify listeners of changes
DROP TRIGGER IF EXISTS location_idx_notify ON location_idx;
CREATE TRIGGER location_idx_notify
    AFTER INSERT OR UPDATE OR DELETE ON location_idx
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();