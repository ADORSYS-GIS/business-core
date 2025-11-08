use heapless::String as HeaplessString;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use std::collections::HashMap;
use crate::{HasPrimaryKey, IdxModelCache, Indexable};
use crate::models::{IndexAware, Identifiable, Index};

/// # Documentation
/// - Country structure with ISO 3166-1 alpha-2 code
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CountryModel {
    pub id: Uuid,
    
    pub iso2: HeaplessString<2>,

    pub name_l1: HeaplessString<100>,
    pub name_l2: Option<HeaplessString<100>>,
    pub name_l3: Option<HeaplessString<100>>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CountryIdxModel {
    pub id: Uuid,

    pub iso2_hash: i64,
}

impl HasPrimaryKey for CountryIdxModel {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}

impl Identifiable for CountryModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl IndexAware for CountryModel {
    type IndexType = CountryIdxModel;
    
    fn to_index(&self) -> Self::IndexType {
        // Calculate hash for iso2 field
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};
        
        let mut hasher = DefaultHasher::new();
        self.iso2.as_str().hash(&mut hasher);
        let iso2_hash = hasher.finish() as i64;
        
        CountryIdxModel {
            id: self.id,
            iso2_hash,
        }
    }
}

impl Identifiable for CountryIdxModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Index for CountryIdxModel {}

impl Indexable for CountryIdxModel {
    fn i64_keys(&self) -> HashMap<String, Option<i64>> {
        let mut keys = HashMap::new();
        keys.insert("iso2_hash".to_string(), Some(self.iso2_hash));
        keys
    }

    fn uuid_keys(&self) -> HashMap<String, Option<Uuid>> {
        HashMap::new()
    }
}

pub type CountryIdxModelCache = IdxModelCache<CountryIdxModel>;