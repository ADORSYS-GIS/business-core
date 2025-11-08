use business_core_db::models::person::country::{CountryIdxModel, CountryModel};
use crate::utils::{get_heapless_string, get_optional_heapless_string, TryFromRow};
use postgres_unit_of_work::Executor;
use std::sync::Arc;
use sqlx::{postgres::PgRow, Row};
use std::error::Error;

pub struct CountryRepositoryImpl {
    pub executor: Executor,
    pub country_idx_cache: Arc<parking_lot::RwLock<business_core_db::IdxModelCache<CountryIdxModel>>>,
}

impl CountryRepositoryImpl {
    pub fn new(
        executor: Executor,
        country_idx_cache: Arc<parking_lot::RwLock<business_core_db::IdxModelCache<CountryIdxModel>>>,
    ) -> Self {
        Self {
            executor,
            country_idx_cache,
        }
    }

    pub async fn load_all_country_idx(
        executor: &Executor,
    ) -> Result<Vec<CountryIdxModel>, sqlx::Error> {
        let query = sqlx::query("SELECT * FROM country_idx");
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
            idx_models.push(CountryIdxModel::try_from_row(&row).map_err(|e| sqlx::Error::Decode(e.into()))?);
        }
        Ok(idx_models)
    }
}

impl TryFromRow<PgRow> for CountryModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(CountryModel {
            id: row.get("id"),
            iso2: get_heapless_string(row, "iso2")?,
            name_l1: get_heapless_string(row, "name_l1")?,
            name_l2: get_optional_heapless_string(row, "name_l2")?,
            name_l3: get_optional_heapless_string(row, "name_l3")?,
        })
    }
}

impl TryFromRow<PgRow> for CountryIdxModel {
    fn try_from_row(row: &PgRow) -> Result<Self, Box<dyn Error + Send + Sync>> {
        Ok(CountryIdxModel {
            id: row.get("country_id"),
            iso2_hash: row.try_get("iso2_hash")?,
        })
    }
}
