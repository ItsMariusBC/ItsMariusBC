//! Sums the `my_commits` column (field index 2) of each repo line in the cache.

use anyhow::Result;
use std::fs;

use crate::config::user_file_name;

pub fn commit_counter(comment_size: usize) -> Result<i64> {
    let filename = user_file_name();
    let content = fs::read_to_string(&filename)?;
    let lines: Vec<&str> = content.split('\n').collect();
    let mut total = 0i64;
    for line in lines.iter().skip(comment_size) {
        if line.is_empty() {
            continue;
        }
        let parts: Vec<&str> = line.split(' ').collect();
        if parts.len() > 2 {
            total += parts[2].parse::<i64>().unwrap_or(0);
        }
    }
    Ok(total)
}
