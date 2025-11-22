-- Cleanup: Drop DateCalculationRules Schema
-- Description: Removes date_calculation_rules tables, triggers, and enum types

-- Drop triggers
DROP TRIGGER IF EXISTS calendar_date_calculation_rules_notify ON calendar_date_calculation_rules;
DROP TRIGGER IF EXISTS calendar_date_calculation_rules_idx_notify ON calendar_date_calculation_rules_idx;

-- Drop tables (cascade will handle foreign key constraints)
DROP TABLE IF EXISTS calendar_date_calculation_rules_idx CASCADE;
DROP TABLE IF EXISTS calendar_date_calculation_rules CASCADE;

-- Drop enum types
DROP TYPE IF EXISTS date_shift_rule CASCADE;
DROP TYPE IF EXISTS date_rule_purpose CASCADE;