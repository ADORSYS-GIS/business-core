use business_core_db::models::person::document::{DocumentModel, DocumentStatus};
use heapless::String as HeaplessString;
use uuid::Uuid;

pub fn create_test_document(person_id: Uuid) -> DocumentModel {
    DocumentModel {
        id: Uuid::new_v4(),
        person_id,
        document_type: HeaplessString::try_from("Passport").unwrap(),
        document_path: Some(HeaplessString::try_from("/documents/passport.pdf").unwrap()),
        status: DocumentStatus::Uploaded,
        predecessor_1: None,
        predecessor_2: None,
        predecessor_3: None,
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}

pub fn create_test_document_with_type(
    person_id: Uuid,
    document_type: &str,
) -> DocumentModel {
    DocumentModel {
        id: Uuid::new_v4(),
        person_id,
        document_type: HeaplessString::try_from(document_type).unwrap(),
        document_path: Some(HeaplessString::try_from("/documents/document.pdf").unwrap()),
        status: DocumentStatus::Uploaded,
        predecessor_1: None,
        predecessor_2: None,
        predecessor_3: None,
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}

pub fn create_test_document_with_status(
    person_id: Uuid,
    status: DocumentStatus,
) -> DocumentModel {
    DocumentModel {
        id: Uuid::new_v4(),
        person_id,
        document_type: HeaplessString::try_from("ID Card").unwrap(),
        document_path: Some(HeaplessString::try_from("/documents/id.pdf").unwrap()),
        status,
        predecessor_1: None,
        predecessor_2: None,
        predecessor_3: None,
        antecedent_hash: 0,
        antecedent_audit_log_id: Uuid::nil(),
        hash: 0,
        audit_log_id: None,
    }
}