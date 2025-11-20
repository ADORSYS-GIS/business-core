# Skill: Adding Fields to an Auditable Object

This guide explains how to add new fields to an existing auditable entity in the business-core architecture.

## Architecture Overview

The architecture consists of three main components:

1. **Database Model** (`business-core-db/src/models/{module}/{entity}.rs`)
   - Defines the Rust struct representing the entity
   - Includes audit fields: `hash`, `audit_log_id`, `antecedent_hash`, `antecedent_audit_log_id`

2. **Repository** (`business-core-postgres/src/repository/{module}/{entity}_repository/`)
   - Contains CRUD operations and finder methods
   - Handles audit trail creation and updates

3. **Database Migrations**
   - Init: `business-core-postgres/migrations/<n>_initial_schema_{module}_{entity}.sql`
   - Cleanup: `business-core-postgres/cleanup/<n>_cleanup_{module}_{entity}.sql`

## Steps to Add Fields

### 1. Update the Database Model

**File**: `business-core-db/src/models/{module}/{entity}.rs`

Add the new field to the struct:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct DocumentModel {
    pub id: Uuid,
    pub person_id: Uuid,
    pub document_type: HeaplessString<50>,
    pub document_path: Option<HeaplessString<500>>,
    pub status: DocumentStatus,
    
    // NEW FIELD - Add here
    pub expiry_date: Option<chrono::NaiveDate>,
    
    // Audit fields (always at the end)
    pub antecedent_hash: i64,
    pub antecedent_audit_log_id: Uuid,
    pub hash: i64,
    pub audit_log_id: Option<Uuid>,
}
```

**Key Points**:
- Add the field before the audit fields
- Use appropriate types (`HeaplessString<N>` for strings, `Option<T>` for nullable fields)
- Add documentation comments if the field requires explanation

### 2. Update the Migration Script

**File**: `business-core-postgres/migrations/<n>_initial_schema_{module}_{entity}.sql`

Add the column to both the main table and the audit table:

```sql
-- Main Entity Table
CREATE TABLE IF NOT EXISTS person_document (
    id UUID PRIMARY KEY,
    person_id UUID NOT NULL,
    document_type VARCHAR(50) NOT NULL,
    document_path VARCHAR(500),
    status document_status NOT NULL,
    -- NEW FIELD
    expiry_date DATE,
    hash BIGINT NOT NULL DEFAULT 0,
    audit_log_id UUID REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000'
);

