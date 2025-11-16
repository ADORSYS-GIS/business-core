use heapless::String as HeaplessString;
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use std::collections::HashMap;
use crate::{HasPrimaryKey, IdxModelCache, Indexable};
use crate::models::{IndexAware, Identifiable, Index};
use crate::utils::hash_as_i64;

/// # Documentation
/// Compliance-specific metadata for AML/CTF/KYC reasons
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ComplianceMetadataModel {
    pub id: Uuid,
    
    /// Regulatory reference code (e.g., "FATF-R.16", "BSA-3.14")
    pub regulatory_code: Option<HeaplessString<20>>,
    
    /// Whether this reason requires regulatory reporting
    pub reportable: bool,
    
    /// Whether this triggers a Suspicious Activity Report
    pub requires_sar: bool,
    
    /// Whether this triggers a Currency Transaction Report
    pub requires_ctr: bool,
    
    /// Minimum retention period in years for audit
    pub retention_years: i16,
    
    /// Whether management escalation is required
    pub escalation_required: bool,
    
    /// Risk score impact (0-100)
    pub risk_score_impact: Option<i16>,
    
    /// Whether customer notification is prohibited (tipping off)
    pub no_tipping_off: bool,
    
    /// Relevant jurisdiction codes
    pub jurisdictions1: HeaplessString<2>,
    pub jurisdictions2: HeaplessString<2>,
    pub jurisdictions3: HeaplessString<2>,
    pub jurisdictions4: HeaplessString<2>,
    pub jurisdictions5: HeaplessString<2>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct ComplianceMetadataIdxModel {
    pub id: Uuid,
    pub regulatory_code_hash: Option<i64>,
}

impl HasPrimaryKey for ComplianceMetadataIdxModel {
    fn primary_key(&self) -> Uuid {
        self.id
    }
}

impl Identifiable for ComplianceMetadataModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl IndexAware for ComplianceMetadataModel {
    type IndexType = ComplianceMetadataIdxModel;
    
    fn to_index(&self) -> Self::IndexType {
        let regulatory_code_hash = self.regulatory_code
            .as_ref()
            .and_then(|code| hash_as_i64(&code.as_str()).ok());
        
        ComplianceMetadataIdxModel {
            id: self.id,
            regulatory_code_hash,
        }
    }
}

impl Identifiable for ComplianceMetadataIdxModel {
    fn get_id(&self) -> Uuid {
        self.id
    }
}

impl Index for ComplianceMetadataIdxModel {}

impl Indexable for ComplianceMetadataIdxModel {
    fn i64_keys(&self) -> HashMap<String, Option<i64>> {
        let mut keys = HashMap::new();
        keys.insert("regulatory_code_hash".to_string(), self.regulatory_code_hash);
        keys
    }

    fn uuid_keys(&self) -> HashMap<String, Option<Uuid>> {
        HashMap::new()
    }
}

pub type ComplianceMetadataIdxModelCache = IdxModelCache<ComplianceMetadataIdxModel>;