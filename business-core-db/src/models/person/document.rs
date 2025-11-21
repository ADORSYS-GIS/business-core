use heapless::String as HeaplessString;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlx::FromRow;
use std::str::FromStr;
use uuid::Uuid;
use crate::models::auditable::Auditable;
use crate::models::identifiable::Identifiable;

/// # Documentation
/// Database model for Customer documents
/// 
/// Tracks document information for a person including:
/// - Document type (e.g., passport, driver's license)
/// - Document storage path
/// - Document verification status
///
/// This entity is auditable but not indexable - accessed by ID only.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DocumentModel {
    pub id: Uuid,
    
    /// Reference to the person who owns this document
    pub person_id: Uuid,
    
    /// Type of document
    #[serde(
        serialize_with = "serialize_document_type",
        deserialize_with = "deserialize_document_type"
    )]
    pub document_type: DocumentType,
    
    /// Storage path or URL to the document
    pub document_path: Option<HeaplessString<255>>,
    
    /// Current status of the document
    #[serde(
        serialize_with = "serialize_document_status",
        deserialize_with = "deserialize_document_status"
    )]
    pub status: DocumentStatus,
    
    /// First predecessor reference (nullable)
    pub predecessor_1: Option<Uuid>,
    
    /// Second predecessor reference (nullable)
    pub predecessor_2: Option<Uuid>,
    
    /// Third predecessor reference (nullable)
    pub predecessor_3: Option<Uuid>,
    
    /// Hash from the previous audit record for chain verification (0 for initial create)
    pub antecedent_hash: i64,
    
    /// Reference to the previous audit log entry (Uuid::nil() for initial create)
    pub antecedent_audit_log_id: Uuid,
    
    /// Hash of the entity with hash field set to 0
    /// - 0: for new entities not yet created or not yet hashed
    /// - Non-zero: computed hash providing tamper detection
    pub hash: i64,
    
    /// Reference to the current audit log entry for this entity
    /// - None: for new entities not yet created
    /// - Some(uuid): updated on every create/update operation to reference the latest audit log
    /// 
    /// This field, together with `id`, forms the composite primary key in the audit table
    pub audit_log_id: Option<Uuid>,
}

impl Identifiable for DocumentModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Auditable for DocumentModel {
    fn get_audit_log_id(&self) -> Option<Uuid> {
        self.audit_log_id
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "document_status", rename_all = "PascalCase")]
pub enum DocumentStatus {
    Uploaded,
    Verified,
    Rejected,
    Expired,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "document_type", rename_all = "PascalCase")]
pub enum DocumentType {
    // Identity Documents
    Passport,
    NationalIdCard,
    DriverLicense,
    ResidencePermit,
    VoterIdCard,
    AsylumCard,
    IdApplicationReceipt,
    BirthCertificate,
    
    // Proof of Address
    UtilityBill,
    BankStatement,
    TaxNotice,
    TaxBill,
    LeaseAgreement,
    PropertyDeed,
    LandTitle,
    
    // Financial/Income Documents
    EmploymentLetter,
    PaySlip,
    TaxReturn,
    IncomeStatement,
    FinancialStatement,
    CreditReport,
    
    // Business Documents
    CertificateOfIncorporation,
    ArticlesOfAssociation,
    TaxIdentificationNumber,
    BusinessLicense,
    ShareholderRegister,
    BusinessPlan,
    AuditorsReport,
    
    // Compliance/Legal
    ProofOfFunds,
    CriminalRecordCheck,
    CourtOrder,
    CourtDocument,
    PowerOfAttorney,
    BeneficialOwnershipDeclaration,
    MarriageCertificate,
    DeathCertificate,
    PoliceReport,
    
    // Vehicle/Property
    CarTitle,
    
    // Insurance Documents
    HealthInsurance,
    LifeInsurance,
    PropertyInsurance,
    
    // Banking/Microfinance Specific
    LoanApplication,
    CollateralDocument,
    GuarantorLetter,
    SavingsGroupMembership,
    AgriculturalLoanDocumentation,
    ReferenceLetter,
    
