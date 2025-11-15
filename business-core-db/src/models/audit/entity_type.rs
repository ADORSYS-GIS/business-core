use serde::{Deserialize, Serialize};
use sqlx::Type;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Type)]
#[sqlx(type_name = "entity_type", rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EntityType {
    Location,
}

impl From<EntityType> for &str {
    fn from(val: EntityType) -> Self {
        match val {
            EntityType::Location => "LOCATION",
        }
    }
}