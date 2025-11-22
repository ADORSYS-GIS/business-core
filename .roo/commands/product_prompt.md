Crate a command .roo/commands/account_gl_mapping.md that will use the skill  docs/skills/entity_template/entity_with_index_and_audit_and_maincache.md to generate model and repository code for the entity `AccountGlMapping` using the sample code in business-core-db/src/models/product/account_gl_mapping_example.rs .

additional instructions
===
- `AccountGlMapping` is auditable, indexable and cachable
- do not forget to provide at least single test per repository method
- do not forget to test trigger functionality for index and for main object.
- if you are missing any instruction, on handling index cache or main model caching, have a look at business-core-db/src/models/calendar/business_day.rs and business-core-postgres/src/repository/calendar/business_day_repository
- if you are missing any instruction, on handling audit functionality, have a look at business-core-db/src/models/person/person.rs and business-core-postgres/src/repository/person/person_repository
- Delete the business-core-db/src/models/product/account_gl_mapping_example.rs when done.
===