    // Islamic Banking Specific
    ShariaComplianceCertificate,
    WakalaAgreement,
    MudarabaAgreement,
    MurabahaAgreement,
    IjaraAgreement,
    MusharakaAgreement,
    TakafulCertificate,
    ZakatCertificate,
    HalalCertification,
    
    // Catch-all
    Other,
}

impl std::fmt::Display for DocumentType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentType::Passport => write!(f, "Passport"),
            DocumentType::NationalIdCard => write!(f, "NationalIdCard"),
            DocumentType::DriverLicense => write!(f, "DriverLicense"),
            DocumentType::ResidencePermit => write!(f, "ResidencePermit"),
            DocumentType::VoterIdCard => write!(f, "VoterIdCard"),
            DocumentType::AsylumCard => write!(f, "AsylumCard"),
            DocumentType::IdApplicationReceipt => write!(f, "IdApplicationReceipt"),
            DocumentType::BirthCertificate => write!(f, "BirthCertificate"),
            DocumentType::UtilityBill => write!(f, "UtilityBill"),
            DocumentType::BankStatement => write!(f, "BankStatement"),
            DocumentType::TaxNotice => write!(f, "TaxNotice"),
            DocumentType::TaxBill => write!(f, "TaxBill"),
            DocumentType::LeaseAgreement => write!(f, "LeaseAgreement"),
            DocumentType::PropertyDeed => write!(f, "PropertyDeed"),
            DocumentType::LandTitle => write!(f, "LandTitle"),
            DocumentType::EmploymentLetter => write!(f, "EmploymentLetter"),
            DocumentType::PaySlip => write!(f, "PaySlip"),
            DocumentType::TaxReturn => write!(f, "TaxReturn"),
            DocumentType::IncomeStatement => write!(f, "IncomeStatement"),
            DocumentType::FinancialStatement => write!(f, "FinancialStatement"),
            DocumentType::CreditReport => write!(f, "CreditReport"),
            DocumentType::CertificateOfIncorporation => write!(f, "CertificateOfIncorporation"),
            DocumentType::ArticlesOfAssociation => write!(f, "ArticlesOfAssociation"),
            DocumentType::TaxIdentificationNumber => write!(f, "TaxIdentificationNumber"),
            DocumentType::BusinessLicense => write!(f, "BusinessLicense"),
            DocumentType::ShareholderRegister => write!(f, "ShareholderRegister"),
            DocumentType::BusinessPlan => write!(f, "BusinessPlan"),
            DocumentType::AuditorsReport => write!(f, "AuditorsReport"),
            DocumentType::ProofOfFunds => write!(f, "ProofOfFunds"),
            DocumentType::CriminalRecordCheck => write!(f, "CriminalRecordCheck"),
            DocumentType::CourtOrder => write!(f, "CourtOrder"),
            DocumentType::CourtDocument => write!(f, "CourtDocument"),
            DocumentType::PowerOfAttorney => write!(f, "PowerOfAttorney"),
            DocumentType::BeneficialOwnershipDeclaration => write!(f, "BeneficialOwnershipDeclaration"),
            DocumentType::MarriageCertificate => write!(f, "MarriageCertificate"),
            DocumentType::DeathCertificate => write!(f, "DeathCertificate"),
            DocumentType::PoliceReport => write!(f, "PoliceReport"),
            DocumentType::CarTitle => write!(f, "CarTitle"),
            DocumentType::HealthInsurance => write!(f, "HealthInsurance"),
            DocumentType::LifeInsurance => write!(f, "LifeInsurance"),
            DocumentType::PropertyInsurance => write!(f, "PropertyInsurance"),
            DocumentType::LoanApplication => write!(f, "LoanApplication"),
            DocumentType::CollateralDocument => write!(f, "CollateralDocument"),
            DocumentType::GuarantorLetter => write!(f, "GuarantorLetter"),
            DocumentType::SavingsGroupMembership => write!(f, "SavingsGroupMembership"),
            DocumentType::AgriculturalLoanDocumentation => write!(f, "AgriculturalLoanDocumentation"),
            DocumentType::ReferenceLetter => write!(f, "ReferenceLetter"),
            DocumentType::ShariaComplianceCertificate => write!(f, "ShariaComplianceCertificate"),
            DocumentType::WakalaAgreement => write!(f, "WakalaAgreement"),
            DocumentType::MudarabaAgreement => write!(f, "MudarabaAgreement"),
            DocumentType::MurabahaAgreement => write!(f, "MurabahaAgreement"),
            DocumentType::IjaraAgreement => write!(f, "IjaraAgreement"),
            DocumentType::MusharakaAgreement => write!(f, "MusharakaAgreement"),
            DocumentType::TakafulCertificate => write!(f, "TakafulCertificate"),
            DocumentType::ZakatCertificate => write!(f, "ZakatCertificate"),
            DocumentType::HalalCertification => write!(f, "HalalCertification"),
            DocumentType::Other => write!(f, "Other"),
        }
    }
}

