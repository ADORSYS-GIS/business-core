-- Migration: Initial Country Schema
-- Description: Creates country-related tables and indexes

-- Country Table
CREATE TABLE IF NOT EXISTS country (
    id UUID PRIMARY KEY,
    iso2 VARCHAR(2) NOT NULL,
    name UUID NOT NULL
);

-- Country Index Table
CREATE TABLE IF NOT EXISTS country_idx (
    id UUID PRIMARY KEY REFERENCES country(id) ON DELETE CASCADE,
    iso2_hash BIGINT NOT NULL UNIQUE
);

-- Create trigger for country_idx table to notify listeners of changes
DROP TRIGGER IF EXISTS country_idx_notify ON country_idx;
CREATE TRIGGER country_idx_notify
    AFTER INSERT OR UPDATE OR DELETE ON country_idx
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();