pub mod country_repository;
pub mod country_subdivision_repository;
pub mod locality_repository;
pub mod location_repository;
pub mod person_repository;
pub mod entity_reference_repository;
pub mod factory;

pub use country_repository::CountryRepositoryImpl;
pub use country_subdivision_repository::CountrySubdivisionRepositoryImpl;
pub use locality_repository::LocalityRepositoryImpl;
pub use location_repository::LocationRepositoryImpl;
pub use person_repository::PersonRepositoryImpl;
pub use entity_reference_repository::EntityReferenceRepositoryImpl;
pub use factory::{PersonRepoFactory, PersonRepositories};

#[cfg(test)]
pub mod test_utils;