impl FromStr for DocumentType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Passport" => Ok(DocumentType::Passport),
            "NationalIdCard" => Ok(DocumentType::NationalIdCard),
            "DriverLicense" => Ok(DocumentType::DriverLicense),
            "ResidencePermit" => Ok(DocumentType::ResidencePermit),
            "VoterIdCard" => Ok(DocumentType::VoterIdCard),
            "AsylumCard" => Ok(DocumentType::AsylumCard),
            "IdApplicationReceipt" => Ok(DocumentType::IdApplicationReceipt),
            "BirthCertificate" => Ok(DocumentType::BirthCertificate),
            "UtilityBill" => Ok(DocumentType::UtilityBill),
            "BankStatement" => Ok(DocumentType::BankStatement),
            "TaxNotice" => Ok(DocumentType::TaxNotice),
            "TaxBill" => Ok(DocumentType::TaxBill),
            "LeaseAgreement" => Ok(DocumentType::LeaseAgreement),
            "PropertyDeed" => Ok(DocumentType::PropertyDeed),
            "LandTitle" => Ok(DocumentType::LandTitle),
            "EmploymentLetter" => Ok(DocumentType::EmploymentLetter),
            "PaySlip" => Ok(DocumentType::PaySlip),
            "TaxReturn" => Ok(DocumentType::TaxReturn),
            "IncomeStatement" => Ok(DocumentType::IncomeStatement),
            "FinancialStatement" => Ok(DocumentType::FinancialStatement),
            "CreditReport" => Ok(DocumentType::CreditReport),
            "CertificateOfIncorporation" => Ok(DocumentType::CertificateOfIncorporation),
            "ArticlesOfAssociation" => Ok(DocumentType::ArticlesOfAssociation),
            "TaxIdentificationNumber" => Ok(DocumentType::TaxIdentificationNumber),
            "BusinessLicense" => Ok(DocumentType::BusinessLicense),
            "ShareholderRegister" => Ok(DocumentType::ShareholderRegister),
            "BusinessPlan" => Ok(DocumentType::BusinessPlan),
            "AuditorsReport" => Ok(DocumentType::AuditorsReport),
            "ProofOfFunds" => Ok(DocumentType::ProofOfFunds),
            "CriminalRecordCheck" => Ok(DocumentType::CriminalRecordCheck),
            "CourtOrder" => Ok(DocumentType::CourtOrder),
            "CourtDocument" => Ok(DocumentType::CourtDocument),
            "PowerOfAttorney" => Ok(DocumentType::PowerOfAttorney),
            "BeneficialOwnershipDeclaration" => Ok(DocumentType::BeneficialOwnershipDeclaration),
            "MarriageCertificate" => Ok(DocumentType::MarriageCertificate),
            "DeathCertificate" => Ok(DocumentType::DeathCertificate),
            "PoliceReport" => Ok(DocumentType::PoliceReport),
            "CarTitle" => Ok(DocumentType::CarTitle),
            "HealthInsurance" => Ok(DocumentType::HealthInsurance),
            "LifeInsurance" => Ok(DocumentType::LifeInsurance),
            "PropertyInsurance" => Ok(DocumentType::PropertyInsurance),
            "LoanApplication" => Ok(DocumentType::LoanApplication),
            "CollateralDocument" => Ok(DocumentType::CollateralDocument),
            "GuarantorLetter" => Ok(DocumentType::GuarantorLetter),
            "SavingsGroupMembership" => Ok(DocumentType::SavingsGroupMembership),
            "AgriculturalLoanDocumentation" => Ok(DocumentType::AgriculturalLoanDocumentation),
            "ReferenceLetter" => Ok(DocumentType::ReferenceLetter),
            "ShariaComplianceCertificate" => Ok(DocumentType::ShariaComplianceCertificate),
            "WakalaAgreement" => Ok(DocumentType::WakalaAgreement),
            "MudarabaAgreement" => Ok(DocumentType::MudarabaAgreement),
            "MurabahaAgreement" => Ok(DocumentType::MurabahaAgreement),
            "IjaraAgreement" => Ok(DocumentType::IjaraAgreement),
            "MusharakaAgreement" => Ok(DocumentType::MusharakaAgreement),
            "TakafulCertificate" => Ok(DocumentType::TakafulCertificate),
            "ZakatCertificate" => Ok(DocumentType::ZakatCertificate),
            "HalalCertification" => Ok(DocumentType::HalalCertification),
            "Other" => Ok(DocumentType::Other),
            _ => Err(()),
        }
    }
}

