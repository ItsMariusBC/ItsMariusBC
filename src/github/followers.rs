//! Followers counter. Port of `functions/datas/followers/followers.ts`.

use anyhow::Result;
use reqwest::Client;
use serde_json::json;

use crate::github::queries::FOLLOWERS_QUERY;
use crate::github::request::simple_request;
use crate::query_count::query_count;

pub async fn follower_getter(client: &Client, username: &str) -> Result<i64> {
    query_count("follower_getter");
    let body = simple_request(
        client,
        "follower_getter",
        FOLLOWERS_QUERY,
        json!({ "login": username }),
    )
    .await?;
    Ok(body["data"]["user"]["followers"]["totalCount"]
        .as_i64()
        .unwrap_or(0))
}
