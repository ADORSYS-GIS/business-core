use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Fee-specific GL mapping that allows flexible fee type management
/// Supports both conventional and Islamic banking fee structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeeTypeGlMappingModel {
    pub id: Uuid,
    pub fee_type: FeeType,
    pub gl_code: heapless::String<50>,
}

/// Comprehensive fee types for banking products
/// Supports both conventional and Islamic (Shariah-compliant) banking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
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

// Display implementation for database compatibility
impl std::fmt::Display for FeeType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            // Conventional Banking
            FeeType::InterestExpense => write!(f, "InterestExpense"),
            FeeType::GeneralFeeIncome => write!(f, "GeneralFeeIncome"),
            
            // Transaction Fees
            FeeType::AtmWithdrawalOwn => write!(f, "AtmWithdrawalOwn"),
            FeeType::AtmWithdrawalOther => write!(f, "AtmWithdrawalOther"),
            FeeType::TransferDomestic => write!(f, "TransferDomestic"),
            FeeType::TransferInternational => write!(f, "TransferInternational"),
            FeeType::DebitCardTransaction => write!(f, "DebitCardTransaction"),
            FeeType::CreditCardTransaction => write!(f, "CreditCardTransaction"),
            FeeType::CheckProcessing => write!(f, "CheckProcessing"),
            FeeType::StopPayment => write!(f, "StopPayment"),
            FeeType::CashDeposit => write!(f, "CashDeposit"),
            FeeType::CashWithdrawal => write!(f, "CashWithdrawal"),
            
            // Account Fees
            FeeType::MaintenanceFee => write!(f, "MaintenanceFee"),
            FeeType::MinimumBalancePenalty => write!(f, "MinimumBalancePenalty"),
            FeeType::AccountOpening => write!(f, "AccountOpening"),
            FeeType::AccountClosure => write!(f, "AccountClosure"),
            FeeType::DormancyReactivation => write!(f, "DormancyReactivation"),
            FeeType::StatementPaper => write!(f, "StatementPaper"),
            FeeType::StatementCopy => write!(f, "StatementCopy"),
            
            // Service Fees
            FeeType::SmsAlert => write!(f, "SmsAlert"),
            FeeType::EmailAlert => write!(f, "EmailAlert"),
            FeeType::CheckbookIssuance => write!(f, "CheckbookIssuance"),
            FeeType::DebitCardIssuance => write!(f, "DebitCardIssuance"),
            FeeType::DebitCardReplacement => write!(f, "DebitCardReplacement"),
            FeeType::CreditCardAnnual => write!(f, "CreditCardAnnual"),
            FeeType::ForeignExchange => write!(f, "ForeignExchange"),
            FeeType::AccountCertificate => write!(f, "AccountCertificate"),
            FeeType::BalanceInquiry => write!(f, "BalanceInquiry"),
            
            // Penalties/NSF
            FeeType::NsfFee => write!(f, "NsfFee"),
            FeeType::OverdraftPenalty => write!(f, "OverdraftPenalty"),
            FeeType::OverLimitFee => write!(f, "OverLimitFee"),
            FeeType::LatePaymentPenalty => write!(f, "LatePaymentPenalty"),
            FeeType::ReturnedItem => write!(f, "ReturnedItem"),
            FeeType::GeneralPenalty => write!(f, "GeneralPenalty"),
            
            // Islamic Banking
            FeeType::MudarabahProfit => write!(f, "MudarabahProfit"),
            FeeType::MusharakahProfit => write!(f, "MusharakahProfit"),
            FeeType::MusharakahLoss => write!(f, "MusharakahLoss"),
            FeeType::WakalahFee => write!(f, "WakalahFee"),
            FeeType::Hibah => write!(f, "Hibah"),
            FeeType::QardHasanAdminFee => write!(f, "QardHasanAdminFee"),
            FeeType::CharityPenalty => write!(f, "CharityPenalty"),
            FeeType::UjrahServiceFee => write!(f, "UjrahServiceFee"),
            FeeType::TakafulContribution => write!(f, "TakafulContribution"),
            
            // Additional
            FeeType::SafekeepingFee => write!(f, "SafekeepingFee"),
            FeeType::DocumentProcessing => write!(f, "DocumentProcessing"),
            FeeType::AccountResearch => write!(f, "AccountResearch"),
            FeeType::ThirdPartyService => write!(f, "ThirdPartyService"),
            FeeType::Other => write!(f, "Other"),
        }
    }
}

