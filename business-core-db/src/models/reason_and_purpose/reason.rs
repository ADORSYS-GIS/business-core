use heapless::String as HeaplessString;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use sqlx::FromRow;
use uuid::Uuid;
use std::collections::HashMap;
use crate::{HasPrimaryKey, IdxModelCache, Indexable};
use crate::models::{IndexAware, Identifiable, Index};
use crate::utils::hash_as_i64;

/// Database model for ReasonCategory enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "reason_category", rename_all = "PascalCase")]
pub enum ReasonCategory {
    // Loan related
    LoanPurpose,
    LoanRejection,
    
    // Account lifecycle
    AccountClosure,
    AccountSuspension,
    AccountReactivation,
    StatusChange,
    
    // Transaction related
    TransactionRejection,
    TransactionReversal,
    HoldReason,
    
    // Compliance
    Compliance,
    ComplianceFlag,
    AuditFinding,
    
    // AML/CTF Categories
    AmlAlert,
    AmlInvestigation,
    SuspiciousActivity,
    CtfRiskFlag,
    SanctionsHit,
    PepFlag,  // Politically Exposed Person
    HighRiskCountry,
    UnusualPattern,
    
    // KYC Categories
    KycMissingDocument,
    KycDocumentRejection,
    KycVerificationFailure,
    KycUpdateRequired,
    IdentityVerificationIssue,
    LocationVerificationIssue,
    SourceOfFundsRequired,
    
    // Customer service
    ComplaintReason,
    ServiceRequest,
    
    // System
    SystemGenerated,
    MaintenanceReason,
    
    // Other
    Other,
}

impl std::fmt::Display for ReasonCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReasonCategory::LoanPurpose => write!(f, "LoanPurpose"),
            ReasonCategory::LoanRejection => write!(f, "LoanRejection"),
            ReasonCategory::AccountClosure => write!(f, "AccountClosure"),
            ReasonCategory::AccountSuspension => write!(f, "AccountSuspension"),
            ReasonCategory::AccountReactivation => write!(f, "AccountReactivation"),
            ReasonCategory::StatusChange => write!(f, "StatusChange"),
            ReasonCategory::TransactionRejection => write!(f, "TransactionRejection"),
            ReasonCategory::TransactionReversal => write!(f, "TransactionReversal"),
            ReasonCategory::HoldReason => write!(f, "HoldReason"),
            ReasonCategory::Compliance => write!(f, "Compliance"),
            ReasonCategory::ComplianceFlag => write!(f, "ComplianceFlag"),
            ReasonCategory::AuditFinding => write!(f, "AuditFinding"),
            ReasonCategory::AmlAlert => write!(f, "AmlAlert"),
            ReasonCategory::AmlInvestigation => write!(f, "AmlInvestigation"),
            ReasonCategory::SuspiciousActivity => write!(f, "SuspiciousActivity"),
            ReasonCategory::CtfRiskFlag => write!(f, "CtfRiskFlag"),
            ReasonCategory::SanctionsHit => write!(f, "SanctionsHit"),
            ReasonCategory::PepFlag => write!(f, "PepFlag"),
            ReasonCategory::HighRiskCountry => write!(f, "HighRiskCountry"),
            ReasonCategory::UnusualPattern => write!(f, "UnusualPattern"),
            ReasonCategory::KycMissingDocument => write!(f, "KycMissingDocument"),
            ReasonCategory::KycDocumentRejection => write!(f, "KycDocumentRejection"),
            ReasonCategory::KycVerificationFailure => write!(f, "KycVerificationFailure"),
            ReasonCategory::KycUpdateRequired => write!(f, "KycUpdateRequired"),
            ReasonCategory::IdentityVerificationIssue => write!(f, "IdentityVerificationIssue"),
            ReasonCategory::LocationVerificationIssue => write!(f, "LocationVerificationIssue"),
            ReasonCategory::SourceOfFundsRequired => write!(f, "SourceOfFundsRequired"),
            ReasonCategory::ComplaintReason => write!(f, "ComplaintReason"),
            ReasonCategory::ServiceRequest => write!(f, "ServiceRequest"),
            ReasonCategory::SystemGenerated => write!(f, "SystemGenerated"),
            ReasonCategory::MaintenanceReason => write!(f, "MaintenanceReason"),
            ReasonCategory::Other => write!(f, "Other"),
        }
    }
}

