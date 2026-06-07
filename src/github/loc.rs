//! LOC orchestration: paginate all repositories, then delegate to the cache.

use anyhow::Result;
use reqwest::Client;
use serde_json::{json, Value};

use crate::cache::cache_builder;
use crate::config::USER_NAME;
use crate::github::queries::LOC_QUERY;
use crate::github::request::simple_request;
use crate::query_count::query_count;

/// Collects all repository edges (paginated) then delegates to `cache_builder`.
/// Returns `(loc_add, loc_del, loc_diff, cached)`.
pub async fn loc_query(
    client: &Client,
    owner_affiliation: &[&str],
    comment_size: usize,
) -> Result<(i64, i64, i64, bool)> {
    let mut edges: Vec<Value> = Vec::new();
    let mut cursor: Option<String> = None;

    loop {
        query_count("loc_query");
        let variables = json!({
            "owner_affiliation": owner_affiliation,
            "login": *USER_NAME,
            "cursor": cursor,
        });
        let body = simple_request(client, "loc_query", LOC_QUERY, variables).await?;
        let repositories = &body["data"]["user"]["repositories"];

        if let Some(page) = repositories["edges"].as_array() {
            edges.extend(page.iter().cloned());
        }

        if repositories["pageInfo"]["hasNextPage"]
            .as_bool()
            .unwrap_or(false)
        {
            cursor = repositories["pageInfo"]["endCursor"]
                .as_str()
                .map(|s| s.to_string());
        } else {
            break;
        }
    }

    cache_builder(client, &edges, comment_size).await
}
