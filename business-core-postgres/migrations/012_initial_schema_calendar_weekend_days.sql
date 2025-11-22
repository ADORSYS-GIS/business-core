-- Migration: Initial WeekendDays Schema with Index and Main Cache
-- Description: Creates weekend_days-related tables with index and cache notification triggers

-- Create weekday enum type if it doesn't exist
DO $$ BEGIN
    CREATE TYPE weekday AS ENUM ('Monday', 'Tuesday', 'Wednesday', 'Thursday', 'Friday', 'Saturday', 'Sunday');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- WeekendDays Table
CREATE TABLE IF NOT EXISTS calendar_weekend_days (
    id UUID PRIMARY KEY,
    country_id UUID,
    country_subdivision_id UUID,
    weekend_day_01 weekday,
    weekend_day_02 weekday,
    weekend_day_03 weekday,
    weekend_day_04 weekday,
    weekend_day_05 weekday,
    weekend_day_06 weekday,
    weekend_day_07 weekday,
    effective_date DATE NOT NULL,
    expiry_date DATE
);

-- WeekendDays Index Table
CREATE TABLE IF NOT EXISTS calendar_weekend_days_idx (
    id UUID PRIMARY KEY REFERENCES calendar_weekend_days(id) ON DELETE CASCADE,
    country_id UUID,
    country_subdivision_id UUID
);

-- Create trigger for calendar_weekend_days_idx table to notify index cache changes
DROP TRIGGER IF EXISTS calendar_weekend_days_idx_notify ON calendar_weekend_days_idx;
CREATE TRIGGER calendar_weekend_days_idx_notify
    AFTER INSERT OR UPDATE OR DELETE ON calendar_weekend_days_idx
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();

-- Create trigger for calendar_weekend_days table to notify main cache changes
DROP TRIGGER IF EXISTS calendar_weekend_days_notify ON calendar_weekend_days;
CREATE TRIGGER calendar_weekend_days_notify
    AFTER INSERT OR UPDATE OR DELETE ON calendar_weekend_days
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();