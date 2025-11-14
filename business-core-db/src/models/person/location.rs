use heapless::String as HeaplessString;
use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use std::collections::HashMap;

/// Database model for location type enum
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, sqlx::Type)]
#[sqlx(type_name = "location_type", rename_all = "PascalCase")]
pub enum LocationType {
    Residential,
    Business,
    Mailing,
    Temporary,
    Branch,
    Community,
    Other,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LocationModel {
    pub id: Uuid,

    /// - Structured location components - 4 street lines
    pub street_line1: HeaplessString<50>,
    pub street_line2: Option<HeaplessString<50>>,
    pub street_line3: Option<HeaplessString<50>>,
    pub street_line4: Option<HeaplessString<50>>,

    pub locality_id: Uuid,
    pub postal_code: Option<HeaplessString<20>>,

    /// Geographical coordinates (decimal degrees)
    pub latitude: Option<Decimal>,
    pub longitude: Option<Decimal>,
    pub accuracy_meters: Option<f32>,
    
    #[serde(serialize_with = "serialize_location_type", deserialize_with = "deserialize_location_type")]
    pub location_type: LocationType,

    pub last_audit_log_id: Option<Uuid>,
}


#[derive(Debug, Clone, FromRow)]
pub struct LocationIdxModel {
    pub location_id: Uuid,
    pub locality_id: Uuid,
}

