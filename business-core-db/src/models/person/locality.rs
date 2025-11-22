use heapless::String as HeaplessString;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use std::collections::HashMap;
use crate::{HasPrimaryKey, IdxModelCache, Indexable};
use crate::models::{IndexAware, Identifiable, Index};
use crate::utils::hash_as_i64;

/// # Documentation
/// - Locality structure with locality code and multi-language names
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LocalityModel {
    pub id: Uuid,
    
    pub country_subdivision_id: Uuid,
    
    pub code: HeaplessString<50>,

    pub name: Uuid,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct LocalityIdxModel {
    pub id: Uuid,

    pub country_subdivision_id: Uuid,
    
    pub code_hash: i64,
}

impl HasPrimaryKey for LocalityIdxModel {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}

impl Identifiable for LocalityModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl IndexAware for LocalityModel {
    type IndexType = LocalityIdxModel;
    
    fn to_index(&self) -> Self::IndexType {
        // Calculate hash for code field using the centralized hash_as_i64 function
        let code_hash = hash_as_i64(&self.code.as_str()).unwrap();
        
        LocalityIdxModel {
            id: self.id,
            country_subdivision_id: self.country_subdivision_id,
            code_hash,
        }
    }
}

impl Identifiable for LocalityIdxModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Index for LocalityIdxModel {}

impl Indexable for LocalityIdxModel {
    fn i64_keys(&self) -> HashMap<String, Option<i64>> {
        let mut keys = HashMap::new();
        keys.insert("code_hash".to_string(), Some(self.code_hash));
        keys
    }

    fn uuid_keys(&self) -> HashMap<String, Option<Uuid>> {
        let mut keys = HashMap::new();
        keys.insert("country_subdivision_id".to_string(), Some(self.country_subdivision_id));
        keys
    }
}

pub type LocalityIdxModelCache = IdxModelCache<LocalityIdxModel>;