use async_trait::async_trait;
use business_core_db::models::person::risk_summary::RiskSummaryModel;
use business_core_db::repository::load_batch::LoadBatch;
use crate::utils::TryFromRow;
use sqlx::Postgres;
use std::error::Error;
use uuid::Uuid;

use super::repo_impl::RiskSummaryRepositoryImpl;

impl RiskSummaryRepositoryImpl {
    pub(super) async fn load_batch_impl(
        repo: &RiskSummaryRepositoryImpl,
        ids: &[Uuid],
    ) -> Result<Vec<Option<RiskSummaryModel>>, Box<dyn Error + Send + Sync>> {
        if ids.is_empty() {
            return Ok(Vec::new());
        }
        
        let query = r#"SELECT * FROM risk_summary WHERE id = ANY($1)"#;
        let rows = {
            let mut tx = repo.executor.tx.lock().await;
            let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
            sqlx::query(query).bind(ids).fetch_all(&mut **transaction).await?
        };
        
        let mut item_map = std::collections::HashMap::new();
        for row in rows {
            let item = RiskSummaryModel::try_from_row(&row)?;
            item_map.insert(item.id, item);
        }
        
        let mut result = Vec::with_capacity(ids.len());
        for id in ids {
            result.push(item_map.remove(id));
        }
        Ok(result)
    }
}

#[async_trait]
impl LoadBatch<Postgres, RiskSummaryModel> for RiskSummaryRepositoryImpl {
    async fn load_batch(&self, ids: &[Uuid]) -> Result<Vec<Option<RiskSummaryModel>>, Box<dyn Error + Send + Sync>> {
        Self::load_batch_impl(self, ids).await
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helper::setup_test_context;
    use business_core_db::repository::create_batch::CreateBatch;
    use business_core_db::repository::load_batch::LoadBatch;
    use uuid::Uuid;
    use super::super::test_utils::test_utils::{create_test_risk_summary, create_test_person};
    use crate::repository::person::test_utils::create_test_audit_log;

    #[tokio::test]
    async fn test_load_batch() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;
        let risk_summary_repo = &ctx.person_repos().risk_summary_repository;

        // Create audit log
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create person
        let person = create_test_person();
        person_repo.create_batch(vec![person.clone()], Some(audit_log.id)).await?;

        // Create risk summaries
        let risk_summary1 = create_test_risk_summary(person.id);
        let risk_summary2 = create_test_risk_summary(person.id);

        let saved = risk_summary_repo.create_batch(vec![risk_summary1.clone(), risk_summary2.clone()], Some(audit_log.id)).await?;

        // Load the risk summaries
        let ids = vec![saved[0].id, saved[1].id];
        let loaded = risk_summary_repo.load_batch(&ids).await?;

        assert_eq!(loaded.len(), 2);
        assert!(loaded[0].is_some());
        assert!(loaded[1].is_some());
        assert_eq!(loaded[0].as_ref().unwrap().id, saved[0].id);
        assert_eq!(loaded[1].as_ref().unwrap().id, saved[1].id);

        Ok(())
    }

    #[tokio::test]
    async fn test_load_batch_with_non_existing() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let ctx = setup_test_context().await?;
        let audit_log_repo = &ctx.audit_repos().audit_log_repository;
        let person_repo = &ctx.person_repos().person_repository;
        let risk_summary_repo = &ctx.person_repos().risk_summary_repository;

        // Create audit log
        let audit_log = create_test_audit_log();
        audit_log_repo.create(&audit_log).await?;

        // Create person
        let person = create_test_person();
        person_repo.create_batch(vec![person.clone()], Some(audit_log.id)).await?;

        // Create one risk summary
        let risk_summary = create_test_risk_summary(person.id);
        let saved = risk_summary_repo.create_batch(vec![risk_summary.clone()], Some(audit_log.id)).await?;

        // Try to load existing and non-existing
        let non_existing_id = Uuid::new_v4();
        let ids = vec![saved[0].id, non_existing_id];
        let loaded = risk_summary_repo.load_batch(&ids).await?;

        assert_eq!(loaded.len(), 2);
        assert!(loaded[0].is_some());
        assert!(loaded[1].is_none());
        assert_eq!(loaded[0].as_ref().unwrap().id, saved[0].id);

        Ok(())
    }
}