impl std::str::FromStr for ReasonCategory {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "LoanPurpose" => Ok(ReasonCategory::LoanPurpose),
            "LoanRejection" => Ok(ReasonCategory::LoanRejection),
            "AccountClosure" => Ok(ReasonCategory::AccountClosure),
            "AccountSuspension" => Ok(ReasonCategory::AccountSuspension),
            "AccountReactivation" => Ok(ReasonCategory::AccountReactivation),
            "StatusChange" => Ok(ReasonCategory::StatusChange),
            "TransactionRejection" => Ok(ReasonCategory::TransactionRejection),
            "TransactionReversal" => Ok(ReasonCategory::TransactionReversal),
            "HoldReason" => Ok(ReasonCategory::HoldReason),
            "Compliance" => Ok(ReasonCategory::Compliance),
            "ComplianceFlag" => Ok(ReasonCategory::ComplianceFlag),
            "AuditFinding" => Ok(ReasonCategory::AuditFinding),
            "AmlAlert" => Ok(ReasonCategory::AmlAlert),
            "AmlInvestigation" => Ok(ReasonCategory::AmlInvestigation),
            "SuspiciousActivity" => Ok(ReasonCategory::SuspiciousActivity),
            "CtfRiskFlag" => Ok(ReasonCategory::CtfRiskFlag),
            "SanctionsHit" => Ok(ReasonCategory::SanctionsHit),
            "PepFlag" => Ok(ReasonCategory::PepFlag),
            "HighRiskCountry" => Ok(ReasonCategory::HighRiskCountry),
            "UnusualPattern" => Ok(ReasonCategory::UnusualPattern),
            "KycMissingDocument" => Ok(ReasonCategory::KycMissingDocument),
            "KycDocumentRejection" => Ok(ReasonCategory::KycDocumentRejection),
            "KycVerificationFailure" => Ok(ReasonCategory::KycVerificationFailure),
            "KycUpdateRequired" => Ok(ReasonCategory::KycUpdateRequired),
            "IdentityVerificationIssue" => Ok(ReasonCategory::IdentityVerificationIssue),
            "LocationVerificationIssue" => Ok(ReasonCategory::LocationVerificationIssue),
            "SourceOfFundsRequired" => Ok(ReasonCategory::SourceOfFundsRequired),
            "ComplaintReason" => Ok(ReasonCategory::ComplaintReason),
            "ServiceRequest" => Ok(ReasonCategory::ServiceRequest),
            "SystemGenerated" => Ok(ReasonCategory::SystemGenerated),
            "MaintenanceReason" => Ok(ReasonCategory::MaintenanceReason),
            "Other" => Ok(ReasonCategory::Other),
            _ => Err(format!("Invalid ReasonCategory: {s}")),
        }
    }
}

/// Database model for ReasonContext enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "reason_context", rename_all = "PascalCase")]
pub enum ReasonContext {
    Account,
    Loan,
    Transaction,
    Customer,
    Compliance,
    AmlCtf,        // Anti-Money Laundering / Counter-Terrorism Financing
    Kyc,           // Know Your Customer
    System,
    General,
}

impl std::fmt::Display for ReasonContext {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReasonContext::Account => write!(f, "Account"),
            ReasonContext::Loan => write!(f, "Loan"),
            ReasonContext::Transaction => write!(f, "Transaction"),
            ReasonContext::Customer => write!(f, "Customer"),
            ReasonContext::Compliance => write!(f, "Compliance"),
            ReasonContext::AmlCtf => write!(f, "AmlCtf"),
            ReasonContext::Kyc => write!(f, "Kyc"),
            ReasonContext::System => write!(f, "System"),
            ReasonContext::General => write!(f, "General"),
        }
    }
}