impl std::str::FromStr for FeeType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            // Conventional Banking
            "InterestExpense" => Ok(FeeType::InterestExpense),
            "GeneralFeeIncome" => Ok(FeeType::GeneralFeeIncome),
            
            // Transaction Fees
            "AtmWithdrawalOwn" => Ok(FeeType::AtmWithdrawalOwn),
            "AtmWithdrawalOther" => Ok(FeeType::AtmWithdrawalOther),
            "TransferDomestic" => Ok(FeeType::TransferDomestic),
            "TransferInternational" => Ok(FeeType::TransferInternational),
            "DebitCardTransaction" => Ok(FeeType::DebitCardTransaction),
            "CreditCardTransaction" => Ok(FeeType::CreditCardTransaction),
            "CheckProcessing" => Ok(FeeType::CheckProcessing),
            "StopPayment" => Ok(FeeType::StopPayment),
            "CashDeposit" => Ok(FeeType::CashDeposit),
            "CashWithdrawal" => Ok(FeeType::CashWithdrawal),
            
            // Account Fees
            "MaintenanceFee" => Ok(FeeType::MaintenanceFee),
            "MinimumBalancePenalty" => Ok(FeeType::MinimumBalancePenalty),
            "AccountOpening" => Ok(FeeType::AccountOpening),
            "AccountClosure" => Ok(FeeType::AccountClosure),
            "DormancyReactivation" => Ok(FeeType::DormancyReactivation),
            "StatementPaper" => Ok(FeeType::StatementPaper),
            "StatementCopy" => Ok(FeeType::StatementCopy),
            
            // Service Fees
            "SmsAlert" => Ok(FeeType::SmsAlert),
            "EmailAlert" => Ok(FeeType::EmailAlert),
            "CheckbookIssuance" => Ok(FeeType::CheckbookIssuance),
            "DebitCardIssuance" => Ok(FeeType::DebitCardIssuance),
            "DebitCardReplacement" => Ok(FeeType::DebitCardReplacement),
            "CreditCardAnnual" => Ok(FeeType::CreditCardAnnual),
            "ForeignExchange" => Ok(FeeType::ForeignExchange),
            "AccountCertificate" => Ok(FeeType::AccountCertificate),
            "BalanceInquiry" => Ok(FeeType::BalanceInquiry),
            
            // Penalties/NSF
            "NsfFee" => Ok(FeeType::NsfFee),
            "OverdraftPenalty" => Ok(FeeType::OverdraftPenalty),
            "OverLimitFee" => Ok(FeeType::OverLimitFee),
            "LatePaymentPenalty" => Ok(FeeType::LatePaymentPenalty),
            "ReturnedItem" => Ok(FeeType::ReturnedItem),
            "GeneralPenalty" => Ok(FeeType::GeneralPenalty),
            
            // Islamic Banking
            "MudarabahProfit" => Ok(FeeType::MudarabahProfit),
            "MusharakahProfit" => Ok(FeeType::MusharakahProfit),
            "MusharakahLoss" => Ok(FeeType::MusharakahLoss),
            "WakalahFee" => Ok(FeeType::WakalahFee),
            "Hibah" => Ok(FeeType::Hibah),
            "QardHasanAdminFee" => Ok(FeeType::QardHasanAdminFee),
            "CharityPenalty" => Ok(FeeType::CharityPenalty),
            "UjrahServiceFee" => Ok(FeeType::UjrahServiceFee),
            "TakafulContribution" => Ok(FeeType::TakafulContribution),
            
            // Additional
            "SafekeepingFee" => Ok(FeeType::SafekeepingFee),
            "DocumentProcessing" => Ok(FeeType::DocumentProcessing),
            "AccountResearch" => Ok(FeeType::AccountResearch),
            "ThirdPartyService" => Ok(FeeType::ThirdPartyService),
            "Other" => Ok(FeeType::Other),
            
            _ => Err(format!("Invalid FeeType: {s}")),
        }
    }
}