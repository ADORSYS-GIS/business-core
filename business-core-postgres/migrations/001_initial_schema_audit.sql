-- Migration: Initial Audit Schema
-- Description: Creates the audit_log table for tracking changes

-- Audit Log Table
CREATE TABLE IF NOT EXISTS audit_log (
    id UUID PRIMARY KEY,
    updated_at TIMESTAMPTZ NOT NULL,
    updated_by_person_id UUID NOT NULL
);

-- Indexes for audit_log
CREATE INDEX IF NOT EXISTS idx_audit_log_updated_at ON audit_log(updated_at);
CREATE INDEX IF NOT EXISTS idx_audit_log_updated_by_person_id ON audit_log(updated_by_person_id);

-- Audit Entity Type Enum
CREATE TYPE audit_entity_type AS ENUM ('Location', 'Person');

-- Audit Link Table
CREATE TABLE IF NOT EXISTS audit_link (
    audit_log_id UUID NOT NULL,
    entity_id UUID NOT NULL,
    entity_type audit_entity_type NOT NULL,
    PRIMARY KEY (audit_log_id, entity_id)
);

-- Indexes for audit_link
CREATE INDEX IF NOT EXISTS idx_audit_link_audit_log_id ON audit_link(audit_log_id);