fn serialize_document_type<S>(value: &DocumentType, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let value_str = match value {
        DocumentType::Passport => "Passport",
        DocumentType::NationalIdCard => "NationalIdCard",
        DocumentType::DriverLicense => "DriverLicense",
        DocumentType::ResidencePermit => "ResidencePermit",
        DocumentType::VoterIdCard => "VoterIdCard",
        DocumentType::AsylumCard => "AsylumCard",
        DocumentType::IdApplicationReceipt => "IdApplicationReceipt",
        DocumentType::BirthCertificate => "BirthCertificate",
        DocumentType::UtilityBill => "UtilityBill",
        DocumentType::BankStatement => "BankStatement",
        DocumentType::TaxNotice => "TaxNotice",
        DocumentType::TaxBill => "TaxBill",
        DocumentType::LeaseAgreement => "LeaseAgreement",
        DocumentType::PropertyDeed => "PropertyDeed",
        DocumentType::LandTitle => "LandTitle",
        DocumentType::EmploymentLetter => "EmploymentLetter",
        DocumentType::PaySlip => "PaySlip",
        DocumentType::TaxReturn => "TaxReturn",
        DocumentType::IncomeStatement => "IncomeStatement",
        DocumentType::FinancialStatement => "FinancialStatement",
        DocumentType::CreditReport => "CreditReport",
        DocumentType::CertificateOfIncorporation => "CertificateOfIncorporation",
        DocumentType::ArticlesOfAssociation => "ArticlesOfAssociation",
        DocumentType::TaxIdentificationNumber => "TaxIdentificationNumber",
        DocumentType::BusinessLicense => "BusinessLicense",
        DocumentType::ShareholderRegister => "ShareholderRegister",
        DocumentType::BusinessPlan => "BusinessPlan",
        DocumentType::AuditorsReport => "AuditorsReport",
        DocumentType::ProofOfFunds => "ProofOfFunds",
        DocumentType::CriminalRecordCheck => "CriminalRecordCheck",
        DocumentType::CourtOrder => "CourtOrder",
        DocumentType::CourtDocument => "CourtDocument",
        DocumentType::PowerOfAttorney => "PowerOfAttorney",
        DocumentType::BeneficialOwnershipDeclaration => "BeneficialOwnershipDeclaration",
        DocumentType::MarriageCertificate => "MarriageCertificate",
        DocumentType::DeathCertificate => "DeathCertificate",
        DocumentType::PoliceReport => "PoliceReport",
        DocumentType::CarTitle => "CarTitle",
        DocumentType::HealthInsurance => "HealthInsurance",
        DocumentType::LifeInsurance => "LifeInsurance",
        DocumentType::PropertyInsurance => "PropertyInsurance",
        DocumentType::LoanApplication => "LoanApplication",
        DocumentType::CollateralDocument => "CollateralDocument",
        DocumentType::GuarantorLetter => "GuarantorLetter",
        DocumentType::SavingsGroupMembership => "SavingsGroupMembership",
        DocumentType::AgriculturalLoanDocumentation => "AgriculturalLoanDocumentation",
        DocumentType::ReferenceLetter => "ReferenceLetter",
        DocumentType::ShariaComplianceCertificate => "ShariaComplianceCertificate",
        DocumentType::WakalaAgreement => "WakalaAgreement",
        DocumentType::MudarabaAgreement => "MudarabaAgreement",
        DocumentType::MurabahaAgreement => "MurabahaAgreement",
        DocumentType::IjaraAgreement => "IjaraAgreement",
        DocumentType::MusharakaAgreement => "MusharakaAgreement",
        DocumentType::TakafulCertificate => "TakafulCertificate",
        DocumentType::ZakatCertificate => "ZakatCertificate",
        DocumentType::HalalCertification => "HalalCertification",
        DocumentType::Other => "Other",
    };
    serializer.serialize_str(value_str)
}

