use business_core_db::models::person::country_subdivision::{CountrySubdivisionIdxModel, CountrySubdivisionModel};
use crate::utils::{get_heapless_string, get_optional_heapless_string, TryFromRow};
use postgres_unit_of_work::Executor;
use std::sync::Arc;
use sqlx::{postgres::PgRow, Row};
use std::error::Error;

pub struct CountrySubdivisionRepositoryImpl {
    pub executor: Executor,
    pub country_subdivision_idx_cache: Arc<parking_lot::RwLock<business_core_db::IdxModelCache<CountrySubdivisionIdxModel>>>,
}

impl CountrySubdivisionRepositoryImpl {
    pub fn new(
        executor: Executor,
        country_subdivision_idx_cache: Arc<parking_lot::RwLock<business_core_db::IdxModelCache<CountrySubdivisionIdxModel>>>,
    ) -> Self {
        Self {
            executor,
            country_subdivision_idx_cache,
        }
    }

    pub async fn load_all_country_subdivision_idx(
        executor: &Executor,
    ) -> Result<Vec<CountrySubdivisionIdxModel>, sqlx::Error> {
        let query = sqlx::query("SELECT * FROM country_subdivision_idx");
        let rows = {
            let mut tx = executor.tx.lock().await;
            if let Some(transaction) = tx.as_mut() {
                query.fetch_all(&mut **transaction).await?
            } else {
                return Err(sqlx::Error::PoolTimedOut);
            }
        };
        
        let mut idx_models = Vec::with_capacity(rows.len());
        for row in rows {
            idx_models.push(CountrySubdivisionIdxModel::try_from_row(&row).map_err(sqlx::Error::Decode)?);
        }
        Ok(idx_models)
    }
}

impl TryFromRow<PgRow> for CountrySubdivisionModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(CountrySubdivisionModel {
            id: row.get("id"),
            country_id: row.get("country_id"),
            code: get_heapless_string(row, "code")?,
            name_l1: get_heapless_string(row, "name_l1")?,
            name_l2: get_optional_heapless_string(row, "name_l2")?,
            name_l3: get_optional_heapless_string(row, "name_l3")?,
        })
    }
}

impl TryFromRow<PgRow> for CountrySubdivisionIdxModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(CountrySubdivisionIdxModel {
            id: row.get("id"),
            country_id: row.get("country_id"),
            code_hash: row.try_get("code_hash")?,
        })
    }
}