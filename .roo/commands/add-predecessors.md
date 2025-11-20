---
description: Add predecessor fields (predecessor_1, predecessor_2, predecessor_3) to an auditable entity
arguments:
  - name: model_path
    description: Path to the database model file (e.g., 'business-core-db/src/models/person/document.rs')
    required: true
---

# Add Predecessor Fields Command

Use the skill 'docs/skills/refactoring/add-fields.md' to add 3 predecessor fields to the specified entity model:

- `predecessor_1: Option<Uuid>` - First predecessor reference (nullable)
- `predecessor_2: Option<Uuid>` - Second predecessor reference (nullable)
- `predecessor_3: Option<Uuid>` - Third predecessor reference (nullable)

## Input
- Model path: `{{model_path}}`

## Instructions

Follow the skill guide to add these three fields to the entity. The fields should:

1. Be added to the model struct before the audit fields
2. Be nullable (`Option<Uuid>`)
3. Be added to both the main table and audit table in the migration scripts
4. Be added to all repository SQL queries (INSERT, UPDATE, SELECT)
5. Use SQL type `UUID` with no constraints

## Example Usage

```
/add-predecessors business-core-db/src/models/person/document.rs
```

This will add the three predecessor fields to the DocumentModel entity.