use crate::models::auditable::Auditable;
use crate::models::identifiable::Identifiable;
use crate::models::{Index, IndexAware};
use crate::{HasPrimaryKey, IdxModelCache, Indexable};
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
use std::collections::HashMap;
use uuid::Uuid;

/// Represents a tier for interest rate calculation based on balance.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InterestRateTierModel {
    pub id: Uuid,
    pub name: Uuid,
    pub minimum_balance: Decimal,
    pub maximum_balance: Option<Decimal>,
    pub interest_rate: Decimal,

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

impl Identifiable for InterestRateTierModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Auditable for InterestRateTierModel {
    fn get_audit_log_id(&self) -> Option<Uuid> {
        self.audit_log_id
    }
}

impl IndexAware for InterestRateTierModel {
    type IndexType = InterestRateTierIdxModel;

    fn to_index(&self) -> Self::IndexType {
        InterestRateTierIdxModel {
            id: self.id,
            name: self.name,
            minimum_balance: self.minimum_balance,
            maximum_balance: self.maximum_balance,
            interest_rate: self.interest_rate,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct InterestRateTierIdxModel {
    pub id: Uuid,
    // - Not a secondary Index. Do not provide any finder!
    pub name: Uuid,
    // - Not a secondary Index. Do not provide any finder!
    pub minimum_balance: Decimal,
    // - Not a secondary Index. Do not provide any finder!
    pub maximum_balance: Option<Decimal>,
    // - Not a secondary Index. Do not provide any finder!
    pub interest_rate: Decimal,
}

impl HasPrimaryKey for InterestRateTierIdxModel {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}

impl Identifiable for InterestRateTierIdxModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Index for InterestRateTierIdxModel {}

impl Indexable for InterestRateTierIdxModel {
    fn i64_keys(&self) -> HashMap<String, Option<i64>> {
        HashMap::new()
    }

    fn uuid_keys(&self) -> HashMap<String, Option<Uuid>> {
        HashMap::new()
    }
}

pub type InterestRateTierIdxModelCache = IdxModelCache<InterestRateTierIdxModel>;