fn deserialize_document_type<'de, D>(deserializer: D) -> Result<DocumentType, D::Error>
where
    D: Deserializer<'de>,
{
    let value_str = String::deserialize(deserializer)?;
    match value_str.as_str() {
        "Passport" => Ok(DocumentType::Passport),
        "NationalIdCard" => Ok(DocumentType::NationalIdCard),
        "DriverLicense" => Ok(DocumentType::DriverLicense),
        "ResidencePermit" => Ok(DocumentType::ResidencePermit),
        "VoterIdCard" => Ok(DocumentType::VoterIdCard),
        "AsylumCard" => Ok(DocumentType::AsylumCard),
        "IdApplicationReceipt" => Ok(DocumentType::IdApplicationReceipt),
        "BirthCertificate" => Ok(DocumentType::BirthCertificate),
        "UtilityBill" => Ok(DocumentType::UtilityBill),
        "BankStatement" => Ok(DocumentType::BankStatement),
        "TaxNotice" => Ok(DocumentType::TaxNotice),
        "TaxBill" => Ok(DocumentType::TaxBill),
        "LeaseAgreement" => Ok(DocumentType::LeaseAgreement),
        "PropertyDeed" => Ok(DocumentType::PropertyDeed),
        "LandTitle" => Ok(DocumentType::LandTitle),
        "EmploymentLetter" => Ok(DocumentType::EmploymentLetter),
        "PaySlip" => Ok(DocumentType::PaySlip),
        "TaxReturn" => Ok(DocumentType::TaxReturn),
        "IncomeStatement" => Ok(DocumentType::IncomeStatement),
        "FinancialStatement" => Ok(DocumentType::FinancialStatement),
        "CreditReport" => Ok(DocumentType::CreditReport),
        "CertificateOfIncorporation" => Ok(DocumentType::CertificateOfIncorporation),
        "ArticlesOfAssociation" => Ok(DocumentType::ArticlesOfAssociation),
        "TaxIdentificationNumber" => Ok(DocumentType::TaxIdentificationNumber),
        "BusinessLicense" => Ok(DocumentType::BusinessLicense),
        "ShareholderRegister" => Ok(DocumentType::ShareholderRegister),
        "BusinessPlan" => Ok(DocumentType::BusinessPlan),
        "AuditorsReport" => Ok(DocumentType::AuditorsReport),
        "ProofOfFunds" => Ok(DocumentType::ProofOfFunds),
        "CriminalRecordCheck" => Ok(DocumentType::CriminalRecordCheck),
        "CourtOrder" => Ok(DocumentType::CourtOrder),
        "CourtDocument" => Ok(DocumentType::CourtDocument),
        "PowerOfAttorney" => Ok(DocumentType::PowerOfAttorney),
        "BeneficialOwnershipDeclaration" => Ok(DocumentType::BeneficialOwnershipDeclaration),
        "MarriageCertificate" => Ok(DocumentType::MarriageCertificate),
        "DeathCertificate" => Ok(DocumentType::DeathCertificate),
        "PoliceReport" => Ok(DocumentType::PoliceReport),
        "CarTitle" => Ok(DocumentType::CarTitle),
        "HealthInsurance" => Ok(DocumentType::HealthInsurance),
        "LifeInsurance" => Ok(DocumentType::LifeInsurance),
        "PropertyInsurance" => Ok(DocumentType::PropertyInsurance),
        "LoanApplication" => Ok(DocumentType::LoanApplication),
        "CollateralDocument" => Ok(DocumentType::CollateralDocument),
        "GuarantorLetter" => Ok(DocumentType::GuarantorLetter),
        "SavingsGroupMembership" => Ok(DocumentType::SavingsGroupMembership),
        "AgriculturalLoanDocumentation" => Ok(DocumentType::AgriculturalLoanDocumentation),
        "ReferenceLetter" => Ok(DocumentType::ReferenceLetter),
        "ShariaComplianceCertificate" => Ok(DocumentType::ShariaComplianceCertificate),
        "WakalaAgreement" => Ok(DocumentType::WakalaAgreement),
        "MudarabaAgreement" => Ok(DocumentType::MudarabaAgreement),
        "MurabahaAgreement" => Ok(DocumentType::MurabahaAgreement),
        "IjaraAgreement" => Ok(DocumentType::IjaraAgreement),
        "MusharakaAgreement" => Ok(DocumentType::MusharakaAgreement),
        "TakafulCertificate" => Ok(DocumentType::TakafulCertificate),
        "ZakatCertificate" => Ok(DocumentType::ZakatCertificate),
        "HalalCertification" => Ok(DocumentType::HalalCertification),
        "Other" => Ok(DocumentType::Other),
        _ => Err(serde::de::Error::custom(format!(
            "Invalid DocumentType: {value_str}"
        ))),
    }
}

