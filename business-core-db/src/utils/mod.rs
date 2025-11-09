use serde::Serialize;
use std::hash::Hasher;
use twox_hash::XxHash64;

/// Hashes serializable data into an i64 using CBOR serialization and XxHash64.
///
/// This provides a stable hash across different runs and systems by:
/// - Serializing the data to CBOR format (deterministic binary representation)
/// - Using XxHash64 with a fixed seed (0) for consistent hashing
pub fn hash_as_i64<T: Serialize>(data: &T) -> i64 {
    let mut hasher = XxHash64::with_seed(0);
    let mut cbor = Vec::new();
    ciborium::ser::into_writer(data, &mut cbor).unwrap();
    hasher.write(&cbor);
    hasher.finish() as i64
}