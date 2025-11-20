use business_core_db::models::person::activity_log::ActivityLogModel;
use heapless::String as HeaplessString;
use uuid::Uuid;

pub fn create_test_activity_log(person_id: Uuid) -> ActivityLogModel {
    ActivityLogModel {
        id: Uuid::new_v4(),
        person_id,
        activity_summary: Some(HeaplessString::try_from("Test activity summary").unwrap()),
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}

pub fn create_test_activity_log_with_summary(
    person_id: Uuid,
    summary: &str,
) -> ActivityLogModel {
    ActivityLogModel {
        id: Uuid::new_v4(),
        person_id,
        activity_summary: Some(HeaplessString::try_from(summary).unwrap()),
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}