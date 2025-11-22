
use serde::{Deserialize, Serialize};
use uuid::Uuid;
use heapless::String as HeaplessString;
use sqlx::FromRow;

pub fn deserialize_customer_account_code<'de, D>(
    deserializer: D,
) -> Result<heapless::String<50>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    std::str::FromStr::from_str(&s).map_err(|_| {
        serde::de::Error::custom("Value for customer_account_code is too long (max 50 chars)")
    })
}

pub fn deserialize_overdraft_code<'de, D>(
    deserializer: D,
) -> Result<Option<heapless::String<50>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s: Option<String> = Option::deserialize(deserializer)?;
    s.map(|s| {
        std::str::FromStr::from_str(&s).map_err(|_| {
            serde::de::Error::custom("Value for overdraft_code is too long (max 50 chars)")
        })
    })
    .transpose()
}

/// Core GL mapping for a product
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccountGlMappingModel {
    pub id: Uuid,
    pub customer_account_code: HeaplessString<50>,
    pub overdraft_code: Option<HeaplessString<50>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccountGlMappingIdxModel {
    pub id: Uuid,
    #[serde(deserialize_with = "deserialize_customer_account_code")]
    pub customer_account_code: HeaplessString<50>,
    #[serde(deserialize_with = "deserialize_overdraft_code")]
    pub overdraft_code: Option<HeaplessString<50>>,
}
