use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use rust_decimal::Decimal;
use sqlx::prelude::FromRow;
use uuid::Uuid;
use std::collections::HashMap;
use crate::models::auditable::Auditable;
use crate::models::identifiable::Identifiable;
use crate::{HasPrimaryKey, IdxModelCache, Indexable};
use crate::models::{Index, IndexAware};

/// Represents a banking product in the database.
/// 
/// This entity is auditable and indexable - accessed by ID with comprehensive audit tracking.
/// Supports various product types (CASA, LOAN) with configurable interest calculation methods,
/// transaction limits, and fee structures.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductModel {
    pub id: Uuid,
    pub name: Uuid,
    #[serde(serialize_with = "serialize_product_type", deserialize_with = "deserialize_product_type")]
    pub product_type: ProductType,
    pub minimum_balance: Decimal,
    pub maximum_balance: Option<Decimal>,
    pub overdraft_allowed: bool,
    pub overdraft_limit: Option<Decimal>,
    #[serde(serialize_with = "serialize_interest_calculation_method", deserialize_with = "deserialize_interest_calculation_method")]
    pub interest_calculation_method: InterestCalculationMethod,
    #[serde(serialize_with = "serialize_posting_frequency", deserialize_with = "deserialize_posting_frequency")]
    pub interest_posting_frequency: PostingFrequency,
    pub dormancy_threshold_days: i32,
    pub minimum_opening_balance: Decimal,
    pub closure_fee: Decimal,
    pub maintenance_fee: Option<Decimal>,
    #[serde(default, serialize_with = "serialize_maintenance_fee_frequency", deserialize_with = "deserialize_maintenance_fee_frequency")]
    pub maintenance_fee_frequency: MaintenanceFeeFrequency,
    pub default_dormancy_days: Option<i32>,
    pub default_overdraft_limit: Option<Decimal>,
    pub per_transaction_limit: Option<Decimal>,
    pub daily_transaction_limit: Option<Decimal>,
    pub weekly_transaction_limit: Option<Decimal>,
    pub monthly_transaction_limit: Option<Decimal>,
    pub overdraft_interest_rate: Option<Decimal>,
    #[serde(serialize_with = "serialize_product_accrual_frequency", deserialize_with = "deserialize_product_accrual_frequency")]
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

impl Identifiable for ProductModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Auditable for ProductModel {
    fn get_audit_log_id(&self) -> Option<Uuid> {
        self.audit_log_id
    }
}