-- Audit Table (must match main table fields)
CREATE TABLE IF NOT EXISTS person_document_audit (
    id UUID NOT NULL,
    person_id UUID NOT NULL,
    document_type VARCHAR(50) NOT NULL,
    document_path VARCHAR(500),
    status document_status NOT NULL,
    -- NEW FIELD
    expiry_date DATE,
    
    -- Audit-specific fields
    hash BIGINT NOT NULL,
    audit_log_id UUID NOT NULL REFERENCES audit_log(id),
    antecedent_hash BIGINT NOT NULL DEFAULT 0,
    antecedent_audit_log_id UUID NOT NULL DEFAULT '00000000-0000-0000-0000-000000000000',
    
    PRIMARY KEY (id, audit_log_id)
);
```

**Key Points**:
- Add the column in **both** tables (main and audit)
- Place the new field before the audit fields
- Match the SQL type to the Rust type (VARCHAR for HeaplessString, DATE for NaiveDate, etc.)
- Use appropriate constraints (NOT NULL, DEFAULT values)

### 3. Update Repository Operations

For each repository operation that includes field-level queries, update the SQL statements:

#### 3.1 Update Batch (`update_batch.rs`)

**File**: `business-core-postgres/src/repository/{module}/{entity}_repository/update_batch.rs`

Update the INSERT and UPDATE queries:

```rust
// Audit insert query
let audit_insert_query = sqlx::query(
    r#"
    INSERT INTO person_document_audit
    (id, person_id, document_type, document_path, status, expiry_date, 
     antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
    "#,
)
.bind(entity.id)
.bind(entity.person_id)
.bind(entity.document_type.as_str())
.bind(entity.document_path.as_deref())
.bind(entity.status)
.bind(entity.expiry_date)  // NEW FIELD
.bind(entity.antecedent_hash)
.bind(entity.antecedent_audit_log_id)
.bind(entity.hash)
.bind(entity.audit_log_id);

// Entity update query
let rows_affected = sqlx::query(
    r#"
    UPDATE person_document SET
        person_id = $2,
        document_type = $3,
        document_path = $4,
        status = $5,
        expiry_date = $6,  -- NEW FIELD
        antecedent_hash = $7,
        antecedent_audit_log_id = $8,
        hash = $9,
        audit_log_id = $10
    WHERE id = $1
      AND hash = $11
      AND audit_log_id = $12
    "#,
)
.bind(entity.id)
.bind(entity.person_id)
.bind(entity.document_type.as_str())
.bind(entity.document_path.as_deref())
.bind(entity.status)
.bind(entity.expiry_date)  // NEW FIELD
.bind(entity.antecedent_hash)
.bind(entity.antecedent_audit_log_id)
.bind(entity.hash)
.bind(entity.audit_log_id)
.bind(previous_hash)
.bind(previous_audit_log_id)
.execute(&mut **transaction)
.await?
.rows_affected();
```

#### 3.2 Create Batch (`create_batch.rs`)

Similar updates for INSERT queries in the create operation.

#### 3.3 Delete Batch (`delete_batch.rs`)

**File**: `business-core-postgres/src/repository/{module}/{entity}_repository/delete_batch.rs`

Update the INSERT query for the audit table (the entity is being deleted, but we record one final audit entry):

```rust
// Audit insert query
let audit_insert_query = sqlx::query(
    r#"
    INSERT INTO person_document_audit
    (id, person_id, document_type, document_path, status, expiry_date,
     antecedent_hash, antecedent_audit_log_id, hash, audit_log_id)
    VALUES ($1, $2, $3, $4, $5, $6, $7, $8, $9, $10)
    "#,
)
.bind(final_audit_entity.id)
.bind(final_audit_entity.person_id)
.bind(final_audit_entity.document_type.as_str())
.bind(final_audit_entity.document_path.as_deref())
.bind(final_audit_entity.status)
.bind(final_audit_entity.expiry_date)  // NEW FIELD
.bind(final_audit_entity.antecedent_hash)
.bind(final_audit_entity.antecedent_audit_log_id)
.bind(final_audit_entity.hash)
.bind(final_audit_entity.audit_log_id);
```

#### 3.4 Load Operations (`load_batch.rs`, `load_audits.rs`)

If using custom SELECT queries (not `SELECT *`), add the new field to the SELECT clause.

### 4. Create Migration for Existing Systems

If the entity already exists in production, create an additional migration script:

**File**: `business-core-postgres/migrations/<n+1>_add_{field}_to_{module}_{entity}.sql`

```sql
-- Add expiry_date to existing document tables
ALTER TABLE person_document ADD COLUMN IF NOT EXISTS expiry_date DATE;
ALTER TABLE person_document_audit ADD COLUMN IF NOT EXISTS expiry_date DATE;
```

## Checklist

- [ ] Update the model struct in `business-core-db/src/models/{module}/{entity}.rs`
- [ ] Update the initial schema migration script (both main and audit tables)
- [ ] Update `update_batch.rs` (INSERT and UPDATE queries)
- [ ] Update `create_batch.rs` (INSERT queries)
- [ ] Update `delete_batch.rs` (INSERT queries for audit)
- [ ] Update `repo_impl.rs` (update `try_from_row`)
- [ ] Check `load_batch.rs` and `load_audits.rs` if custom SELECT queries are used
- [ ] Create an ALTER TABLE migration if the entity already exists in production
- [ ] Update tests that create test entities with the new field
- [ ] Compile and test all changes

## Important Notes

1. **Audit Table Parity**: The audit table MUST have the same business fields as the main table
2. **Field Placement**: Always add new fields before the audit fields (`hash`, `audit_log_id`, etc.)
3. **Bind Order**: In SQL queries, ensure `.bind()` calls match the parameter order ($1, $2, etc.)
4. **Hash Calculation**: The entity hash is automatically recalculated based on all fields (including new ones)
5. **Testing**: Update test utilities in `test_utils.rs` to include the new field

## Example: Document Model

See the complete example in:
- Model: `business-core-db/src/models/person/document.rs`
- Repository: `business-core-postgres/src/repository/person/document_repository/`
- Migration: `business-core-postgres/migrations/018_initial_schema_person_document.sql`