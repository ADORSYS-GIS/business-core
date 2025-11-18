use business_core_db::models::calendar::date_calculation_rules::DateCalculationRulesModel;
use business_core_db::repository::load_batch::LoadBatch;
use crate::utils::TryFromRow;
use super::repo_impl::DateCalculationRulesRepositoryImpl;
use async_trait::async_trait;
use std::error::Error;
use uuid::Uuid;
use sqlx::Postgres;

#[async_trait]
impl LoadBatch<sqlx::Postgres, DateCalculationRulesModel> for DateCalculationRulesRepositoryImpl {
    async fn load_batch(
        &self,
        ids: &[Uuid],
    ) -> Result<Vec<Option<DateCalculationRulesModel>>, Box<dyn Error + Send + Sync>> {
        Self::load_batch_impl(self, ids).await
    }
}

impl DateCalculationRulesRepositoryImpl {
    pub(super) async fn load_batch_impl(
        repo: &DateCalculationRulesRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<Option<DateCalculationRulesModel>>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        
        // Try to get from cache first
        let main_cache = repo.date_calculation_rules_cache.read().await;
        let mut result = Vec::with_capacity(ids.len());
        let mut missing_ids = Vec::new();
        
        for &id in ids {
            match main_cache.get(&id) {
                Some(item) => result.push(Some(item)),
                None => {
                    result.push(None);
                    missing_ids.push(id);
                }
            }
        }
        
        drop(main_cache); // Release read lock
        
        // If all found in cache, return early
        if missing_ids.is_empty() {
            return Ok(result);
        }
        
        // Load missing items from database
        let query = r#"SELECT * FROM calendar_date_calculation_rules WHERE id = ANY($1)"#;
        let rows = {
            let mut tx = repo.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
            sqlx::query(query).bind(&missing_ids).fetch_all(&mut **transaction).await?
        };
        
        let mut loaded_map = std::collections::HashMap::new();
        for row in rows {
            let item = DateCalculationRulesModel::try_from_row(&row)?;
            loaded_map.insert(item.id, item);
        }
        
        // Update result and add to cache
        let main_cache = repo.date_calculation_rules_cache.read().await;
        for (i, &id) in ids.iter().enumerate() {
            if result[i].is_none() {
                if let Some(item) = loaded_map.remove(&id) {
                    main_cache.insert(item.clone());
                    result[i] = Some(item);
                }
            }
        }
        
        Ok(result)
    }
}