I need a file call docs/skills/entity_template/entity_with_index_and_maincache.md. Beside the index of the entity, we also maintain a cache of the entity.

We rely on /home/francis/dev/ledger-rust/postgres-index-cache/src/main_model_cache.rs  and /home/francis/dev/ledger-rust/postgres-index-cache/src/transaction_aware_main_model_cache.rs for the implementation of the main object cache.

Beside the three skills in @/docs/skills/entity_template, i now do need a skill for an rntity which has and index but also can cache the main entity in the application.

- The Cacheable entity is not preloaded, but explicitely added to the cache whenever created or loaded by the repository.
- The database trigger is registered directly on the main entity table.
- The repository uses the transaction aware cache
- A database triger notifies all application nodes when changes occur in the database (after taransaction commit)

Please generate he skill named docs/skills/entity_template/entity_with_index_and_maincache.md