/// Index model for Product entity
/// Contains all product fields for application-layer caching and indexing
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ProductIdxModel {
    pub id: Uuid,
    pub name: Uuid,
    #[serde(serialize_with = "serialize_product_type", deserialize_with = "deserialize_product_type")]
    pub product_type: ProductType,
    pub minimum_balance: Decimal,
    pub maximum_balance: Option<Decimal>,
    pub overdraft_allowed: bool,
    pub overdraft_limit: Option<Decimal>,
    #[serde(serialize_with = "serialize_interest_calculation_method", deserialize_with = "deserialize_interest_calculation_method")]
    pub interest_calculation_method: InterestCalculationMethod,
    #[serde(serialize_with = "serialize_posting_frequency", deserialize_with = "deserialize_posting_frequency")]
    pub interest_posting_frequency: PostingFrequency,
    pub dormancy_threshold_days: i32,
    pub minimum_opening_balance: Decimal,
    pub closure_fee: Decimal,
    pub maintenance_fee: Option<Decimal>,
    #[serde(default, serialize_with = "serialize_maintenance_fee_frequency", deserialize_with = "deserialize_maintenance_fee_frequency")]
    pub maintenance_fee_frequency: MaintenanceFeeFrequency,
    pub default_dormancy_days: Option<i32>,
    pub default_overdraft_limit: Option<Decimal>,
    pub per_transaction_limit: Option<Decimal>,
    pub daily_transaction_limit: Option<Decimal>,
    pub weekly_transaction_limit: Option<Decimal>,
    pub monthly_transaction_limit: Option<Decimal>,
    pub overdraft_interest_rate: Option<Decimal>,
    #[serde(serialize_with = "serialize_product_accrual_frequency", deserialize_with = "deserialize_product_accrual_frequency")]
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

impl HasPrimaryKey for ProductIdxModel {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}

impl IndexAware for ProductModel {
    type IndexType = ProductIdxModel;
    
    fn to_index(&self) -> Self::IndexType {
        ProductIdxModel {
            id: self.id,
            name: self.name,
            product_type: self.product_type,
            minimum_balance: self.minimum_balance,
            maximum_balance: self.maximum_balance,
            overdraft_allowed: self.overdraft_allowed,
            overdraft_limit: self.overdraft_limit,
            interest_calculation_method: self.interest_calculation_method,
            interest_posting_frequency: self.interest_posting_frequency,
            dormancy_threshold_days: self.dormancy_threshold_days,
            minimum_opening_balance: self.minimum_opening_balance,
            closure_fee: self.closure_fee,
            maintenance_fee: self.maintenance_fee,
            maintenance_fee_frequency: self.maintenance_fee_frequency,
            default_dormancy_days: self.default_dormancy_days,
            default_overdraft_limit: self.default_overdraft_limit,
            per_transaction_limit: self.per_transaction_limit,
            daily_transaction_limit: self.daily_transaction_limit,
            weekly_transaction_limit: self.weekly_transaction_limit,
            monthly_transaction_limit: self.monthly_transaction_limit,
            overdraft_interest_rate: self.overdraft_interest_rate,
            accrual_frequency: self.accrual_frequency,
            interest_rate_tier_1: self.interest_rate_tier_1,
            interest_rate_tier_2: self.interest_rate_tier_2,
            interest_rate_tier_3: self.interest_rate_tier_3,
            interest_rate_tier_4: self.interest_rate_tier_4,
            interest_rate_tier_5: self.interest_rate_tier_5,
            account_gl_mapping: self.account_gl_mapping,
            fee_type_gl_mapping: self.fee_type_gl_mapping,
            is_active: self.is_active,
            valid_from: self.valid_from,
            valid_to: self.valid_to,
        }
    }
}

impl Identifiable for ProductIdxModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Index for ProductIdxModel {}

impl Indexable for ProductIdxModel {
    fn i64_keys(&self) -> HashMap<String, Option<i64>> {
        HashMap::new()
    }

    fn uuid_keys(&self) -> HashMap<String, Option<Uuid>> {
        HashMap::new()
    }
}

pub type ProductIdxModelCache = IdxModelCache<ProductIdxModel>;

/// The type of banking product.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "product_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum ProductType {
    CASA,
    LOAN,
}

/// Frequency for interest posting
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "posting_frequency", rename_all = "PascalCase")]
pub enum PostingFrequency {
    Daily,
    Weekly,
    Monthly,
    Quarterly,
    Annually,
}

/// Interest calculation method for CASA products (conventional and Islamic banking)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "interest_calculation_method", rename_all = "PascalCase")]
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

/// Frequency for maintenance fee charges
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type, Default)]
#[sqlx(type_name = "maintenance_fee_frequency", rename_all = "PascalCase")]
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

    /// No fee will be charged
    #[default]
    None,
}

/// Frequency for interest accrual
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, sqlx::Type)]
#[sqlx(type_name = "product_accrual_frequency", rename_all = "PascalCase")]
pub enum ProductAccrualFrequency {
    Daily,
    BusinessDaysOnly,
    None,
}

pub fn serialize_product_type<S>(product_type: &ProductType, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(match product_type {
        ProductType::CASA => "CASA",
        ProductType::LOAN => "LOAN",
    })
}

pub fn deserialize_product_type<'de, D>(deserializer: D) -> Result<ProductType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "CASA" => Ok(ProductType::CASA),
        "LOAN" => Ok(ProductType::LOAN),
        _ => Err(serde::de::Error::custom(format!("Unknown product type: {s}"))),
    }
}

pub fn serialize_posting_frequency<S>(posting_frequency: &PostingFrequency, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(match posting_frequency {
        PostingFrequency::Daily => "Daily",
        PostingFrequency::Weekly => "Weekly",
        PostingFrequency::Monthly => "Monthly",
        PostingFrequency::Quarterly => "Quarterly",
        PostingFrequency::Annually => "Annually",
    })
}

pub fn deserialize_posting_frequency<'de, D>(deserializer: D) -> Result<PostingFrequency, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "Daily" => Ok(PostingFrequency::Daily),
        "Weekly" => Ok(PostingFrequency::Weekly),
        "Monthly" => Ok(PostingFrequency::Monthly),
        "Quarterly" => Ok(PostingFrequency::Quarterly),
        "Annually" => Ok(PostingFrequency::Annually),
        _ => Err(serde::de::Error::custom(format!("Unknown posting frequency: {s}"))),
    }
}

