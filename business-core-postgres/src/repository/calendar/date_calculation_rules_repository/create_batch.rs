use business_core_db::models::calendar::date_calculation_rules::DateCalculationRulesModel;
use business_core_db::models::index_aware::IndexAware;
use business_core_db::repository::create_batch::CreateBatch;
use super::repo_impl::DateCalculationRulesRepositoryImpl;
use async_trait::async_trait;
use std::error::Error;
use uuid::Uuid;

#[async_trait]
impl CreateBatch<sqlx::Postgres, DateCalculationRulesModel> for DateCalculationRulesRepositoryImpl {
    async fn create_batch(
        &self,
        items: Vec<DateCalculationRulesModel>,
        _audit_info: Option<Uuid>,
    ) -> Result<Vec<DateCalculationRulesModel>, Box<dyn Error + Send + Sync>> {
        Self::create_batch_impl(self, items).await
    }
}

impl DateCalculationRulesRepositoryImpl {
    pub(super) async fn create_batch_impl(
        repo: &DateCalculationRulesRepositoryImpl,
        items: Vec<DateCalculationRulesModel>,
    ) -> Result<Vec<DateCalculationRulesModel>, Box<dyn Error + Send + Sync>> {
        if items.is_empty() {
            return Ok(Vec::new());
        }

        let mut saved_items = Vec::new();
        let mut indices = Vec::new();
        
        // Acquire lock once and do all database operations
        let mut tx = repo.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        
        for item in items {
            // Execute main insert
            sqlx::query(
                r#"
                INSERT INTO calendar_date_calculation_rules (
                    id, country_id, country_subdivision_id, rule_name, rule_purpose,
                    default_shift_rule, weekend_days_id, priority, is_active,
                    effective_date, expiry_date
                )
                VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10, $11)
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
            
            // Insert into index table
            let idx = item.to_index();
            sqlx::query(
                r#"
                INSERT INTO calendar_date_calculation_rules_idx (id, country_id, country_subdivision_id, rule_name_hash)
                VALUES ($1, $2, $3, $4)
                "#,
            )
            .bind(idx.id)
            .bind(idx.country_id)
            .bind(idx.country_subdivision_id)
            .bind(idx.rule_name_hash)
            .execute(&mut **transaction)
            .await?;
            
            indices.push(idx);
            saved_items.push(item);
        }
        
        // Release transaction lock before updating caches
        drop(tx);
        
        // Update BOTH caches after releasing transaction lock
        {
            let idx_cache = repo.date_calculation_rules_idx_cache.read().await;
            let main_cache = repo.date_calculation_rules_cache.read().await;
            
            for (idx, item) in indices.iter().zip(saved_items.iter()) {
                idx_cache.add(idx.clone());
                main_cache.insert(item.clone());
            }
        }

        Ok(saved_items)
    }
}