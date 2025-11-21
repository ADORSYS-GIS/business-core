-- Migration: Initial Document Schema with Audit Support
-- Description: Creates document-related tables with audit trail.
-- Note: This entity is NOT indexed (no idx table, no cache, no triggers).

-- Create DocumentStatus enum
CREATE TYPE document_status AS ENUM ('Uploaded', 'Verified', 'Rejected', 'Expired');

-- Create DocumentType enum
CREATE TYPE document_type AS ENUM (
    -- Identity Documents
    'Passport',
    'NationalIdCard',
    'DriverLicense',
    'ResidencePermit',
    'VoterIdCard',
    'AsylumCard',
    'IdApplicationReceipt',
    'BirthCertificate',
    -- Proof of Address
    'UtilityBill',
    'BankStatement',
    'TaxNotice',
    'TaxBill',
    'LeaseAgreement',
    'PropertyDeed',
    'LandTitle',
    -- Financial/Income Documents
    'EmploymentLetter',
    'PaySlip',
    'TaxReturn',
    'IncomeStatement',
    'FinancialStatement',
    'CreditReport',
    -- Business Documents
    'CertificateOfIncorporation',
    'ArticlesOfAssociation',
    'TaxIdentificationNumber',
    'BusinessLicense',
    'ShareholderRegister',
    'BusinessPlan',
    'AuditorsReport',
    -- Compliance/Legal
    'ProofOfFunds',
    'CriminalRecordCheck',
    'CourtOrder',
    'CourtDocument',
    'PowerOfAttorney',
    'BeneficialOwnershipDeclaration',
    'MarriageCertificate',
    'DeathCertificate',
    'PoliceReport',
    -- Vehicle/Property
    'CarTitle',
    -- Insurance Documents
    'HealthInsurance',
    'LifeInsurance',
    'PropertyInsurance',
    -- Banking/Microfinance Specific
    'LoanApplication',
    'CollateralDocument',
    'GuarantorLetter',
    'SavingsGroupMembership',
    'AgriculturalLoanDocumentation',
    'ReferenceLetter',
    -- Islamic Banking Specific
    'ShariaComplianceCertificate',
    'WakalaAgreement',
    'MudarabaAgreement',
    'MurabahaAgreement',
    'IjaraAgreement',
    'MusharakaAgreement',
    'TakafulCertificate',
    'ZakatCertificate',
    'HalalCertification',
    -- Catch-all
    'Other'
);

-- Main Document Table
-- Stores the current state of the document.
CREATE TABLE IF NOT EXISTS person_document (
    id UUID PRIMARY KEY,
    person_id UUID NOT NULL,
    document_type document_type NOT NULL,
    document_path VARCHAR(500),
    status document_status NOT NULL,
    predecessor_1 UUID,
    predecessor_2 UUID,
    predecessor_3 UUID,
    hash BIGINT NOT NULL DEFAULT 0,
    audit_log_id UUID REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
);

-- Document Audit Table
-- Stores a complete, immutable snapshot of the entity at each change.
CREATE TABLE IF NOT EXISTS person_document_audit (
    -- All entity fields are duplicated here for a complete snapshot.
    id UUID NOT NULL,
    person_id UUID NOT NULL,
    document_type document_type NOT NULL,
    document_path VARCHAR(500),
    status document_status NOT NULL,
    predecessor_1 UUID,
    predecessor_2 UUID,
    predecessor_3 UUID,
    
    -- Audit-specific fields
    hash BIGINT NOT NULL,
    audit_log_id UUID NOT NULL REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    
    -- Composite primary key ensures one audit entry per entity version.
    PRIMARY KEY (id, audit_log_id)
);

-- Index on id for efficient audit queries by entity ID.
-- Note: The audit table intentionally lacks a foreign key to the main table
-- with `ON DELETE CASCADE`. This ensures that audit history is preserved
-- even if the main entity record is deleted.
CREATE INDEX IF NOT EXISTS idx_person_document_audit_id
    ON person_document_audit(id);

-- Update audit_entity_type enum to include Document
-- Note: This assumes the audit_entity_type enum exists from the audit schema migration
ALTER TYPE audit_entity_type ADD VALUE IF NOT EXISTS 'Document';