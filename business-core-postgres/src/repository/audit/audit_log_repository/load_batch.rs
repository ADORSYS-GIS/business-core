use business_core_db::models::audit::AuditLogModel;
use postgres_unit_of_work::Executor;
use uuid::Uuid;

pub async fn load_batch_impl(
    executor: &Executor,
    ids: &[Uuid],
) -> Result<Vec<Option<AuditLogModel>>, Box<dyn std::error::Error + Send + Sync>> {
    if ids.is_empty() {
        return Ok(vec![]);
    }

    let query = sqlx::query_as::<_, AuditLogModel>(
        r#"
        SELECT id, updated_at, updated_by_person_id
        FROM audit_log
        WHERE id = ANY($1)
        "#,
    )
    .bind(ids);

    // Execute query using the new executor structure
    let mut tx = executor.tx.lock().await;
    let rows = if let Some(transaction) = tx.as_mut() {
        query.fetch_all(&mut **transaction).await?
    } else {
        return Err("Transaction has been consumed".into());
    };

    // Build a map of id -> model
    let mut map: std::collections::HashMap<Uuid, AuditLogModel> = rows
        .into_iter()
        .map(|model| (model.id, model))
        .collect();

    // Return results in the same order as input ids
    let result = ids
        .iter()
        .map(|id| map.remove(id))
        .collect();

    Ok(result)
}