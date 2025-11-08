use super::index::Index;

/// Trait for types that can be converted to an Index
pub trait Indexable {
    /// The associated Index type that this type can be converted to
    type IndexType: Index;
    
    /// Converts this type to an Index
    fn to_index(&self) -> Self::IndexType;
}