mod cache;
mod config;
mod format;
mod github;
mod perfs;
mod query_count;
mod svg;

use anyhow::Result;
use std::time::Instant;

use config::{COMMENT_SIZE, USER_NAME};
use format::thousands;
use github::age::age_data;
use github::commits::commit_counter;
use github::followers::follower_getter;
use github::loc::loc_query;
use github::request::build_client;
use github::stars::{graph_repos_stars, CountType};
use perfs::print_time_to_run;
use svg::svg_overwrite;

/// Time a future, returning `(value, seconds)`.
async fn timed<F, T>(fut: F) -> (T, f64)
where
    F: std::future::Future<Output = T>,
{
    let start = Instant::now();
    let value = fut.await;
    (value, start.elapsed().as_secs_f64())
}

fn pad_start(s: &str, width: usize) -> String {
    let n = s.chars().count();
    if n >= width {
        s.to_string()
    } else {
        format!("{}{}", " ".repeat(width - n), s)
    }
}

fn pad_end(s: &str, width: usize) -> String {
    let n = s.chars().count();
    if n >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - n))
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    let client = build_client()?;

    println!("Calculation times:");

    let (age, age_time) = timed(async { age_data() }).await;
    print_time_to_run("age calculation", age_time);

    let (total_loc, loc_time) = timed(loc_query(
        &client,
        &["OWNER", "COLLABORATOR", "ORGANIZATION_MEMBER"],
        COMMENT_SIZE,
    ))
    .await;
    let (loc_add, loc_del, loc_diff, cached) = total_loc?;
    if cached {
        print_time_to_run("LOC (cached)", loc_time);
    } else {
        print_time_to_run("LOC (no cache)", loc_time);
    }

    let (commit_res, commit_time) = timed(async { commit_counter(COMMENT_SIZE) }).await;
    let commit_data = commit_res?;
    print_time_to_run("get commits count", commit_time);

    let (owner_res, star_repo_time) =
        timed(graph_repos_stars(&client, CountType::OwnerCount, &["OWNER"])).await;
    let (stars, repos) = owner_res?;
    print_time_to_run("get stars and repos", star_repo_time);

    let (contrib_res, contrib_time) = timed(graph_repos_stars(
        &client,
        CountType::ContribCount,
        &["OWNER", "COLLABORATOR", "ORGANIZATION_MEMBER"],
    ))
    .await;
    let (_, contrib_repos) = contrib_res?;
    print_time_to_run("get contrib repos", contrib_time);

    let (follower_res, follower_time) = timed(follower_getter(&client, &USER_NAME)).await;
    let follower_data = follower_res?;
    print_time_to_run("get followers count", follower_time);

    let loc_data = [thousands(loc_add), thousands(loc_del), thousands(loc_diff)];

    for filename in ["img/dark_mode.svg", "img/light_mode.svg"] {
        svg_overwrite(
            filename,
            age,
            commit_data,
            stars,
            repos,
            contrib_repos,
            follower_data,
            &loc_data,
        )?;
    }

    let total_time = age_time + loc_time + commit_time + star_repo_time + contrib_time + follower_time;
    println!(
        "{} {}",
        pad_end("\nTotal function time:", 21),
        pad_start(&format!("{:.4}s", total_time), 11)
    );

    let snapshot = query_count::snapshot();
    let total_calls: u64 = snapshot.iter().map(|(_, c)| *c).sum();
    println!(
        "\nTotal GitHub GraphQL API calls: {}",
        pad_start(&total_calls.to_string(), 3)
    );
    for (name, count) in snapshot {
        println!(
            "{} {}",
            pad_end(&format!("   {}:", name), 28),
            pad_start(&count.to_string(), 6)
        );
    }

    Ok(())
}
