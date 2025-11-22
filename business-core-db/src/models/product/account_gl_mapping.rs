use serde::{Deserialize, Serialize};
use uuid::Uuid;
use heapless::String as HeaplessString;
use crate::models::auditable::Auditable;
use crate::models::identifiable::Identifiable;
use crate::models::{Index, IndexAware};
use crate::{HasPrimaryKey, Indexable};
use postgres_index_cache::HasPrimaryKey as HasPrimaryKeyCache;
use std::collections::HashMap;
use sqlx::FromRow;

/// Core GL mapping for a product
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccountGlMappingModel {
    pub id: Uuid,
    pub customer_account_code: HeaplessString<50>,
    pub overdraft_code: Option<HeaplessString<50>>,
    
    // Audit fields
    pub antecedent_hash: i64,
    pub antecedent_audit_log_id: Uuid,
    pub hash: i64,
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

impl HasPrimaryKeyCache for AccountGlMappingModel {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct AccountGlMappingIdxModel {
    pub id: Uuid,
    pub customer_account_code_hash: i64,
}

impl IndexAware for AccountGlMappingModel {
    type IndexType = AccountGlMappingIdxModel;

    fn to_index(&self) -> Self::IndexType {
        let customer_account_code_hash =
            crate::utils::hash_as_i64(&self.customer_account_code.as_str()).unwrap_or(0);

        AccountGlMappingIdxModel {
            id: self.id,
            customer_account_code_hash,
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
        let mut keys = HashMap::new();
        keys.insert(
            "customer_account_code_hash".to_string(),
            Some(self.customer_account_code_hash),
        );
        keys
    }

    fn uuid_keys(&self) -> HashMap<String, Option<Uuid>> {
        HashMap::new()
    }
}

impl HasPrimaryKey for AccountGlMappingIdxModel {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}