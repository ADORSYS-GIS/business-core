Use the skill  `docs/skills/entity_template/entity_with_index_and_audit.md` to generate model and repository code for the entity `InterestRateTier` using the sample code in `business-core-db/src/models/product/interest_rate_tier_example.rs` .

additional instructions
===
- `InterestRateTier` is auditable and indexable
- do not forget to provide at least single test per repository method
- database scripts are in business-core-postgres/migrations/022_xxx and business-core-postgres/cleanup/022_xxx
- if you are missing any instruction, on handling audit, index cache or main model caching, have a look at business-core-db/src/models/description/named.rs and business-core-postgres/src/repository/description/named_repository- do not forget to test trigger functionality for index, e.g business-core-postgres/src/repository/description/named_repository/create_batch.rs:212-214
```
    #[tokio::test]
    async fn test_named_insert_triggers_cache_notification(
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
```
- delete the `business-core-db/src/models/product/interest_rate_tier_example.rs` when done.
===