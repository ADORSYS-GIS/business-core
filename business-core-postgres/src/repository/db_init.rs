//! Database initialization and cleanup utilities
//!
//! This module provides functions to initialize and cleanup the PostgreSQL
//! database schema by executing SQL migration and cleanup files.

use sqlx::PgPool;
use std::fs;
use std::path::Path;

/// Initialize the database by executing migration files in ascending order
///
/// This function reads all SQL files from the migrations directory and executes
/// them in alphabetical/numerical order to set up the database schema.
///
/// # Example
///
/// ```rust,no_run
/// use sqlx::PgPool;
/// use business_core_postgres::repository::db_init::init_database;
///
/// # async fn example(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
/// init_database(pool).await?;
/// # Ok(())
/// # }
/// ```
pub async fn init_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    let migrations_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("migrations");
    execute_sql_files_in_order(pool, &migrations_dir, true).await
}

/// Cleanup the database by executing cleanup files in descending order
///
/// This function reads all SQL files from the cleanup directory and executes
/// them in reverse alphabetical/numerical order to tear down the database schema.
///
/// # Example
///
/// ```rust,no_run
/// use sqlx::PgPool;
/// use business_core_postgres::repository::db_init::cleanup_database;
///
/// # async fn example(pool: &PgPool) -> Result<(), Box<dyn std::error::Error>> {
/// cleanup_database(pool).await?;
/// # Ok(())
/// # }
/// ```
pub async fn cleanup_database(pool: &PgPool) -> Result<(), sqlx::Error> {
    let cleanup_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("cleanup");
    execute_sql_files_in_order(pool, &cleanup_dir, false).await
}

/// Execute SQL files from a directory in the specified order
///
/// # Arguments
/// * `pool` - Database connection pool
/// * `dir` - Directory containing SQL files
/// * `ascending` - If true, execute in ascending order; if false, in descending order
async fn execute_sql_files_in_order(
    pool: &PgPool,
    dir: &Path,
    ascending: bool,
) -> Result<(), sqlx::Error> {
    // Read directory entries
    let mut entries: Vec<_> = fs::read_dir(dir)
        .map_err(|e| sqlx::Error::Io(e))?
        .filter_map(|entry| entry.ok())
        .filter(|entry| {
            entry.path().extension().and_then(|s| s.to_str()) == Some("sql")
        })
        .collect();

    // Sort by filename
    entries.sort_by(|a, b| {
        let ordering = a.file_name().cmp(&b.file_name());
        if ascending {
            ordering
        } else {
            ordering.reverse()
        }
    });

    // Execute each SQL file
    for entry in entries {
        let path = entry.path();
        let sql = fs::read_to_string(&path)
            .map_err(|e| sqlx::Error::Io(e))?;
        
        sqlx::raw_sql(&sql).execute(pool).await?;
    }

    Ok(())
}

#[cfg(test)]
#[serial_test::serial]
mod tests {
    use super::*;

    #[tokio::test]
    #[ignore]
    async fn test_init_and_cleanup() -> Result<(), Box<dyn std::error::Error>> {
        let pool = PgPool::connect("postgresql://postgres:postgres@localhost:5433/business_core_db").await?;

        postgres_index_cache::cleanup_cache_triggers(&pool).await?;

        postgres_index_cache::init_cache_triggers(&pool).await?;
        
        // Test initialization
        init_database(&pool).await?;
        
        // Test cleanup
        cleanup_database(&pool).await?;

        postgres_index_cache::cleanup_cache_triggers(&pool).await?;
        
        Ok(())
    }
}