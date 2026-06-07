use sha2::{Digest, Sha256};
use std::env;
use std::sync::LazyLock;

pub const BIRTH_YEAR: i64 = 2007;
/// 1-indexed month (1 = January). March = 3.
pub const BIRTH_MONTH: i64 = 3;
/// Day of month.
pub const BIRTH_DAY: i64 = 21;
pub const COMMENT_SIZE: usize = 7;

pub static USER_NAME: LazyLock<String> =
    LazyLock::new(|| env::var("USER_NAME").unwrap_or_else(|_| "no user name".to_string()));
pub static USER_ID: LazyLock<String> =
    LazyLock::new(|| env::var("USER_ID").unwrap_or_else(|_| "no user id".to_string()));
pub static ACCESS_TOKEN: LazyLock<String> =
    LazyLock::new(|| env::var("ACCESS_TOKEN").unwrap_or_default());

/// Lowercase hex SHA-256 of a string's UTF-8 bytes.
pub fn sha256_hex(s: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(s.as_bytes());
    hex::encode(hasher.finalize())
}

/// Cache file path: `cache/<sha256(USER_NAME)>.txt`.
pub fn user_file_name() -> String {
    format!("cache/{}.txt", sha256_hex(&USER_NAME))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sha256_matches_cache_filename() {
        // Must match the committed cache filename: cache/<this>.txt
        assert_eq!(
            sha256_hex("ItsMariusBC"),
            "0824a210b33bfbb9c6fc8df895ec775333b51a894127d924e0f2ab466ded3484"
        );
    }
}
