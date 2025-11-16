pub mod auditable;
pub mod identifiable;
pub mod index;
pub mod index_aware;
pub mod audit;
pub mod person;
pub mod reason_and_purpose;

// Models modules will be added here as needed
// For example:
// pub mod person;

// Re-exports
pub use auditable::*;
pub use identifiable::*;
pub use index::*;
pub use index_aware::*;
pub use audit::*;
// pub use person::*;