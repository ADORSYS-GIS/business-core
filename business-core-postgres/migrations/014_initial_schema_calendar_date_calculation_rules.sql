-- Migration: Initial DateCalculationRules Schema with Index and Main Cache
-- Description: Creates date_calculation_rules-related tables with index and cache notification triggers

-- Create date_rule_purpose enum type
DO $$ BEGIN
    CREATE TYPE date_rule_purpose AS ENUM ('DateShift', 'MaturityCalculation', 'PaymentDue');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- Create date_shift_rule enum type
DO $$ BEGIN
    CREATE TYPE date_shift_rule AS ENUM ('NextBusinessDay', 'PreviousBusinessDay', 'NoShift');
EXCEPTION
    WHEN duplicate_object THEN null;
END $$;

-- DateCalculationRules Table
CREATE TABLE IF NOT EXISTS calendar_date_calculation_rules (
    id UUID PRIMARY KEY,
    country_id UUID NOT NULL,
    country_subdivision_id UUID,
    rule_name VARCHAR(50) NOT NULL,
    rule_purpose date_rule_purpose NOT NULL,
    default_shift_rule date_shift_rule NOT NULL,
    weekend_days_id UUID,
    priority INTEGER NOT NULL,
    is_active BOOLEAN NOT NULL,
    effective_date DATE NOT NULL,
    expiry_date DATE
);

-- DateCalculationRules Index Table
CREATE TABLE IF NOT EXISTS calendar_date_calculation_rules_idx (
    id UUID PRIMARY KEY REFERENCES calendar_date_calculation_rules(id) ON DELETE CASCADE,
    country_id UUID,
    country_subdivision_id UUID,
    rule_name_hash BIGINT NOT NULL
);

-- Create trigger for calendar_date_calculation_rules_idx table to notify index cache changes
DROP TRIGGER IF EXISTS calendar_date_calculation_rules_idx_notify ON calendar_date_calculation_rules_idx;
CREATE TRIGGER calendar_date_calculation_rules_idx_notify
    AFTER INSERT OR UPDATE OR DELETE ON calendar_date_calculation_rules_idx
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();

-- Create trigger for calendar_date_calculation_rules table to notify main cache changes
DROP TRIGGER IF EXISTS calendar_date_calculation_rules_notify ON calendar_date_calculation_rules;
CREATE TRIGGER calendar_date_calculation_rules_notify
    AFTER INSERT OR UPDATE OR DELETE ON calendar_date_calculation_rules
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();