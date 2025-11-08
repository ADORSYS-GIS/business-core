pub mod auditable;
pub mod identifiable;
pub mod index;
pub mod indexable;
pub mod audit;

// Models modules will be added here as needed
// For example:
// pub mod person;

// Re-exports
pub use auditable::*;
pub use identifiable::*;
pub use index::*;
pub use indexable::*;
pub use audit::*;
// pub use person::*;