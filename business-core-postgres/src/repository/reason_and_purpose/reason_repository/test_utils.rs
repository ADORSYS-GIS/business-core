#[cfg(test)]
pub mod test_utils {
    use business_core_db::models::reason_and_purpose::reason::{
        ReasonModel, ReasonCategory, ReasonContext, ReasonSeverity
    };
    use heapless::String as HeaplessString;
    use uuid::Uuid;

    pub fn create_test_reason(code: &str, content: &str) -> ReasonModel {
        ReasonModel {
            id: Uuid::new_v4(),
            code: HeaplessString::try_from(code).unwrap(),
            category: ReasonCategory::Compliance,
            context: ReasonContext::Transaction,
            l1_content: Some(HeaplessString::try_from(content).unwrap()),
            l2_content: None,
            l3_content: None,
            l1_language_code: Some(HeaplessString::try_from("eng").unwrap()),
            l2_language_code: None,
            l3_language_code: None,
            requires_details: false,
            is_active: true,
            severity: Some(ReasonSeverity::Medium),
            display_order: 0,
            compliance_metadata: None,
        }
    }

    pub fn create_test_reason_with_category(
        code: &str,
        content: &str,
        category: ReasonCategory,
    ) -> ReasonModel {
        ReasonModel {
            id: Uuid::new_v4(),
            code: HeaplessString::try_from(code).unwrap(),
            category,
            context: ReasonContext::Transaction,
            l1_content: Some(HeaplessString::try_from(content).unwrap()),
            l2_content: None,
            l3_content: None,
            l1_language_code: Some(HeaplessString::try_from("eng").unwrap()),
            l2_language_code: None,
            l3_language_code: None,
            requires_details: false,
            is_active: true,
            severity: Some(ReasonSeverity::Medium),
            display_order: 0,
            compliance_metadata: None,
        }
    }

    pub fn create_test_reason_with_context(
        code: &str,
        content: &str,
        context: ReasonContext,
    ) -> ReasonModel {
        ReasonModel {
            id: Uuid::new_v4(),
            code: HeaplessString::try_from(code).unwrap(),
            category: ReasonCategory::Compliance,
            context,
            l1_content: Some(HeaplessString::try_from(content).unwrap()),
            l2_content: None,
            l3_content: None,
            l1_language_code: Some(HeaplessString::try_from("eng").unwrap()),
            l2_language_code: None,
            l3_language_code: None,
            requires_details: false,
            is_active: true,
            severity: Some(ReasonSeverity::Medium),
            display_order: 0,
            compliance_metadata: None,
        }
    }

    pub fn create_test_reason_with_compliance_metadata(
        code: &str,
        content: &str,
        compliance_metadata: Option<Uuid>,
    ) -> ReasonModel {
        ReasonModel {
            id: Uuid::new_v4(),
            code: HeaplessString::try_from(code).unwrap(),
            category: ReasonCategory::Compliance,
            context: ReasonContext::Transaction,
            l1_content: Some(HeaplessString::try_from(content).unwrap()),
            l2_content: None,
            l3_content: None,
            l1_language_code: Some(HeaplessString::try_from("eng").unwrap()),
            l2_language_code: None,
            l3_language_code: None,
            requires_details: false,
            is_active: true,
            severity: Some(ReasonSeverity::Medium),
            display_order: 0,
            compliance_metadata,
        }
    }
}