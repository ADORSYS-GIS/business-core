-- Migration: Initial InterestRateTier Schema with Audit Support
-- Description: Creates interest_rate_tier-related tables with audit trail.

-- Main InterestRateTier Table
-- Stores the current state of the entity.
CREATE TABLE IF NOT EXISTS interest_rate_tier (
    id UUID PRIMARY KEY,
    name UUID NOT NULL,
    minimum_balance DECIMAL(18, 4) NOT NULL,
    maximum_balance DECIMAL(18, 4),
    interest_rate DECIMAL(9, 5) NOT NULL,
    hash BIGINT NOT NULL DEFAULT 0,
    audit_log_id UUID REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
);

-- InterestRateTier Index Table
-- Contains fields for application-layer indexing and caching.
CREATE TABLE IF NOT EXISTS interest_rate_tier_idx (
    id UUID PRIMARY KEY REFERENCES interest_rate_tier(id) ON DELETE CASCADE,
    name UUID NOT NULL,
    minimum_balance DECIMAL(18, 4) NOT NULL,
    maximum_balance DECIMAL(18, 4),
    interest_rate DECIMAL(9, 5) NOT NULL
);

-- Create trigger for interest_rate_tier_idx table to notify listeners of changes
DROP TRIGGER IF EXISTS interest_rate_tier_idx_notify ON interest_rate_tier_idx;
CREATE TRIGGER interest_rate_tier_idx_notify
    AFTER INSERT OR UPDATE OR DELETE ON interest_rate_tier_idx
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();

-- InterestRateTier Audit Table
-- Stores a complete, immutable snapshot of the entity at each change.
CREATE TABLE IF NOT EXISTS interest_rate_tier_audit (
    -- All entity fields are duplicated here for a complete snapshot.
    id UUID NOT NULL,
    name UUID NOT NULL,
    minimum_balance DECIMAL(18, 4) NOT NULL,
    maximum_balance DECIMAL(18, 4),
    interest_rate DECIMAL(9, 5) NOT NULL,
    
    -- Audit-specific fields
    hash BIGINT NOT NULL,
    audit_log_id UUID NOT NULL REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    
    -- Composite primary key ensures one audit entry per entity version.
    PRIMARY KEY (id, audit_log_id)
);

-- Index on id for efficient audit queries by entity ID.
CREATE INDEX IF NOT EXISTS idx_interest_rate_tier_audit_id
    ON interest_rate_tier_audit(id);

-- Add new entity type to audit_entity_type enum
ALTER TYPE audit_entity_type ADD VALUE 'InterestRateTier';