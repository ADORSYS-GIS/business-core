#[cfg(test)]
pub mod test_utils {
    use business_core_db::models::person::country::CountryModel;
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
}