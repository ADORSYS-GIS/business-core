use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "entity_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntityType {
    Location,
    Person,
    EntityReference,
    ReasonReference,
    ActivityLog,
    Portfolio,
    ComplianceStatus,
    Document,
}

impl From<EntityType> for &str {
    fn from(val: EntityType) -> Self {
        match val {
            EntityType::Location => "LOCATION",
            EntityType::Person => "PERSON",
            EntityType::EntityReference => "ENTITY_REFERENCE",
            EntityType::ReasonReference => "REASON_REFERENCE",
            EntityType::ActivityLog => "ACTIVITY_LOG",
            EntityType::Portfolio => "PORTFOLIO",
            EntityType::ComplianceStatus => "COMPLIANCE_STATUS",
            EntityType::Document => "DOCUMENT",
        }
    }
}