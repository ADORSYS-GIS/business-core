-- Migration: Initial CountrySubdivision Schema
-- Description: Creates country_subdivision-related tables and indexes

-- CountrySubdivision Table
CREATE TABLE IF NOT EXISTS country_subdivision (
    id UUID PRIMARY KEY,
    country_id UUID NOT NULL REFERENCES country(id) ON DELETE CASCADE,
    code VARCHAR(10) NOT NULL UNIQUE,
    name UUID NOT NULL
);

-- CountrySubdivision Index Table
CREATE TABLE IF NOT EXISTS country_subdivision_idx (
    id UUID PRIMARY KEY REFERENCES country_subdivision(id) ON DELETE CASCADE,
    country_id UUID NOT NULL,
    code_hash BIGINT NOT NULL UNIQUE
);

-- Create trigger for country_subdivision_idx table to notify listeners of changes
DROP TRIGGER IF EXISTS country_subdivision_idx_notify ON country_subdivision_idx;
CREATE TRIGGER country_subdivision_idx_notify
    AFTER INSERT OR UPDATE OR DELETE ON country_subdivision_idx
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();