//! Global GraphQL call counters, in fixed display order.
//! `user_getter` and `graph_commits` are intentionally never incremented.

use std::sync::atomic::{AtomicU64, Ordering};

pub static QUERY_COUNT: [(&str, AtomicU64); 6] = [
    ("user_getter", AtomicU64::new(0)),
    ("follower_getter", AtomicU64::new(0)),
    ("graph_repos_stars", AtomicU64::new(0)),
    ("recursive_loc", AtomicU64::new(0)),
    ("graph_commits", AtomicU64::new(0)),
    ("loc_query", AtomicU64::new(0)),
];

pub fn query_count(id: &str) {
    for (key, value) in &QUERY_COUNT {
        if *key == id {
            value.fetch_add(1, Ordering::Relaxed);
        }
    }
}

/// Snapshot of all counters in fixed order.
pub fn snapshot() -> Vec<(&'static str, u64)> {
    QUERY_COUNT
        .iter()
        .map(|(k, v)| (*k, v.load(Ordering::Relaxed)))
        .collect()
}

/// JSON representation embedded in error messages.
pub fn as_json() -> String {
    let body = QUERY_COUNT
        .iter()
        .map(|(k, v)| format!("\"{}\":{}", k, v.load(Ordering::Relaxed)))
        .collect::<Vec<_>>()
        .join(",");
    format!("{{{}}}", body)
}
