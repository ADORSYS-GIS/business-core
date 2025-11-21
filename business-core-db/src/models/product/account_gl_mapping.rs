use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// Core GL mapping for a product
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlMappingModel {
    pub id: Uuid,
    pub customer_account_code: heapless::String<50>,
    pub overdraft_code: Option<heapless::String<50>>,
}