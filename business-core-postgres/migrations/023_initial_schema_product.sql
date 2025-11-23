-- Migration: Initial Product Schema with Audit Support
-- Description: Creates product-related tables with audit trail.

-- Enum Types for Product
CREATE TYPE product_type AS ENUM ('CASA', 'LOAN');

CREATE TYPE posting_frequency AS ENUM (
    'Daily',
    'Weekly',
    'Monthly',
    'Quarterly',
    'Annually'
);

CREATE TYPE interest_calculation_method AS ENUM (
    'DailyBalance',
    'AverageDailyBalance',
    'MinimumBalance',
    'Simple',
    'Compound',
    'Mudarabah',
    'Musharakah',
    'Wakalah',
    'QardHasan'
);

CREATE TYPE maintenance_fee_frequency AS ENUM (
    'Daily',
    'Weekly',
    'BiWeekly',
    'Monthly',
    'BiMonthly',
    'Quarterly',
    'SemiAnnually',
    'Annually',
    'OneTime',
    'None'
);

CREATE TYPE product_accrual_frequency AS ENUM (
    'Daily',
    'BusinessDaysOnly',
    'None'
);

-- Main Product Table
-- Stores the current state of the product entity.
CREATE TABLE IF NOT EXISTS product (
    id UUID PRIMARY KEY,
    name UUID NOT NULL,
    product_type product_type NOT NULL,
    minimum_balance DECIMAL(19, 4) NOT NULL,
    maximum_balance DECIMAL(19, 4),
    overdraft_allowed BOOLEAN NOT NULL DEFAULT FALSE,
    overdraft_limit DECIMAL(19, 4),
    interest_calculation_method interest_calculation_method NOT NULL,
    interest_posting_frequency posting_frequency NOT NULL,
    dormancy_threshold_days INTEGER NOT NULL,
    minimum_opening_balance DECIMAL(19, 4) NOT NULL,
    closure_fee DECIMAL(19, 4) NOT NULL,
    maintenance_fee DECIMAL(19, 4),
    maintenance_fee_frequency maintenance_fee_frequency NOT NULL DEFAULT 'None',
    default_dormancy_days INTEGER,
    default_overdraft_limit DECIMAL(19, 4),
    per_transaction_limit DECIMAL(19, 4),
    daily_transaction_limit DECIMAL(19, 4),
    weekly_transaction_limit DECIMAL(19, 4),
    monthly_transaction_limit DECIMAL(19, 4),
    overdraft_interest_rate DECIMAL(19, 4),
    accrual_frequency product_accrual_frequency NOT NULL,
    interest_rate_tier_1 UUID REFERENCES interest_rate_tier(id),
    interest_rate_tier_2 UUID REFERENCES interest_rate_tier(id),
    interest_rate_tier_3 UUID REFERENCES interest_rate_tier(id),
    interest_rate_tier_4 UUID REFERENCES interest_rate_tier(id),
    interest_rate_tier_5 UUID REFERENCES interest_rate_tier(id),
    account_gl_mapping UUID NOT NULL REFERENCES account_gl_mapping(id),
    fee_type_gl_mapping UUID NOT NULL REFERENCES fee_type_gl_mapping(id),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    valid_from DATE NOT NULL,
    valid_to DATE,
    hash BIGINT NOT NULL DEFAULT 0,
    audit_log_id UUID REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
);

