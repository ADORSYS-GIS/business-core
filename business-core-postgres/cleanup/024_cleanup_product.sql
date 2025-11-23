-- Cleanup script for product tables
-- This script removes all product-related data and tables

-- Drop triggers first
DROP TRIGGER IF EXISTS notify_product_idx_changes ON product_idx;

-- Drop tables (in order due to foreign key constraints)
DROP TABLE IF EXISTS product_audit CASCADE;
DROP TABLE IF EXISTS product_idx CASCADE;
DROP TABLE IF EXISTS product CASCADE;

-- Drop enum types
DROP TYPE IF EXISTS product_type CASCADE;
DROP TYPE IF EXISTS posting_frequency CASCADE;
DROP TYPE IF EXISTS interest_calculation_method CASCADE;
DROP TYPE IF EXISTS maintenance_fee_frequency CASCADE;
DROP TYPE IF EXISTS product_accrual_frequency CASCADE;

-- Remove Product from audit_entity_type enum
-- Note: PostgreSQL doesn't support removing enum values directly
-- This would require recreating the enum type if needed in production
-- For cleanup purposes, we document this but don't execute it
-- ALTER TYPE audit_entity_type DROP VALUE IF EXISTS 'Product';