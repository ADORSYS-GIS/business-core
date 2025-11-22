use crate::models::auditable::Auditable;
use crate::models::identifiable::Identifiable;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;
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

impl InterestRateTierModel {
    pub fn to_index(&self) -> InterestRateTierIdxModel {
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
    pub name: Uuid,
    pub minimum_balance: Decimal,
    pub maximum_balance: Option<Decimal>,
    pub interest_rate: Decimal,
}