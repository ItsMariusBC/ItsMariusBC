//! Per-repo additions/deletions/commits counter.
//! Port of `functions/datas/lines_of_code/counter.ts` with the async recursion
//! flattened into an iterative page loop.

use anyhow::{bail, Result};
use reqwest::Client;
use serde_json::json;

use crate::cache::force_close_file;
use crate::config::USER_ID;
use crate::github::queries::RECURSIVE_LOC_QUERY;
use crate::github::request::post_graphql;
use crate::query_count::{as_json, query_count};

/// Returns `(additions, deletions, my_commits)` for one repository, walking the
/// full commit history. On any non-200/error, persists the partial cache
/// (`force_close_file`) and propagates the error, like the original.
pub async fn recursive_loc(
    client: &Client,
    owner: &str,
    repo_name: &str,
    data: &[String],
    cache_comment: &[String],
) -> Result<(i64, i64, i64)> {
    let (mut addition_total, mut deletion_total, mut my_commits) = (0i64, 0i64, 0i64);
    let mut cursor: Option<String> = None;

    loop {
        query_count("recursive_loc");
        let variables = json!({
            "repo_name": repo_name,
            "owner": owner,
            "cursor": cursor,
        });

        let (status, body) = match post_graphql(client, RECURSIVE_LOC_QUERY, variables).await {
            Ok(v) => v,
            Err(e) => {
                force_close_file(data, cache_comment);
                return Err(e);
            }
        };

        if status.as_u16() != 200 {
            force_close_file(data, cache_comment);
            if status.as_u16() == 403 {
                bail!("Too many requests in a short amount of time!\nNon-documented rate-limit reached!");
            }
            bail!(
                "recursive_loc() has failed with a {} {}",
                status.as_u16(),
                as_json()
            );
        }

        let default_branch_ref = &body["data"]["repository"]["defaultBranchRef"];
        if default_branch_ref.is_null() {
            return Ok((0, 0, 0));
        }

        let history = &default_branch_ref["target"]["history"];
        let edges = history["edges"].as_array().cloned().unwrap_or_default();

        for node in &edges {
            let commit = &node["node"];
            let author_user = &commit["author"]["user"];
            if author_user.is_object() && author_user["id"].as_str() == Some(USER_ID.as_str()) {
                my_commits += 1;
                addition_total += commit["additions"].as_i64().unwrap_or(0);
                deletion_total += commit["deletions"].as_i64().unwrap_or(0);
            }
        }

        let has_next = history["pageInfo"]["hasNextPage"].as_bool().unwrap_or(false);
        if edges.is_empty() || !has_next {
            return Ok((addition_total, deletion_total, my_commits));
        }
        cursor = history["pageInfo"]["endCursor"]
            .as_str()
            .map(|s| s.to_string());
    }
}
