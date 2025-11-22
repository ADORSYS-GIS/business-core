use crate::models::auditable::Auditable;
use crate::models::identifiable::Identifiable;
use crate::{HasPrimaryKey, IdxModelCache, Indexable};
use crate::models::{Index, IndexAware};
use heapless::String as HeaplessString;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use std::collections::HashMap;

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

impl Identifiable for AccountGlMappingModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Auditable for AccountGlMappingModel {
    fn get_audit_log_id(&self) -> Option<Uuid> {
        self.audit_log_id
    }
}

/// As the AccountGlMappingModel is tiny, we can keep main model
/// info here and use the model to perform work.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccountGlMappingIdxModel {
    pub id: Uuid,
    #[serde(deserialize_with = "deserialize_customer_account_code")]
    // - Not a secondary Index. Do not provide any finder!
    pub customer_account_code: HeaplessString<50>,
    #[serde(deserialize_with = "deserialize_overdraft_code")]
    // - Not a secondary Index. Do not provide any finder!
    pub overdraft_code: Option<HeaplessString<50>>,
}

impl HasPrimaryKey for AccountGlMappingIdxModel {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}

impl IndexAware for AccountGlMappingModel {
    type IndexType = AccountGlMappingIdxModel;
    
    fn to_index(&self) -> Self::IndexType {
        AccountGlMappingIdxModel {
            id: self.id,
            customer_account_code: self.customer_account_code.clone(),
            overdraft_code: self.overdraft_code.clone(),
        }
    }
}

impl Identifiable for AccountGlMappingIdxModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Index for AccountGlMappingIdxModel {}

impl Indexable for AccountGlMappingIdxModel {
    fn i64_keys(&self) -> HashMap<String, Option<i64>> {
        HashMap::new()
    }

    fn uuid_keys(&self) -> HashMap<String, Option<Uuid>> {
        HashMap::new()
    }
}

pub type AccountGlMappingIdxModelCache = IdxModelCache<AccountGlMappingIdxModel>;