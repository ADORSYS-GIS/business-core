pub mod repo_impl;
pub mod create_batch;
pub mod delete_batch;
pub mod exist_by_ids;
pub mod load_batch;
pub mod update_batch;
pub mod find_ids_by_code_hash;
pub mod find_ids_by_country_subdivision_id;

pub use repo_impl::LocalityRepositoryImpl;

#[cfg(test)]
pub mod test_utils {
    use business_core_db::models::person::country::CountryModel;
    use business_core_db::models::person::country_subdivision::CountrySubdivisionModel;
    use business_core_db::models::person::locality::LocalityModel;
    use heapless::String as HeaplessString;
    use uuid::Uuid;

    pub fn create_test_country(iso2: &str, name: &str) -> CountryModel {
        CountryModel {
            id: Uuid::new_v4(),
            iso2: HeaplessString::try_from(iso2).unwrap(),
            name_l1: HeaplessString::try_from(name).unwrap(),
            name_l2: None,
            name_l3: None,
        }
    }

    pub fn create_test_country_subdivision(
        country_id: Uuid,
        code: &str,
        name: &str,
    ) -> CountrySubdivisionModel {
        CountrySubdivisionModel {
            id: Uuid::new_v4(),
            country_id,
            code: HeaplessString::try_from(code).unwrap(),
            name_l1: HeaplessString::try_from(name).unwrap(),
            name_l2: None,
            name_l3: None,
        }
    }

    pub fn create_test_locality(
        country_subdivision_id: Uuid,
        code: &str,
        name: &str,
    ) -> LocalityModel {
        LocalityModel {
            id: Uuid::new_v4(),
            country_subdivision_id,
            code: HeaplessString::try_from(code).unwrap(),
            name_l1: HeaplessString::try_from(name).unwrap(),
            name_l2: None,
            name_l3: None,
        }
    }
}