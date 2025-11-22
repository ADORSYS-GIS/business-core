use heapless::String as HeaplessString;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use std::collections::HashMap;
use crate::{HasPrimaryKey, IdxModelCache, Indexable};
use crate::models::{IndexAware, Identifiable, Index};
use crate::utils::hash_as_i64;

/// # Documentation
/// - CountrySubdivision structure with subdivision code and multi-language names
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CountrySubdivisionModel {
    pub id: Uuid,
    
    pub country_id: Uuid,
    
    pub code: HeaplessString<10>,

    pub name: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CountrySubdivisionIdxModel {
    pub id: Uuid,

    pub country_id: Uuid,
    
    pub code_hash: i64,
}

impl HasPrimaryKey for CountrySubdivisionIdxModel {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}

impl Identifiable for CountrySubdivisionModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl IndexAware for CountrySubdivisionModel {
    type IndexType = CountrySubdivisionIdxModel;
    
    fn to_index(&self) -> Self::IndexType {
        // Calculate hash for code field using the centralized hash_as_i64 function
        let code_hash = hash_as_i64(&self.code.as_str()).unwrap();
        
        CountrySubdivisionIdxModel {
            id: self.id,
            country_id: self.country_id,
            code_hash,
        }
    }
}

impl Identifiable for CountrySubdivisionIdxModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Index for CountrySubdivisionIdxModel {}

impl Indexable for CountrySubdivisionIdxModel {
    fn i64_keys(&self) -> HashMap<String, Option<i64>> {
        let mut keys = HashMap::new();
        keys.insert("code_hash".to_string(), Some(self.code_hash));
        keys
    }

    fn uuid_keys(&self) -> HashMap<String, Option<Uuid>> {
        let mut keys = HashMap::new();
        keys.insert("country_id".to_string(), Some(self.country_id));
        keys
    }
}

pub type CountrySubdivisionIdxModelCache = IdxModelCache<CountrySubdivisionIdxModel>;