impl std::str::FromStr for ReasonContext {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Account" => Ok(ReasonContext::Account),
            "Loan" => Ok(ReasonContext::Loan),
            "Transaction" => Ok(ReasonContext::Transaction),
            "Customer" => Ok(ReasonContext::Customer),
            "Compliance" => Ok(ReasonContext::Compliance),
            "AmlCtf" => Ok(ReasonContext::AmlCtf),
            "Kyc" => Ok(ReasonContext::Kyc),
            "System" => Ok(ReasonContext::System),
            "General" => Ok(ReasonContext::General),
            _ => Err(format!("Invalid ReasonContext: {s}")),
        }
    }
}

/// Database model for ReasonSeverity enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "reason_severity", rename_all = "PascalCase")]
pub enum ReasonSeverity {
    Critical,
    High,
    Medium,
    Low,
    Informational,
}

impl std::fmt::Display for ReasonSeverity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ReasonSeverity::Critical => write!(f, "Critical"),
            ReasonSeverity::High => write!(f, "High"),
            ReasonSeverity::Medium => write!(f, "Medium"),
            ReasonSeverity::Low => write!(f, "Low"),
            ReasonSeverity::Informational => write!(f, "Informational"),
        }
    }
}

impl std::str::FromStr for ReasonSeverity {
    type Err = String;
    
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Critical" => Ok(ReasonSeverity::Critical),
            "High" => Ok(ReasonSeverity::High),
            "Medium" => Ok(ReasonSeverity::Medium),
            "Low" => Ok(ReasonSeverity::Low),
            "Informational" => Ok(ReasonSeverity::Informational),
            _ => Err(format!("Invalid ReasonSeverity: {s}")),
        }
    }
}

/// # Documentation
/// - Reason entity for capturing transaction purposes and explanations
/// - Supports multi-language content (up to 3 languages)
/// - Provides categorization and context for different use cases
/// - Includes compliance metadata linking for AML/CTF/KYC purposes
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReasonModel {
    pub id: Uuid,
    
    /// Unique identifier code for programmatic reference
    pub code: HeaplessString<50>,
    
    /// Category to group related reasons
    #[serde(
        serialize_with = "serialize_reason_category",
        deserialize_with = "deserialize_reason_category"
    )]
    pub category: ReasonCategory,
    
    /// Context where this reason is used
    #[serde(
        serialize_with = "serialize_reason_context",
        deserialize_with = "deserialize_reason_context"
    )]
    pub context: ReasonContext,
    
    /// Language content - up to 3 languages supported
    pub l1_content: Option<HeaplessString<100>>,
    pub l2_content: Option<HeaplessString<100>>,
    pub l3_content: Option<HeaplessString<100>>,
    
    /// Language codes for each content field
    pub l1_language_code: Option<HeaplessString<3>>,
    pub l2_language_code: Option<HeaplessString<3>>,
    pub l3_language_code: Option<HeaplessString<3>>,
    
    /// Whether this reason requires additional details
    pub requires_details: bool,
    
    /// Whether this reason is currently active
    pub is_active: bool,
    
    /// Severity or importance level
    #[serde(
        serialize_with = "serialize_reason_severity_option",
        deserialize_with = "deserialize_reason_severity_option"
    )]
    pub severity: Option<ReasonSeverity>,
    
    /// Sort order for UI display
    pub display_order: i32,
    
    /// Compliance-specific metadata (for AML/CTF/KYC reasons)
    pub compliance_metadata: Option<Uuid>,
}

// Custom serialization functions for database compatibility
fn serialize_reason_category<S>(category: &ReasonCategory, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&category.to_string())
}

fn deserialize_reason_category<'de, D>(deserializer: D) -> Result<ReasonCategory, D::Error>
where
    D: Deserializer<'de>,
{
    let category_str = String::deserialize(deserializer)?;
    category_str.parse().map_err(serde::de::Error::custom)
}

fn serialize_reason_context<S>(context: &ReasonContext, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&context.to_string())
}

