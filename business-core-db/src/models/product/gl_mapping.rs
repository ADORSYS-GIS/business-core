use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlMappingModel {
    pub product_id: Uuid,
    pub customer_account_code: heapless::String<50>,
    pub interest_expense_code: heapless::String<50>,
    pub fee_income_code: heapless::String<50>,
    pub overdraft_code: Option<heapless::String<50>>,
}