use std::error::Error;

use business_core_db::{
    models::product::fee_type_gl_mapping::FeeTypeGlMappingModel,
    search::pageable::{Page, PageRequest},
};
use uuid::Uuid;

use super::repo_impl::FeeTypeGlMappingRepositoryImpl;

impl FeeTypeGlMappingRepositoryImpl {
    pub async fn load_audits_internal(
        &self,
        id: Uuid,
        page_req: PageRequest,
    ) -> Result<Page<FeeTypeGlMappingModel>, Box<dyn Error + Send + Sync>> {
        let mut tx = self.executor.tx.lock().await;
        let transaction = tx.as_mut().ok_or("Transaction has been consumed")?;
        let count: i64 = sqlx::query_scalar(
            r#"
            SELECT COUNT(*) FROM fee_type_gl_mapping_audit WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_one(&mut **transaction)
        .await?;

        let items = sqlx::query_as::<_, FeeTypeGlMappingModel>(
            r#"
            SELECT * FROM fee_type_gl_mapping_audit WHERE id = $1
            ORDER BY audit_log_id
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(id)
        .bind(page_req.get_limit())
        .bind(page_req.get_offset())
        .fetch_all(&mut **transaction)
        .await?;

        Ok(Page::new(items, count, page_req))
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;

    use crate::{repository::db_init::DbInitializer, test_helper::TestHelper};
    use business_core_db::{models::product::fee_type_gl_mapping::FeeType, search::pageable::PageRequest};

    use super::*;

    #[tokio::test]
    async fn test_load_audits() -> Result<(), Box<dyn Error + Send + Sync>> {
        let db = DbInitializer::init_test_db().await?;
        let helper = TestHelper::new(db.clone());
        let repo = FeeTypeGlMappingRepositoryImpl::new(db.clone());

        let audit_log = helper.create_audit_log().await?;
        let item = super::super::test_utils::create_test_fee_type_gl_mapping(
            FeeType::InterestExpense,
            "GLCODE1",
        );
        let item_id = item.id;
        let mut saved = repo.create_batch(vec![item], Some(audit_log.id)).await?;

        for i in 1..=3 {
            let audit_log = helper.create_audit_log().await?;
            let mut updated = saved[0].clone();
            updated.gl_code = format!("GLCODE{}", i + 1).try_into().unwrap();
            saved = repo.update_batch(vec![updated], Some(audit_log.id)).await?;
        }

        let page = repo.load_audits(item_id, PageRequest::new(2, 0)).await?;
        assert_eq!(page.total, 4);
        assert_eq!(page.items.len(), 2);
        assert_eq!(page.page_number(), 1);
        assert_eq!(page.total_pages(), 2);
        assert!(page.has_more());

        let page2 = repo.load_audits(item_id, PageRequest::new(2, 2)).await?;
        assert_eq!(page2.total, 4);
        assert_eq!(page2.items.len(), 2);
        assert_eq!(page2.page_number(), 2);
        assert!(!page2.has_more());

        Ok(())
    }

    #[tokio::test]
    async fn test_load_audits_empty() -> Result<(), Box<dyn Error + Send + Sync>> {
        let db = DbInitializer::init_test_db().await?;
        let repo = FeeTypeGlMappingRepositoryImpl::new(db.clone());

        let non_existing_id = Uuid::new_v4();
        let page = repo.load_audits(non_existing_id, PageRequest::new(20, 0)).await?;

        assert_eq!(page.total, 0);
        assert_eq!(page.items.len(), 0);
        assert_eq!(page.page_number(), 1);
        assert!(!page.has_more());

        Ok(())
    }
}