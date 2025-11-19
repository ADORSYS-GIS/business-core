#[cfg(test)]
#[allow(clippy::module_inception)]
pub mod test_utils {
    pub use crate::repository::person::test_utils::{
        create_test_country, create_test_country_subdivision, create_test_locality,
    };
}