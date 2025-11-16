-- Migration: Initial Locality Schema
-- Description: Creates locality-related tables and indexes

-- Locality Table
CREATE TABLE IF NOT EXISTS locality (
    id UUID PRIMARY KEY,
    country_subdivision_id UUID NOT NULL REFERENCES country_subdivision(id) ON DELETE CASCADE,
    code VARCHAR(50) NOT NULL UNIQUE,
    name_l1 VARCHAR(50) NOT NULL,
    name_l2 VARCHAR(50),
    name_l3 VARCHAR(50)
);

-- Locality Index Table
CREATE TABLE IF NOT EXISTS locality_idx (
    id UUID PRIMARY KEY REFERENCES locality(id) ON DELETE CASCADE,
    country_subdivision_id UUID NOT NULL,
    code_hash BIGINT NOT NULL UNIQUE
);

-- Create trigger for locality_idx table to notify listeners of changes
DROP TRIGGER IF EXISTS locality_idx_notify ON locality_idx;
CREATE TRIGGER locality_idx_notify
    AFTER INSERT OR UPDATE OR DELETE ON locality_idx
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();