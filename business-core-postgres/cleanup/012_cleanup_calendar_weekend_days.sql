-- Cleanup: Initial WeekendDays Schema with Index and Main Cache
-- Description: Removes all artifacts created by 011_initial_schema_calendar_weekend_days.sql

-- Drop triggers first
DROP TRIGGER IF EXISTS calendar_weekend_days_notify ON calendar_weekend_days;
DROP TRIGGER IF EXISTS calendar_weekend_days_idx_notify ON calendar_weekend_days_idx;

-- Drop tables (index table first, then main table)
DROP TABLE IF EXISTS calendar_weekend_days_idx CASCADE;
DROP TABLE IF EXISTS calendar_weekend_days CASCADE;

-- Drop the weekday enum type
DROP TYPE IF EXISTS weekday CASCADE;