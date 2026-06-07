//! Repos + stars counters. Port of `functions/datas/stars/stars.ts`.

use anyhow::Result;
use reqwest::Client;
use serde_json::{json, Value};

use crate::config::USER_NAME;
use crate::github::queries::STARS_REPOS_QUERY;
use crate::github::request::simple_request;
use crate::query_count::query_count;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CountType {
    OwnerCount,
    ContribCount,
}

fn stars_counter(edges: &[Value]) -> i64 {
    edges
        .iter()
        .map(|repo| {
            repo["node"]["stargazers"]["totalCount"]
                .as_i64()
                .unwrap_or(0)
        })
        .sum()
}

/// Returns `(stars, repos)`. Single request (no pagination), matching the original.
pub async fn graph_repos_stars(
    client: &Client,
    count_type: CountType,
    owner_affiliation: &[&str],
) -> Result<(i64, i64)> {
    query_count("graph_repos_stars");
    let variables = json!({
        "owner_affiliation": owner_affiliation,
        "login": *USER_NAME,
        "cursor": Value::Null,
    });
    let body = simple_request(client, "graph_repos_stars", STARS_REPOS_QUERY, variables).await?;
    let repositories = &body["data"]["user"]["repositories"];
    let repos = repositories["totalCount"].as_i64().unwrap_or(0);
    let stars = match count_type {
        CountType::OwnerCount => {
            let edges = repositories["edges"].as_array().cloned().unwrap_or_default();
            stars_counter(&edges)
        }
        CountType::ContribCount => 0,
    };
    Ok((stars, repos))
}