impl std::fmt::Display for DocumentStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DocumentStatus::Uploaded => write!(f, "Uploaded"),
            DocumentStatus::Verified => write!(f, "Verified"),
            DocumentStatus::Rejected => write!(f, "Rejected"),
            DocumentStatus::Expired => write!(f, "Expired"),
        }
    }
}

impl FromStr for DocumentStatus {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Uploaded" => Ok(DocumentStatus::Uploaded),
            "Verified" => Ok(DocumentStatus::Verified),
            "Rejected" => Ok(DocumentStatus::Rejected),
            "Expired" => Ok(DocumentStatus::Expired),
            _ => Err(()),
        }
    }
}

fn serialize_document_status<S>(value: &DocumentStatus, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let value_str = match value {
        DocumentStatus::Uploaded => "Uploaded",
        DocumentStatus::Verified => "Verified",
        DocumentStatus::Rejected => "Rejected",
        DocumentStatus::Expired => "Expired",
    };
    serializer.serialize_str(value_str)
}

fn deserialize_document_status<'de, D>(deserializer: D) -> Result<DocumentStatus, D::Error>
where
    D: Deserializer<'de>,
{
    let value_str = String::deserialize(deserializer)?;
    match value_str.as_str() {
        "Uploaded" => Ok(DocumentStatus::Uploaded),
        "Verified" => Ok(DocumentStatus::Verified),
        "Rejected" => Ok(DocumentStatus::Rejected),
        "Expired" => Ok(DocumentStatus::Expired),
        _ => Err(serde::de::Error::custom(format!(
            "Invalid DocumentStatus: {value_str}"
        ))),
    }
}