fn deserialize_reason_context<'de, D>(deserializer: D) -> Result<ReasonContext, D::Error>
where
    D: Deserializer<'de>,
{
    let context_str = String::deserialize(deserializer)?;
    context_str.parse().map_err(serde::de::Error::custom)
}

fn serialize_reason_severity_option<S>(severity: &Option<ReasonSeverity>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    match severity {
        Some(severity) => serializer.serialize_str(&severity.to_string()),
        None => serializer.serialize_none(),
    }
}

fn deserialize_reason_severity_option<'de, D>(deserializer: D) -> Result<Option<ReasonSeverity>, D::Error>
where
    D: Deserializer<'de>,
{
    let option_str: Option<String> = Option::deserialize(deserializer)?;
    match option_str {
        Some(severity_str) => {
            let severity = severity_str.parse().map_err(serde::de::Error::custom)?;
            Ok(Some(severity))
        }
        None => Ok(None),
    }
}

impl ReasonModel {
    /// Get content in specified language, fallback to primary if not available
    pub fn get_content(&self, language_code: &[u8; 3]) -> Option<&str> {
        if self.l1_language_code.as_ref().map(|s| s.as_bytes()) == Some(language_code) {
            self.l1_content.as_deref()
        } else if self.l2_language_code.as_ref().map(|s| s.as_bytes()) == Some(language_code) {
            self.l2_content.as_deref()
        } else if self.l3_language_code.as_ref().map(|s| s.as_bytes()) == Some(language_code) {
            self.l3_content.as_deref()
        } else {
            // Fallback to primary language
            self.l1_content.as_deref()
        }
    }
    
    /// Get content with fallback chain
    pub fn get_content_with_fallback(&self, preferred_languages: &[[u8; 3]]) -> Option<&str> {
        for lang in preferred_languages {
            if let Some(content) = self.get_content(lang) {
                return Some(content);
            }
        }
        // Final fallback to any available content
        self.l1_content.as_deref()
            .or(self.l2_content.as_deref())
            .or(self.l3_content.as_deref())
    }
    
    /// Check if reason has content in specified language
    pub fn has_language(&self, language_code: &[u8; 3]) -> bool {
        self.l1_language_code.as_ref().map(|s| s.as_bytes()) == Some(language_code) ||
        self.l2_language_code.as_ref().map(|s| s.as_bytes()) == Some(language_code) ||
        self.l3_language_code.as_ref().map(|s| s.as_bytes()) == Some(language_code)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ReasonIdxModel {
    pub id: Uuid,
    pub code_hash: i64,
    pub category_hash: i64,
    pub context_hash: i64,
    pub compliance_metadata: Option<Uuid>,
}

impl HasPrimaryKey for ReasonIdxModel {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}

impl Identifiable for ReasonModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl IndexAware for ReasonModel {
    type IndexType = ReasonIdxModel;
    
    fn to_index(&self) -> Self::IndexType {
        let code_hash = hash_as_i64(&self.code.as_str()).unwrap();
        let category_hash = hash_as_i64(&self.category.to_string()).unwrap();
        let context_hash = hash_as_i64(&self.context.to_string()).unwrap();
        
        ReasonIdxModel {
            id: self.id,
            code_hash,
            category_hash,
            context_hash,
            compliance_metadata: self.compliance_metadata,
        }
    }
}

impl Identifiable for ReasonIdxModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Index for ReasonIdxModel {}

impl Indexable for ReasonIdxModel {
    fn i64_keys(&self) -> HashMap<String, Option<i64>> {
        let mut keys = HashMap::new();
        keys.insert("code_hash".to_string(), Some(self.code_hash));
        keys.insert("category_hash".to_string(), Some(self.category_hash));
        keys.insert("context_hash".to_string(), Some(self.context_hash));
        keys
    }

    fn uuid_keys(&self) -> HashMap<String, Option<Uuid>> {
        let mut keys = HashMap::new();
        keys.insert(
            "compliance_metadata".to_string(),
            self.compliance_metadata,
        );
        keys
    }
}

pub type ReasonIdxModelCache = IdxModelCache<ReasonIdxModel>;