-- Migration: Initial FeeTypeGlMapping Schema with Audit Support
-- Description: Creates fee_type_gl_mapping-related tables with audit trail.

-- Enum Type for FeeType
CREATE TYPE fee_type AS ENUM (
    'InterestExpense',
    'GeneralFeeIncome',
    'AtmWithdrawalOwn',
    'AtmWithdrawalOther',
    'TransferDomestic',
    'TransferInternational',
    'DebitCardTransaction',
    'CreditCardTransaction',
    'CheckProcessing',
    'StopPayment',
    'CashDeposit',
    'CashWithdrawal',
    'MaintenanceFee',
    'MinimumBalancePenalty',
    'AccountOpening',
    'AccountClosure',
    'DormancyReactivation',
    'StatementPaper',
    'StatementCopy',
    'SmsAlert',
    'EmailAlert',
    'CheckbookIssuance',
    'DebitCardIssuance',
    'DebitCardReplacement',
    'CreditCardAnnual',
    'ForeignExchange',
    'AccountCertificate',
    'BalanceInquiry',
    'NsfFee',
    'OverdraftPenalty',
    'OverLimitFee',
    'LatePaymentPenalty',
    'ReturnedItem',
    'GeneralPenalty',
    'MudarabahProfit',
    'MusharakahProfit',
    'MusharakahLoss',
    'WakalahFee',
    'Hibah',
    'QardHasanAdminFee',
    'CharityPenalty',
    'UjrahServiceFee',
    'TakafulContribution',
    'SafekeepingFee',
    'DocumentProcessing',
    'AccountResearch',
    'ThirdPartyService',
    'Other'
);

-- Main FeeTypeGlMapping Table
CREATE TABLE IF NOT EXISTS fee_type_gl_mapping (
    id UUID PRIMARY KEY,
    fee_type fee_type NOT NULL,
    gl_code VARCHAR(50) NOT NULL,
    hash BIGINT NOT NULL DEFAULT 0,
    audit_log_id UUID REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
);

-- FeeTypeGlMapping Index Table
CREATE TABLE IF NOT EXISTS fee_type_gl_mapping_idx (
    id UUID PRIMARY KEY REFERENCES fee_type_gl_mapping(id) ON DELETE CASCADE,
    fee_type fee_type NOT NULL,
    gl_code VARCHAR(50) NOT NULL
);

-- Create trigger for fee_type_gl_mapping_idx table to notify listeners of changes
DROP TRIGGER IF EXISTS fee_type_gl_mapping_idx_notify ON fee_type_gl_mapping_idx;
CREATE TRIGGER fee_type_gl_mapping_idx_notify
    AFTER INSERT OR UPDATE OR DELETE ON fee_type_gl_mapping_idx
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();

-- FeeTypeGlMapping Audit Table
CREATE TABLE IF NOT EXISTS fee_type_gl_mapping_audit (
    id UUID NOT NULL,
    fee_type fee_type NOT NULL,
    gl_code VARCHAR(50) NOT NULL,
    hash BIGINT NOT NULL,
    audit_log_id UUID NOT NULL REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    PRIMARY KEY (id, audit_log_id)
);

-- Index on id for efficient audit queries by entity ID.
CREATE INDEX IF NOT EXISTS idx_fee_type_gl_mapping_audit_id
    ON fee_type_gl_mapping_audit(id);

-- Add new entity type to audit_entity_type enum
ALTER TYPE audit_entity_type ADD VALUE 'FeeTypeGlMapping';