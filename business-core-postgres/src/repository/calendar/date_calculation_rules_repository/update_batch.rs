use business_core_db::models::calendar::date_calculation_rules::DateCalculationRulesModel;
use business_core_db::models::index_aware::IndexAware;
use business_core_db::repository::update_batch::UpdateBatch;
use super::repo_impl::DateCalculationRulesRepositoryImpl;
use async_trait::async_trait;
use std::error::Error;
use uuid::Uuid;

#[async_trait]
impl UpdateBatch<sqlx::Postgres, DateCalculationRulesModel> for DateCalculationRulesRepositoryImpl {
    async fn update_batch(
        &self,
        items: Vec<DateCalculationRulesModel>,
        _audit_info: Option<Uuid>,
    ) -> Result<Vec<DateCalculationRulesModel>, Box<dyn Error + Send + Sync>> {
        Self::update_batch_impl(self, items).await
    }
}

impl DateCalculationRulesRepositoryImpl {
    pub(super) async fn update_batch_impl(
        repo: &DateCalculationRulesRepositoryImpl,
        items: Vec<DateCalculationRulesModel>,
    ) -> Result<Vec<DateCalculationRulesModel>, Box<dyn Error + Send + Sync>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut updated_items = Vec::new();
        let mut indices = Vec::new();
        
        // Acquire lock once and do all database operations
        let mut tx = repo.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        
        for item in items {
            // Execute update on main table
            sqlx::query(
                r#"
                UPDATE calendar_date_calculation_rules
                SET country_id = $2, country_subdivision_id = $3, rule_name = $4, rule_purpose = $5,
                    default_shift_rule = $6, weekend_days_id = $7, priority = $8, is_active = $9,
                    effective_date = $10, expiry_date = $11
                WHERE id = $1
                "#,
            )
            .bind(item.id)
            .bind(item.country_id)
            .bind(item.country_subdivision_id)
            .bind(item.rule_name.as_str())
            .bind(item.rule_purpose)
            .bind(item.default_shift_rule)
            .bind(item.weekend_days_id)
            .bind(item.priority)
            .bind(item.is_active)
            .bind(item.effective_date)
            .bind(item.expiry_date)
            .execute(&mut **transaction)
            .await?;
            
            // Update index table
            let idx = item.to_index();
            sqlx::query(
                r#"
                UPDATE calendar_date_calculation_rules_idx
                SET country_id = $2, country_subdivision_id = $3, rule_name_hash = $4
                WHERE id = $1
                "#,
            )
            .bind(idx.id)
            .bind(idx.country_id)
            .bind(idx.country_subdivision_id)
            .bind(idx.rule_name_hash)
            .execute(&mut **transaction)
            .await?;
            
            indices.push((item.id, idx));
            updated_items.push(item);
        }
        
        drop(tx); // Release transaction lock
        
        // Update BOTH caches after releasing transaction lock
        {
            let idx_cache = repo.date_calculation_rules_idx_cache.read().await;
            let main_cache = repo.date_calculation_rules_cache.read().await;
            
            for (id, idx) in indices.iter() {
                idx_cache.remove(id);
                idx_cache.add(idx.clone());
                // Main cache update replaces the entire entity
                if let Some(item) = updated_items.iter().find(|i| i.id == *id) {
                    main_cache.update(item.clone());
                }
            }
        }

        Ok(updated_items)
    }
}