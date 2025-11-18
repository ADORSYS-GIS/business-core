use chrono::{DateTime, Utc, NaiveDate};
use uuid::Uuid;
use heapless::String as HeaplessString;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[cfg_attr(feature = "sqlx", derive(sqlx::FromRow))]
pub struct DateCalculationRulesModel {
    pub id: Uuid,
    pub country_id: Uuid,
    pub country_subdivision_id: Option<Uuid>,
    pub rule_name: HeaplessString<50>,
    pub rule_purpose: DateRulePurpose,
    pub default_shift_rule: DateShiftRule,
    pub weekend_days_id: Option<Uuid>,
    pub priority: i32, // Rule precedence order
    pub is_active: bool,
    pub effective_date: NaiveDate,
    pub expiry_date: Option<NaiveDate>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "date_shift_rule", rename_all = "PascalCase")]
pub enum DateRulePurpose {
    DateShift,
    MaturityCalculation,
    PaymentDue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "date_shift_rule", rename_all = "PascalCase")]
pub enum DateShiftRule {
    NextBusinessDay,
    PreviousBusinessDay,
    NoShift,
}

impl FromStr for DateShiftRule {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "NextBusinessDay" => Ok(DateShiftRule::NextBusinessDay),
            "PreviousBusinessDay" => Ok(DateShiftRule::PreviousBusinessDay),
            "NoShift" => Ok(DateShiftRule::NoShift),
            _ => Err(()),
        }
    }
}
