use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use uuid::Uuid;

/// Represents a banking product in the database.
/// # Audit
/// # Index
/// # Cache
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductModel {
    pub id: Uuid,
    pub name: Uuid,
    pub product_type: ProductType,
    pub minimum_balance: Decimal,
    pub maximum_balance: Option<Decimal>,
    pub overdraft_allowed: bool,
    pub overdraft_limit: Option<Decimal>,
    pub interest_calculation_method: InterestCalculationMethod,
    pub interest_posting_frequency: PostingFrequency,
    pub dormancy_threshold_days: i32,
    pub minimum_opening_balance: Decimal,
    pub closure_fee: Decimal,
    pub maintenance_fee: Option<Decimal>,
    pub maintenance_fee_frequency: Option<MaintenanceFeeFrequency>,
    pub default_dormancy_days: Option<i32>,
    pub default_overdraft_limit: Option<Decimal>,
    pub per_transaction_limit: Option<Decimal>,
    pub daily_transaction_limit: Option<Decimal>,
    pub weekly_transaction_limit: Option<Decimal>,
    pub monthly_transaction_limit: Option<Decimal>,
    pub overdraft_interest_rate: Option<Decimal>,
    pub accrual_frequency: ProductAccrualFrequency,
    pub interest_rate_tier_1: Option<Uuid>,
    pub interest_rate_tier_2: Option<Uuid>,
    pub interest_rate_tier_3: Option<Uuid>,
    pub interest_rate_tier_4: Option<Uuid>,
    pub interest_rate_tier_5: Option<Uuid>,
    pub account_gl_mapping: Uuid,
    pub fee_type_gl_mapping: Uuid,
    pub is_active: bool,
    pub valid_from: NaiveDate,
    pub valid_to: Option<NaiveDate>,
}

/// The type of banking product.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProductType {
    CASA,
    LOAN,
}

// Display implementations for database compatibility
impl std::fmt::Display for ProductType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ProductType::CASA => write!(f, "CASA"),
            ProductType::LOAN => write!(f, "LOAN"),
        }
    }
}

impl std::str::FromStr for ProductType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "CASA" => Ok(ProductType::CASA),
            "LOAN" => Ok(ProductType::LOAN),
            _ => Err(format!("Invalid ProductType: {s}")),
        }
    }
}

/// Frequency for interest posting
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum PostingFrequency {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Annually,
}

/// Interest calculation method for CASA products (conventional and Islamic banking)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum InterestCalculationMethod {
    /// Daily accrual method: Interest calculated on daily closing balance (rate/365 or rate/360)
    /// Most common modern method. Used in India (RBI-mandated since 2010), US, EU, Asian banks
    DailyBalance,
    
    /// Average daily balance: Interest on average of daily closing balances over the period
    /// Very common in US, Canada, Australia, some European banks
    AverageDailyBalance,
    
    /// Minimum balance: Interest on lowest balance during the period (month/quarter)
    /// Very common in India and emerging markets (Pakistan, Bangladesh, Africa)
    MinimumBalance,
    
    /// Simple interest: Interest = Principal × Rate × Time
    /// Rarely used for modern savings accounts
    Simple,
    
    /// Compound interest (general): Interest on principal + accumulated interest
    /// Most savings accounts are a specific form of this
    Compound,
    
    // Islamic Banking Methods (Shariah-compliant, no "interest" - uses profit-sharing)
    
    /// Mudarabah: Profit-sharing investment account
    /// Bank acts as Mudarib (entrepreneur), customer as Rab-ul-Mal (capital provider)
    /// Profits shared per agreed ratio, losses borne by capital provider
    /// Used in Malaysia, GCC countries, Pakistan, Indonesia
    Mudarabah,
    
    /// Musharakah: Partnership/joint venture profit-sharing
    /// Both bank and customer contribute capital and share profits/losses
    /// Less common for savings, more for investment accounts
    Musharakah,
    
    /// Wakalah: Agency-based investment
    /// Bank acts as agent (Wakeel) investing customer funds for a fee
    /// Returns belong to customer minus agreed fee
    /// Common in Takaful and some investment accounts
    Wakalah,
    
    /// Qard Hasan: Benevolent loan with no return
    /// Zero-return account, purely for safekeeping
    /// Used for current accounts in Islamic banks
    QardHasan,
}

