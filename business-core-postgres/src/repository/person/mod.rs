pub mod country_repository;
pub mod country_subdivision_repository;
pub mod locality_repository;
pub mod location_repository;
pub mod person_repository;
pub mod entity_reference_repository;
pub mod risk_summary_repository;
pub mod activity_log_repository;
pub mod portfolio_repository;
pub mod compliance_status_repository;
pub mod document_repository;
pub mod factory;

pub use country_repository::CountryRepositoryImpl;
pub use country_subdivision_repository::CountrySubdivisionRepositoryImpl;
pub use locality_repository::LocalityRepositoryImpl;
pub use location_repository::LocationRepositoryImpl;
pub use person_repository::PersonRepositoryImpl;
pub use entity_reference_repository::EntityReferenceRepositoryImpl;
pub use risk_summary_repository::RiskSummaryRepositoryImpl;
pub use activity_log_repository::ActivityLogRepositoryImpl;
pub use portfolio_repository::PortfolioRepositoryImpl;
pub use compliance_status_repository::ComplianceStatusRepositoryImpl;
pub use document_repository::DocumentRepositoryImpl;
pub use factory::{PersonRepoFactory, PersonRepositories};

#[cfg(test)]
pub mod test_utils;