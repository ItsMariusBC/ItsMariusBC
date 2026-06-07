//! Number formatting helpers.

/// Group an integer with `,` every three digits (en-US), handling negatives.
/// `21114 -> "21,114"`, `-1234 -> "-1,234"`, `0 -> "0"`.
pub fn thousands(n: i64) -> String {
    let negative = n < 0;
    let digits = n.unsigned_abs().to_string();
    let bytes = digits.as_bytes();
    let mut out = String::with_capacity(digits.len() + digits.len() / 3 + 1);
    let len = bytes.len();
    for (i, b) in bytes.iter().enumerate() {
        if i > 0 && (len - i).is_multiple_of(3) {
            out.push(',');
        }
        out.push(*b as char);
    }
    if negative {
        format!("-{}", out)
    } else {
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn grouping() {
        assert_eq!(thousands(0), "0");
        assert_eq!(thousands(100), "100");
        assert_eq!(thousands(21114), "21,114");
        assert_eq!(thousands(1155289), "1,155,289");
        assert_eq!(thousands(-1234), "-1,234");
    }
}