pub fn serialize_interest_calculation_method<S>(method: &InterestCalculationMethod, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(match method {
        InterestCalculationMethod::DailyBalance => "DailyBalance",
        InterestCalculationMethod::AverageDailyBalance => "AverageDailyBalance",
        InterestCalculationMethod::MinimumBalance => "MinimumBalance",
        InterestCalculationMethod::Simple => "Simple",
        InterestCalculationMethod::Compound => "Compound",
        InterestCalculationMethod::Mudarabah => "Mudarabah",
        InterestCalculationMethod::Musharakah => "Musharakah",
        InterestCalculationMethod::Wakalah => "Wakalah",
        InterestCalculationMethod::QardHasan => "QardHasan",
    })
}

pub fn deserialize_interest_calculation_method<'de, D>(deserializer: D) -> Result<InterestCalculationMethod, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "DailyBalance" => Ok(InterestCalculationMethod::DailyBalance),
        "AverageDailyBalance" => Ok(InterestCalculationMethod::AverageDailyBalance),
        "MinimumBalance" => Ok(InterestCalculationMethod::MinimumBalance),
        "Simple" => Ok(InterestCalculationMethod::Simple),
        "Compound" => Ok(InterestCalculationMethod::Compound),
        "Mudarabah" => Ok(InterestCalculationMethod::Mudarabah),
        "Musharakah" => Ok(InterestCalculationMethod::Musharakah),
        "Wakalah" => Ok(InterestCalculationMethod::Wakalah),
        "QardHasan" => Ok(InterestCalculationMethod::QardHasan),
        _ => Err(serde::de::Error::custom(format!("Unknown interest calculation method: {s}"))),
    }
}

pub fn serialize_maintenance_fee_frequency<S>(frequency: &MaintenanceFeeFrequency, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(match frequency {
        MaintenanceFeeFrequency::Daily => "Daily",
        MaintenanceFeeFrequency::Weekly => "Weekly",
        MaintenanceFeeFrequency::BiWeekly => "BiWeekly",
        MaintenanceFeeFrequency::Monthly => "Monthly",
        MaintenanceFeeFrequency::BiMonthly => "BiMonthly",
        MaintenanceFeeFrequency::Quarterly => "Quarterly",
        MaintenanceFeeFrequency::SemiAnnually => "SemiAnnually",
        MaintenanceFeeFrequency::Annually => "Annually",
        MaintenanceFeeFrequency::OneTime => "OneTime",
        MaintenanceFeeFrequency::None => "None",
    })
}

pub fn deserialize_maintenance_fee_frequency<'de, D>(deserializer: D) -> Result<MaintenanceFeeFrequency, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "Daily" => Ok(MaintenanceFeeFrequency::Daily),
        "Weekly" => Ok(MaintenanceFeeFrequency::Weekly),
        "BiWeekly" => Ok(MaintenanceFeeFrequency::BiWeekly),
        "Monthly" => Ok(MaintenanceFeeFrequency::Monthly),
        "BiMonthly" => Ok(MaintenanceFeeFrequency::BiMonthly),
        "Quarterly" => Ok(MaintenanceFeeFrequency::Quarterly),
        "SemiAnnually" => Ok(MaintenanceFeeFrequency::SemiAnnually),
        "Annually" => Ok(MaintenanceFeeFrequency::Annually),
        "OneTime" => Ok(MaintenanceFeeFrequency::OneTime),
        "None" => Ok(MaintenanceFeeFrequency::None),
        _ => Ok(MaintenanceFeeFrequency::None),
    }
}

pub fn serialize_product_accrual_frequency<S>(frequency: &ProductAccrualFrequency, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(match frequency {
        ProductAccrualFrequency::Daily => "Daily",
        ProductAccrualFrequency::BusinessDaysOnly => "BusinessDaysOnly",
        ProductAccrualFrequency::None => "None",
    })
}

pub fn deserialize_product_accrual_frequency<'de, D>(deserializer: D) -> Result<ProductAccrualFrequency, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "Daily" => Ok(ProductAccrualFrequency::Daily),
        "BusinessDaysOnly" => Ok(ProductAccrualFrequency::BusinessDaysOnly),
        "None" => Ok(ProductAccrualFrequency::None),
        _ => Err(serde::de::Error::custom(format!("Unknown product accrual frequency: {s}"))),
    }
}