pub mod builder;

pub use builder::cache_builder;

use std::fs;

use crate::config::user_file_name;

/// Persist partial cache on error. Port of `force_close_cache.ts`.
///
/// Reproduces the original join quirk: no separator newline between the comment
/// block and the body.
pub fn force_close_file(data: &[String], cache_comment: &[String]) {
    let filename = user_file_name();
    let content = format!("{}{}", cache_comment.join("\n"), data.join("\n"));
    let _ = fs::write(&filename, content);
    eprintln!(
        "There was an error while fetching all the data. The cache file: {} has the partial data saved.",
        filename
    );
}
