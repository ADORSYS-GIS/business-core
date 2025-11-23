use crate::models::auditable::Auditable;
use crate::models::identifiable::Identifiable;
use crate::{HasPrimaryKey, IdxModelCache, Indexable};
use crate::models::{Index, IndexAware};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use std::collections::HashMap;

pub fn deserialize_gl_code<'de, D>(deserializer: D) -> Result<heapless::String<50>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    std::str::FromStr::from_str(&s).map_err(|_| {
        serde::de::Error::custom("Value for gl_code is too long (max 50 chars)")
    })
}

/// Fee-specific GL mapping that allows flexible fee type management
/// Supports both conventional and Islamic banking fee structures
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FeeTypeGlMappingModel {
    pub id: Uuid,
    pub fee_type: FeeType,
    #[serde(deserialize_with = "deserialize_gl_code")]
    pub gl_code: heapless::String<50>,

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

impl Identifiable for FeeTypeGlMappingModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Auditable for FeeTypeGlMappingModel {
    fn get_audit_log_id(&self) -> Option<Uuid> {
        self.audit_log_id
    }
}

/// As the FeeTypeGlMappingModel is tiny, we can keep main model
/// info here and use the model to perform work.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct FeeTypeGlMappingIdxModel {
    pub id: Uuid,

    // - Not a secondary Index. Do not provide any finder!
    pub fee_type: FeeType,

    // - Not a secondary Index. Do not provide any finder!
    #[serde(deserialize_with = "deserialize_gl_code")]
    pub gl_code: heapless::String<50>,
}

impl HasPrimaryKey for FeeTypeGlMappingIdxModel {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}

impl IndexAware for FeeTypeGlMappingModel {
    type IndexType = FeeTypeGlMappingIdxModel;
    
    fn to_index(&self) -> Self::IndexType {
        FeeTypeGlMappingIdxModel {
            id: self.id,
            fee_type: self.fee_type.clone(),
            gl_code: self.gl_code.clone(),
        }
    }
}

impl Identifiable for FeeTypeGlMappingIdxModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Index for FeeTypeGlMappingIdxModel {}

impl Indexable for FeeTypeGlMappingIdxModel {
    fn i64_keys(&self) -> HashMap<String, Option<i64>> {
        HashMap::new()
    }

    fn uuid_keys(&self) -> HashMap<String, Option<Uuid>> {
        HashMap::new()
    }
}

pub type FeeTypeGlMappingIdxModelCache = IdxModelCache<FeeTypeGlMappingIdxModel>;

/// Comprehensive fee types for banking products
/// Supports both conventional and Islamic (Shariah-compliant) banking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash, sqlx::Type)]
#[sqlx(type_name = "fee_type", rename_all = "PascalCase")]
pub enum FeeType {
    // ===== CONVENTIONAL BANKING =====
    /// Interest expense for conventional savings/deposit accounts
    InterestExpense,

    /// General fee income (catch-all)
    GeneralFeeIncome,

    // ----- Transaction Fees -----
    /// ATM withdrawal fee at own bank ATMs
    AtmWithdrawalOwn,

    /// ATM withdrawal fee at other bank ATMs
    AtmWithdrawalOther,

    /// Domestic wire/bank transfer fee
    TransferDomestic,

    /// International wire transfer fee
    TransferInternational,

    /// Debit card transaction fee
    DebitCardTransaction,

    /// Credit card transaction fee
    CreditCardTransaction,

    /// Check processing/clearing fee
    CheckProcessing,

    /// Stop payment order fee
    StopPayment,

    /// Cash deposit fee (over certain limits)
    CashDeposit,

    /// Cash withdrawal fee (over certain limits)
    CashWithdrawal,

    // ----- Account Fees -----
    /// Monthly account maintenance fee
    MaintenanceFee,

    /// Minimum balance penalty fee
    MinimumBalancePenalty,

    /// Account opening fee
    AccountOpening,

    /// Account closure fee
    AccountClosure,

    /// Dormant account reactivation fee
    DormancyReactivation,

    /// Paper statement fee (when e-statements are standard)
    StatementPaper,

    /// Statement copy/duplicate request fee
    StatementCopy,

    // ----- Service Fees -----
    /// SMS banking alert fees
    SmsAlert,

    /// Email alert fees
    EmailAlert,

    /// Checkbook issuance fee
    CheckbookIssuance,

    /// Debit card issuance fee
    DebitCardIssuance,

    /// Debit card replacement fee
    DebitCardReplacement,

    /// Credit card annual fee
    CreditCardAnnual,

    /// Foreign currency conversion fee
    ForeignExchange,

    /// Account certificate/letter fee
    AccountCertificate,

    /// Balance inquiry fee (non-digital channels)
    BalanceInquiry,

    // ----- Penalties/NSF -----
    /// Non-sufficient funds (NSF) / bounced check fee
    NsfFee,

    /// Overdraft penalty fee
    OverdraftPenalty,

    /// Over-limit fee (exceeding account limits)
    OverLimitFee,

    /// Late payment penalty (for loans/credit)
    LatePaymentPenalty,

    /// Returned item fee
    ReturnedItem,

    /// General penalty fee
    GeneralPenalty,

    // ===== ISLAMIC BANKING (SHARIAH-COMPLIANT) =====
    // ----- Profit Distribution (replaces interest) -----
    /// Mudarabah profit distribution (profit-sharing investment)
    MudarabahProfit,

    /// Musharakah profit distribution (partnership profit)
    MusharakahProfit,

    /// Musharakah loss distribution (partnership loss)
    MusharakahLoss,

    /// Wakalah investment management fee
    WakalahFee,

    /// Hibah - discretionary gift from bank to customer
    Hibah,

    // ----- Qard Hasan (Benevolent Loan) -----
    /// Qard Hasan administrative fee (minimal, cost-recovery only)
    QardHasanAdminFee,

    // ----- Charity (for penalties) -----
    /// Charity account for penalty amounts (Shariah requires penalties go to charity, not bank profit)
    CharityPenalty,

    // ----- Islamic Service Fees -----
    /// Ujrah - service fee for actual services rendered (Shariah-compliant)
    UjrahServiceFee,

    /// Takaful (Islamic insurance) contribution
    TakafulContribution,

    // ----- Additional Common Fees -----
    /// Safekeeping/custody fee
    SafekeepingFee,

    /// Document processing fee
    DocumentProcessing,

    /// Account research/investigation fee
    AccountResearch,

    /// Third-party service fee
    ThirdPartyService,

    /// Custom/other fee type (use sparingly)
    Other,
}