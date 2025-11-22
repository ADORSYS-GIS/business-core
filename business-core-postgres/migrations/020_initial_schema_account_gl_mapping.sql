-- Migration: Initial AccountGlMapping Schema with Audit and Main Cache Support
-- Description: Creates account_gl_mapping-related tables with audit trail and cache notification triggers

-- Main AccountGlMapping Table
CREATE TABLE IF NOT EXISTS account_gl_mapping (
    id UUID PRIMARY KEY,
    customer_account_code VARCHAR(50) NOT NULL,
    overdraft_code VARCHAR(50),
    hash BIGINT NOT NULL DEFAULT 0,
    audit_log_id UUID REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
);

-- AccountGlMapping Index Table
CREATE TABLE IF NOT EXISTS account_gl_mapping_idx (
    id UUID PRIMARY KEY REFERENCES account_gl_mapping(id) ON DELETE CASCADE
);

-- Create trigger for account_gl_mapping_idx table to notify index cache changes
DROP TRIGGER IF EXISTS account_gl_mapping_idx_notify ON account_gl_mapping_idx;
CREATE TRIGGER account_gl_mapping_idx_notify
    AFTER INSERT OR UPDATE OR DELETE ON account_gl_mapping_idx
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();

-- Create trigger for account_gl_mapping table to notify main cache changes
DROP TRIGGER IF EXISTS account_gl_mapping_notify ON account_gl_mapping;
CREATE TRIGGER account_gl_mapping_notify
    AFTER INSERT OR UPDATE OR DELETE ON account_gl_mapping
    FOR EACH ROW
    EXECUTE FUNCTION notify_cache_change();

-- AccountGlMapping Audit Table
CREATE TABLE IF NOT EXISTS account_gl_mapping_audit (
    id UUID NOT NULL,
    customer_account_code VARCHAR(50) NOT NULL,
    overdraft_code VARCHAR(50),
    hash BIGINT NOT NULL,
    audit_log_id UUID NOT NULL REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    PRIMARY KEY (id, audit_log_id)
);

-- Index on id for efficient audit queries by entity ID
CREATE INDEX IF NOT EXISTS idx_account_gl_mapping_audit_id
    ON account_gl_mapping_audit(id);