-- Product Index Table
-- Contains fields for application-layer indexing and caching.
CREATE TABLE IF NOT EXISTS product_idx (
    id UUID PRIMARY KEY REFERENCES product(id) ON DELETE CASCADE,
    name UUID NOT NULL,
    product_type product_type NOT NULL,
    minimum_balance DECIMAL(19, 4) NOT NULL,
    maximum_balance DECIMAL(19, 4),
    overdraft_allowed BOOLEAN NOT NULL,
    overdraft_limit DECIMAL(19, 4),
    interest_calculation_method interest_calculation_method NOT NULL,
    interest_posting_frequency posting_frequency NOT NULL,
    dormancy_threshold_days INTEGER NOT NULL,
    minimum_opening_balance DECIMAL(19, 4) NOT NULL,
    closure_fee DECIMAL(19, 4) NOT NULL,
    maintenance_fee DECIMAL(19, 4),
    maintenance_fee_frequency maintenance_fee_frequency NOT NULL DEFAULT 'None',
    default_dormancy_days INTEGER,
    default_overdraft_limit DECIMAL(19, 4),
    per_transaction_limit DECIMAL(19, 4),
    daily_transaction_limit DECIMAL(19, 4),
    weekly_transaction_limit DECIMAL(19, 4),
    monthly_transaction_limit DECIMAL(19, 4),
    overdraft_interest_rate DECIMAL(19, 4),
    accrual_frequency product_accrual_frequency NOT NULL,
    interest_rate_tier_1 UUID,
    interest_rate_tier_2 UUID,
    interest_rate_tier_3 UUID,
    interest_rate_tier_4 UUID,
    interest_rate_tier_5 UUID,
    account_gl_mapping UUID NOT NULL,
    fee_type_gl_mapping UUID NOT NULL,
    is_active BOOLEAN NOT NULL,
    valid_from DATE NOT NULL,
    valid_to DATE
);

-- Create trigger for product_idx table to notify listeners of changes
DROP TRIGGER IF EXISTS product_idx_notify ON product_idx;
CREATE TRIGGER product_idx_notify
    AFTER INSERT OR UPDATE OR DELETE ON product_idx
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();

-- Product Audit Table
-- Stores a complete, immutable snapshot of the product at each change.
CREATE TABLE IF NOT EXISTS product_audit (
    -- All entity fields are duplicated here for a complete snapshot.
    id UUID NOT NULL,
    name UUID NOT NULL,
    product_type product_type NOT NULL,
    minimum_balance DECIMAL(19, 4) NOT NULL,
    maximum_balance DECIMAL(19, 4),
    overdraft_allowed BOOLEAN NOT NULL,
    overdraft_limit DECIMAL(19, 4),
    interest_calculation_method interest_calculation_method NOT NULL,
    interest_posting_frequency posting_frequency NOT NULL,
    dormancy_threshold_days INTEGER NOT NULL,
    minimum_opening_balance DECIMAL(19, 4) NOT NULL,
    closure_fee DECIMAL(19, 4) NOT NULL,
    maintenance_fee DECIMAL(19, 4),
    maintenance_fee_frequency maintenance_fee_frequency NOT NULL DEFAULT 'None',
    default_dormancy_days INTEGER,
    default_overdraft_limit DECIMAL(19, 4),
    per_transaction_limit DECIMAL(19, 4),
    daily_transaction_limit DECIMAL(19, 4),
    weekly_transaction_limit DECIMAL(19, 4),
    monthly_transaction_limit DECIMAL(19, 4),
    overdraft_interest_rate DECIMAL(19, 4),
    accrual_frequency product_accrual_frequency NOT NULL,
    interest_rate_tier_1 UUID,
    interest_rate_tier_2 UUID,
    interest_rate_tier_3 UUID,
    interest_rate_tier_4 UUID,
    interest_rate_tier_5 UUID,
    account_gl_mapping UUID NOT NULL,
    fee_type_gl_mapping UUID NOT NULL,
    is_active BOOLEAN NOT NULL,
    valid_from DATE NOT NULL,
    valid_to DATE,
    
    -- Audit-specific fields
    hash BIGINT NOT NULL,
    audit_log_id UUID NOT NULL REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    
    -- Composite primary key ensures one audit entry per product version.
    PRIMARY KEY (id, audit_log_id)
);

-- Index on id for efficient audit queries by entity ID.
-- Note: The audit table intentionally lacks a foreign key to the main table
-- with `ON DELETE CASCADE`. This ensures that audit history is preserved
-- even if the main entity record is deleted.
CREATE INDEX IF NOT EXISTS idx_product_audit_id
    ON product_audit(id);

-- Add new entity type to audit_entity_type enum
ALTER TYPE audit_entity_type ADD VALUE 'Product';