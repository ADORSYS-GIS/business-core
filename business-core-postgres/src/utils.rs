use heapless::String as HeaplessString;
use sqlx::{postgres::PgRow, Row};
use std::error::Error;
use serde::Serialize;
use std::str::FromStr;
use blake3::Hasher as Blake3Hasher;

/// A trait for converting a database row into a model.
pub trait TryFromRow<R>: Sized {
    /// Performs the conversion.
    fn try_from_row(row: &R) -> Result<Self, Box<dyn Error + Send + Sync>>;
}

/// Retrieves a required `HeaplessString` from a row.
pub fn get_heapless_string<const N: usize>(
    row: &PgRow,
    col_name: &str,
) -> Result<HeaplessString<N>, Box<dyn Error + Send + Sync>> {
    let s: String = row.try_get(col_name)?;
    HeaplessString::from_str(&s).map_err(|_| {
        format!("Value for column '{col_name}' is too long (max {N} chars)").into()
    })
}

/// Retrieves an optional `HeaplessString` from a row.
pub fn get_optional_heapless_string<const N: usize>(
    row: &PgRow,
    col_name: &str,
) -> Result<Option<HeaplessString<N>>, Box<dyn Error + Send + Sync>> {
    let s: Option<String> = row.try_get(col_name)?;
    s.map(|val| HeaplessString::from_str(&val))
        .transpose()
        .map_err(|_| {
            format!("Value for column '{col_name}' is too long (max {N} chars)").into()
        })
}

pub fn hash_as_i64<T: Serialize>(data: &T) -> i64 {
    let mut hasher = Blake3Hasher::new();
    let json = serde_json::to_vec(data).unwrap();
    hasher.update(&json);
    let hash = hasher.finalize();
    i64::from_le_bytes(hash.as_bytes()[0..8].try_into().unwrap())
}