// Display implementation for database compatibility
impl std::fmt::Display for InterestCalculationMethod {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            InterestCalculationMethod::DailyBalance => write!(f, "DailyBalance"),
            InterestCalculationMethod::AverageDailyBalance => write!(f, "AverageDailyBalance"),
            InterestCalculationMethod::MinimumBalance => write!(f, "MinimumBalance"),
            InterestCalculationMethod::Simple => write!(f, "Simple"),
            InterestCalculationMethod::Compound => write!(f, "Compound"),
            InterestCalculationMethod::Mudarabah => write!(f, "Mudarabah"),
            InterestCalculationMethod::Musharakah => write!(f, "Musharakah"),
            InterestCalculationMethod::Wakalah => write!(f, "Wakalah"),
            InterestCalculationMethod::QardHasan => write!(f, "QardHasan"),
        }
    }
}

impl std::str::FromStr for InterestCalculationMethod {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "DailyBalance" => Ok(InterestCalculationMethod::DailyBalance),
            "AverageDailyBalance" => Ok(InterestCalculationMethod::AverageDailyBalance),
            "MinimumBalance" => Ok(InterestCalculationMethod::MinimumBalance),
            "Simple" => Ok(InterestCalculationMethod::Simple),
            "Compound" => Ok(InterestCalculationMethod::Compound),
            "Mudarabah" => Ok(InterestCalculationMethod::Mudarabah),
            "Musharakah" => Ok(InterestCalculationMethod::Musharakah),
            "Wakalah" => Ok(InterestCalculationMethod::Wakalah),
            "QardHasan" => Ok(InterestCalculationMethod::QardHasan),
            _ => Err(format!("Invalid InterestCalculationMethod: {s}")),
        }
    }
}

/// Frequency for maintenance fee charges
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum MaintenanceFeeFrequency {
    /// Fee charged daily
    Daily,
    
    /// Fee charged weekly
    Weekly,
    
    /// Fee charged bi-weekly (every 2 weeks)
    BiWeekly,
    
    /// Fee charged monthly - most common
    Monthly,
    
    /// Fee charged bi-monthly (every 2 months)
    BiMonthly,
    
    /// Fee charged quarterly (every 3 months)
    Quarterly,
    
    /// Fee charged semi-annually (every 6 months)
    SemiAnnually,
    
    /// Fee charged annually
    Annually,
    
    /// One-time fee only
    OneTime,
}

// Display implementation for database compatibility
impl std::fmt::Display for MaintenanceFeeFrequency {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaintenanceFeeFrequency::Daily => write!(f, "Daily"),
            MaintenanceFeeFrequency::Weekly => write!(f, "Weekly"),
            MaintenanceFeeFrequency::BiWeekly => write!(f, "BiWeekly"),
            MaintenanceFeeFrequency::Monthly => write!(f, "Monthly"),
            MaintenanceFeeFrequency::BiMonthly => write!(f, "BiMonthly"),
            MaintenanceFeeFrequency::Quarterly => write!(f, "Quarterly"),
            MaintenanceFeeFrequency::SemiAnnually => write!(f, "SemiAnnually"),
            MaintenanceFeeFrequency::Annually => write!(f, "Annually"),
            MaintenanceFeeFrequency::OneTime => write!(f, "OneTime"),
        }
    }
}

impl std::str::FromStr for MaintenanceFeeFrequency {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "Daily" => Ok(MaintenanceFeeFrequency::Daily),
            "Weekly" => Ok(MaintenanceFeeFrequency::Weekly),
            "BiWeekly" => Ok(MaintenanceFeeFrequency::BiWeekly),
            "Monthly" => Ok(MaintenanceFeeFrequency::Monthly),
            "BiMonthly" => Ok(MaintenanceFeeFrequency::BiMonthly),
            "Quarterly" => Ok(MaintenanceFeeFrequency::Quarterly),
            "SemiAnnually" => Ok(MaintenanceFeeFrequency::SemiAnnually),
            "Annually" => Ok(MaintenanceFeeFrequency::Annually),
            "OneTime" => Ok(MaintenanceFeeFrequency::OneTime),
            _ => Err(format!("Invalid MaintenanceFeeFrequency: {s}")),
        }
    }
}

/// Frequency for interest accrual
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ProductAccrualFrequency {
    Daily,
    BusinessDaysOnly,
    None,
}
