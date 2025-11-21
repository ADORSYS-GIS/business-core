use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "named_entity_type", rename_all = "PascalCase")]
pub enum NamedEntityType {
    Location,
    Person,
    EntityReference,
    ReasonReference,
    ActivityLog,
    Portfolio,
    ComplianceStatus,
    Document,
    Country,
    CountrySubdivision,
    Locality,
    ComplianceMetadata,
    Reason,
    RiskSummary,
    BusinessDay,
    WeekendDays,
    DateCalculationRules,
}

impl From<NamedEntityType> for &str {
    fn from(val: NamedEntityType) -> Self {
        match val {
            NamedEntityType::Location => "Location",
            NamedEntityType::Person => "Person",
            NamedEntityType::EntityReference => "EntityReference",
            NamedEntityType::ReasonReference => "ReasonReference",
            NamedEntityType::ActivityLog => "ActivityLog",
            NamedEntityType::Portfolio => "Portfolio",
            NamedEntityType::ComplianceStatus => "ComplianceStatus",
            NamedEntityType::Document => "Document",
            NamedEntityType::Country => "Country",
            NamedEntityType::CountrySubdivision => "CountrySubdivision",
            NamedEntityType::Locality => "Locality",
            NamedEntityType::ComplianceMetadata => "ComplianceMetadata",
            NamedEntityType::Reason => "Reason",
            NamedEntityType::RiskSummary => "RiskSummary",
            NamedEntityType::BusinessDay => "BusinessDay",
            NamedEntityType::WeekendDays => "WeekendDays",
            NamedEntityType::DateCalculationRules => "DateCalculationRules",
        }
    }
}

pub fn serialize_entity_type<S>(entity_type: &NamedEntityType, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(match entity_type {
        NamedEntityType::Location => "Location",
        NamedEntityType::Person => "Person",
        NamedEntityType::EntityReference => "EntityReference",
        NamedEntityType::ReasonReference => "ReasonReference",
        NamedEntityType::ActivityLog => "ActivityLog",
        NamedEntityType::Portfolio => "Portfolio",
        NamedEntityType::ComplianceStatus => "ComplianceStatus",
        NamedEntityType::Document => "Document",
        NamedEntityType::Country => "Country",
        NamedEntityType::CountrySubdivision => "CountrySubdivision",
        NamedEntityType::Locality => "Locality",
        NamedEntityType::ComplianceMetadata => "ComplianceMetadata",
        NamedEntityType::Reason => "Reason",
        NamedEntityType::RiskSummary => "RiskSummary",
        NamedEntityType::BusinessDay => "BusinessDay",
        NamedEntityType::WeekendDays => "WeekendDays",
        NamedEntityType::DateCalculationRules => "DateCalculationRules",
    })
}

pub fn deserialize_entity_type<'de, D>(deserializer: D) -> Result<NamedEntityType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    match s.as_str() {
        "Location" => Ok(NamedEntityType::Location),
        "Person" => Ok(NamedEntityType::Person),
        "EntityReference" => Ok(NamedEntityType::EntityReference),
        "ReasonReference" => Ok(NamedEntityType::ReasonReference),
        "ActivityLog" => Ok(NamedEntityType::ActivityLog),
        "Portfolio" => Ok(NamedEntityType::Portfolio),
        "ComplianceStatus" => Ok(NamedEntityType::ComplianceStatus),
        "Document" => Ok(NamedEntityType::Document),
        "Country" => Ok(NamedEntityType::Country),
        "CountrySubdivision" => Ok(NamedEntityType::CountrySubdivision),
        "Locality" => Ok(NamedEntityType::Locality),
        "ComplianceMetadata" => Ok(NamedEntityType::ComplianceMetadata),
        "Reason" => Ok(NamedEntityType::Reason),
        "RiskSummary" => Ok(NamedEntityType::RiskSummary),
        "BusinessDay" => Ok(NamedEntityType::BusinessDay),
        "WeekendDays" => Ok(NamedEntityType::WeekendDays),
        "DateCalculationRules" => Ok(NamedEntityType::DateCalculationRules),
        _ => Err(serde::de::Error::custom(format!("Unknown entity type: {s}"))),
    }
}