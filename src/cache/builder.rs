//! Incremental LOC cache.

use anyhow::Result;
use reqwest::Client;
use serde_json::Value;
use std::fs;

use crate::config::{sha256_hex, user_file_name};
use crate::github::counter::recursive_loc;

fn name_with_owner(edge: &Value) -> String {
    edge["node"]["nameWithOwner"]
        .as_str()
        .unwrap_or("")
        .to_string()
}

/// Rebuild the cache file from scratch: keep the comment block, then one
/// `<hash> 0 0 0 0` line per repository.
fn wipe_cache(edges: &[Value], filename: &str, comment_size: usize) {
    let mut data: Vec<String> = Vec::new();
    if comment_size > 0 {
        match fs::read_to_string(filename) {
            Ok(content) => {
                data = content
                    .split('\n')
                    .take(comment_size)
                    .map(String::from)
                    .collect();
            }
            Err(_) => {
                for _ in 0..comment_size {
                    data.push("//\n".to_string());
                }
            }
        }
    }
    let mut content = data.join("\n");
    for edge in edges {
        content += &format!("\n{} 0 0 0 0", sha256_hex(&name_with_owner(edge)));
    }
    let _ = fs::write(filename, content);
}

/// Returns `(loc_add, loc_del, loc_add - loc_del, cached)`.
pub async fn cache_builder(
    client: &Client,
    edges: &[Value],
    comment_size: usize,
) -> Result<(i64, i64, i64, bool)> {
    let mut cached = true;
    let filename = user_file_name();

    let mut cache: Vec<String> = match fs::read_to_string(&filename) {
        Ok(content) => content.split('\n').map(String::from).collect(),
        Err(_) => {
            let mut c: Vec<String> = Vec::new();
            if comment_size > 0 {
                for _ in 0..comment_size {
                    c.push("This line is a comment block. Write whatever you want here.\n".to_string());
                }
            }
            let _ = fs::write(&filename, c.join("\n"));
            c
        }
    };

    if (cache.len() as i64 - comment_size as i64) != edges.len() as i64 {
        cached = false;
        wipe_cache(edges, &filename, comment_size);
        let content = fs::read_to_string(&filename)?;
        cache = content.split('\n').map(String::from).collect();
    }

    let cs = comment_size.min(cache.len());
    let cache_comment: Vec<String> = cache[..cs].to_vec();
    let mut body: Vec<String> = cache[cs..].to_vec();

    for index in 0..edges.len() {
        if index >= body.len() || body[index].is_empty() {
            continue;
        }
        let fields: Vec<String> = body[index].split(' ').map(String::from).collect();
        let repo_hash = fields[0].clone();
        let commit_count: i64 = fields.get(1).and_then(|s| s.parse().ok()).unwrap_or(0);

        let name = name_with_owner(&edges[index]);
        if repo_hash != sha256_hex(&name) {
            continue;
        }

        let total_count = edges[index]["node"]["defaultBranchRef"]["target"]["history"]
            ["totalCount"]
            .as_i64();

        match total_count {
            None => {
                // empty repo / no default branch -> reset line
                body[index] = format!("{} 0 0 0 0", repo_hash);
            }
            Some(tc) if commit_count != tc => {
                let (owner, repo_name) = match name.split_once('/') {
                    Some((o, r)) => (o.to_string(), r.to_string()),
                    None => (name.clone(), String::new()),
                };
                match recursive_loc(client, &owner, &repo_name, &body, &cache_comment).await {
                    Ok((add, del, mine)) => {
                        body[index] = format!("{} {} {} {} {}", repo_hash, tc, mine, add, del);
                    }
                    Err(_) => {
                        body[index] = format!("{} 0 0 0 0", repo_hash);
                    }
                }
            }
            Some(_) => { /* counts equal -> keep existing line */ }
        }
    }

    let out = format!("{}\n{}", cache_comment.join("\n"), body.join("\n"));
    let _ = fs::write(&filename, out);

    let (mut loc_add, mut loc_del) = (0i64, 0i64);
    for line in &body {
        if line.is_empty() {
            continue;
        }
        let loc: Vec<&str> = line.split(' ').collect();
        if loc.len() >= 5 {
            loc_add += loc[3].parse::<i64>().unwrap_or(0);
            loc_del += loc[4].parse::<i64>().unwrap_or(0);
        }
    }

    Ok((loc_add, loc_del, loc_add - loc_del, cached))
}
