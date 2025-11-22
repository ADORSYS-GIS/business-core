-- Cleanup: Remove BusinessDay schema with Index and Main Cache
-- Description: Drops business_day-related tables, triggers, and enum types

-- Drop triggers first
DROP TRIGGER IF EXISTS calendar_business_day_notify ON calendar_business_day;
DROP TRIGGER IF EXISTS calendar_business_day_idx_notify ON calendar_business_day_idx;

-- Drop tables (index table first due to foreign key)
DROP TABLE IF EXISTS calendar_business_day_idx CASCADE;
DROP TABLE IF EXISTS calendar_business_day CASCADE;

-- Drop enum types (only if not used by other tables)
-- Note: These types might be shared with other calendar tables, so be cautious
DROP TYPE IF EXISTS holiday_type CASCADE;
-- DROP TYPE IF EXISTS day_scope CASCADE;
-- DROP TYPE IF EXISTS weekday CASCADE;