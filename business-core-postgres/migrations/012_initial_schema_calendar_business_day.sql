-- Migration: Initial BusinessDay Schema with Index and Main Cache
-- Description: Creates business_day-related tables with index and cache notification triggers

-- Create weekday enum type if it doesn't exist
DO $$ BEGIN
    CREATE TYPE weekday AS ENUM ('Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Create day_scope enum type
DO $$ BEGIN
    CREATE TYPE day_scope AS ENUM ('National', 'Regional', 'Religious', 'Banking');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- BusinessDay Table
CREATE TABLE IF NOT EXISTS calendar_business_day (
    id UUID PRIMARY KEY,
    country_id UUID,
    country_subdivision_id UUID,
    date DATE NOT NULL,
    weekday weekday NOT NULL,
    is_business_day BOOLEAN NOT NULL,
    is_weekend BOOLEAN NOT NULL,
    weekend_day_01 UUID,
    is_holiday BOOLEAN NOT NULL,
    holiday_name VARCHAR(50),
    day_scope day_scope NOT NULL
);

-- BusinessDay Index Table
CREATE TABLE IF NOT EXISTS calendar_business_day_idx (
    id UUID PRIMARY KEY REFERENCES calendar_business_day(id) ON DELETE CASCADE,
    country_id UUID,
    country_subdivision_id UUID,
    date_hash BIGINT NOT NULL
);

-- Create trigger for calendar_business_day_idx table to notify index cache changes
DROP TRIGGER IF EXISTS calendar_business_day_idx_notify ON calendar_business_day_idx;
CREATE TRIGGER calendar_business_day_idx_notify
    AFTER INSERT OR UPDATE OR DELETE ON calendar_business_day_idx
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();

-- Create trigger for calendar_business_day table to notify main cache changes
DROP TRIGGER IF EXISTS calendar_business_day_notify ON calendar_business_day;
CREATE TRIGGER calendar_business_day_notify
    AFTER INSERT OR UPDATE OR DELETE ON calendar_business_day
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();