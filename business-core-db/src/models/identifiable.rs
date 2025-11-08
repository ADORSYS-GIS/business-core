use uuid::Uuid;

/// Trait for entities that can be uniquely identified by a UUID
pub trait Identifiable {
    /// Returns the unique identifier of the entity
    fn get_id(&self) -> Uuid;
}