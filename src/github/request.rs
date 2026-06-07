use anyhow::{bail, Result};
use reqwest::Client;
use serde_json::{json, Value};

use crate::config::ACCESS_TOKEN;
use crate::query_count;

pub const GITHUB_GRAPHQL: &str = "https://api.github.com/graphql";

/// Build the shared HTTP client.
/// A `User-Agent` header is required — GitHub's GraphQL API returns 403 without it.
pub fn build_client() -> Result<Client> {
    use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, USER_AGENT};
    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("token {}", &*ACCESS_TOKEN))?,
    );
    headers.insert(USER_AGENT, HeaderValue::from_static("itsmariusbc-readme"));
    Ok(Client::builder().default_headers(headers).build()?)
}

/// Low-level POST returning the HTTP status and the JSON body.
pub async fn post_graphql(
    client: &Client,
    query: &str,
    variables: Value,
) -> Result<(reqwest::StatusCode, Value)> {
    let resp = client
        .post(GITHUB_GRAPHQL)
        .json(&json!({ "query": query, "variables": variables }))
        .send()
        .await?;
    let status = resp.status();
    let body: Value = resp.json().await.unwrap_or(Value::Null);
    Ok((status, body))
}

/// POST, require HTTP 200, return the JSON body.
pub async fn simple_request(
    client: &Client,
    func_name: &str,
    query: &str,
    variables: Value,
) -> Result<Value> {
    let (status, body) = match post_graphql(client, query, variables).await {
        Ok(v) => v,
        Err(e) => bail!("{} request failed: {}", func_name, e),
    };
    if status.as_u16() == 200 {
        return Ok(body);
    }
    bail!(
        "{} has failed with a {} {}",
        func_name,
        status.as_u16(),
        query_count::as_json()
    );
}
