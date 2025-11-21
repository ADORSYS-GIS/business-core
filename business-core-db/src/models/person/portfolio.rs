use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use crate::models::auditable::Auditable;
use crate::models::identifiable::Identifiable;

/// # Documentation
/// Database model for Customer Portfolio summary
/// 
/// Tracks aggregated portfolio information for a person including:
/// - Total number of accounts
/// - Total balance across all accounts
/// - Outstanding loan amounts (as main borrower and as grantor)
/// - Risk scoring
/// - Compliance status reference
///
/// This entity is auditable but not indexable - accessed by ID only.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct PortfolioModel {
    pub id: Uuid,
    
    /// Reference to the person who owns this portfolio
    pub person_id: Uuid,
    
    /// Total number of accounts in the portfolio
    pub total_accounts: i64,
    
    /// Total balance across all accounts
    pub total_balance: Decimal,
    
    /// Total outstanding loan amount as main borrower
    pub total_loan_outstanding_main: Option<Decimal>,
    
    /// Total outstanding loan amount as grantor
    pub total_loan_outstanding_grantor: Option<Decimal>,
    
    /// Risk score for the portfolio
    pub risk_score: Option<Decimal>,
    
    /// Reference to compliance status
    pub compliance_status: Uuid,
    
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

impl Identifiable for PortfolioModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Auditable for PortfolioModel {
    fn get_audit_log_id(&self) -> Option<Uuid> {
        self.audit_log_id
    }
}