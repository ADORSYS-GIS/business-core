use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "audit_entity_type", rename_all = "PascalCase")]
pub enum AuditEntityType {
    Location,
    Person,
    EntityReference,
    ReasonReference,
    ActivityLog,
    Portfolio,
    ComplianceStatus,
    Document,
    Named,
    AccountGlMapping,
    FeeTypeGlMapping,
    InterestRateTier,
}

impl From<AuditEntityType> for &str {
    fn from(val: AuditEntityType) -> Self {
        match val {
            AuditEntityType::Location => "Location",
            AuditEntityType::Person => "Person",
            AuditEntityType::EntityReference => "EntityReference",
            AuditEntityType::ReasonReference => "ReasonReference",
            AuditEntityType::ActivityLog => "ActivityLog",
            AuditEntityType::Portfolio => "Portfolio",
            AuditEntityType::ComplianceStatus => "ComplianceStatus",
            AuditEntityType::Document => "Document",
            AuditEntityType::Named => "Named",
            AuditEntityType::AccountGlMapping => "AccountGlMapping",
            AuditEntityType::FeeTypeGlMapping => "FeeTypeGlMapping",
            AuditEntityType::InterestRateTier => "InterestRateTier",
        }
    }
}

pub fn serialize_entity_type<S>(entity_type: &AuditEntityType, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(match entity_type {
        AuditEntityType::Location => "Location",
        AuditEntityType::Person => "Person",
        AuditEntityType::EntityReference => "EntityReference",
        AuditEntityType::ReasonReference => "ReasonReference",
        AuditEntityType::ActivityLog => "ActivityLog",
        AuditEntityType::Portfolio => "Portfolio",
        AuditEntityType::ComplianceStatus => "ComplianceStatus",
        AuditEntityType::Document => "Document",
        AuditEntityType::Named => "Named",
        AuditEntityType::AccountGlMapping => "AccountGlMapping",
        AuditEntityType::FeeTypeGlMapping => "FeeTypeGlMapping",
        AuditEntityType::InterestRateTier => "InterestRateTier",
    })
}

pub fn deserialize_entity_type<'de, D>(deserializer: D) -> Result<AuditEntityType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "Location" => Ok(AuditEntityType::Location),
        "Person" => Ok(AuditEntityType::Person),
        "EntityReference" => Ok(AuditEntityType::EntityReference),
        "ReasonReference" => Ok(AuditEntityType::ReasonReference),
        "ActivityLog" => Ok(AuditEntityType::ActivityLog),
        "Portfolio" => Ok(AuditEntityType::Portfolio),
        "ComplianceStatus" => Ok(AuditEntityType::ComplianceStatus),
        "Document" => Ok(AuditEntityType::Document),
        "Named" => Ok(AuditEntityType::Named),
        "AccountGlMapping" => Ok(AuditEntityType::AccountGlMapping),
        "FeeTypeGlMapping" => Ok(AuditEntityType::FeeTypeGlMapping),
        "InterestRateTier" => Ok(AuditEntityType::InterestRateTier),
        _ => Err(serde::de::Error::custom(format!("Unknown entity type: {s}"))),
    }
}