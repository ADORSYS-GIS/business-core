use chrono::NaiveDate;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use super::product_rules::ProductRules;

/// Represents a banking product in the database.
/// # Audit
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProductModel {
    pub id: Uuid,
    pub name_l1: heapless::String<100>,
    pub name_l2: heapless::String<100>,
    pub name_l3: heapless::String<100>,
    pub description: heapless::String<255>,
    pub is_active: bool,
    pub valid_from: NaiveDate,
    pub valid_to: Option<NaiveDate>,
    pub product_type: ProductType,
    pub rules: ProductRules,
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
