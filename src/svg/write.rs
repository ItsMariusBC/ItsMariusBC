//! SVG rewriter. Port of `functions/files/svg/write.ts`.
//!
//! Instead of a full XML round-trip (which would re-escape entities and reorder
//! attributes), we do a surgical, byte-preserving regex replacement of the inner
//! text of `<tspan>` elements identified by `id="..."`. Everything else in the
//! file — ASCII art, the `&apos;` style block, dot-leaders of static rows, and
//! the absence of a trailing newline — is left untouched.

use anyhow::Result;
use regex::Regex;
use std::fs;

/// Replace only the inner text of the unique `<tspan ... id="ID" ...>TEXT</tspan>`.
/// `id` may appear anywhere among the attributes. No-op if the id is absent.
fn find_and_replace(svg: &mut String, id: &str, new_text: &str) {
    let pattern = format!(
        r#"(<tspan\b[^>]*\bid="{}"[^>]*>)([^<]*)(</tspan>)"#,
        regex::escape(id)
    );
    let re = Regex::new(&pattern).expect("valid regex");
    if re.is_match(svg) {
        let replaced = re
            .replacen(svg, 1, |caps: &regex::Captures| {
                format!("{}{}{}", &caps[1], new_text, &caps[3])
            })
            .into_owned();
        *svg = replaced;
    }
}

/// Dot-leader string. Port of the `dot_string` logic inside `svg_write`.
fn dot_string(length: i64, text: &str) -> String {
    let just_len = length - text.chars().count() as i64;
    if just_len == 0 {
        ". ".to_string()
    } else if just_len < 0 {
        String::new()
    } else {
        format!(" {} ", ".".repeat(just_len as usize))
    }
}

/// Set the `id` tspan text and its `id_dots` sibling's dot-leader.
fn svg_write(svg: &mut String, id: &str, text: &str, length: i64) {
    find_and_replace(svg, id, text);
    let dots = dot_string(length, text);
    find_and_replace(svg, &format!("{}_dots", id), &dots);
}

/// Inject all data values into an SVG string. `loc_data` is `[add, del, diff]`
/// already formatted with thousands separators.
#[allow(clippy::too_many_arguments)] // mirrors the original svg_overwrite signature
pub fn svg_overwrite_str(
    svg: &mut String,
    age_data: i64,
    commit_data: i64,
    star_data: i64,
    repo_data: i64,
    contrib_data: i64,
    follower_data: i64,
    loc_data: &[String; 3],
) {
    // Uptime line: right-align to column 61 (compensating variable digit counts).
    let age_str = age_data.to_string();
    let plus_str = (age_data - 11).to_string();
    let uptime_fixed = ". Uptime:".len() as i64
        + 2
        + age_str.len() as i64
        + " years on Earth, ".len() as i64
        + plus_str.len() as i64
        + "+ coding".len() as i64;
    svg_write(svg, "age_data", &age_str, (60 - uptime_fixed) + age_str.len() as i64);
    svg_write(svg, "age_data_plus", &plus_str, 2);

    svg_write(svg, "commit_data", &commit_data.to_string(), 22);
    svg_write(svg, "star_data", &star_data.to_string(), 14);
    svg_write(svg, "repo_data", &repo_data.to_string(), 8);
    svg_write(svg, "contrib_data", &contrib_data.to_string(), 0);
    svg_write(svg, "follower_data", &follower_data.to_string(), 10);
    svg_write(svg, "loc_data", &loc_data[2], 7);
    svg_write(svg, "loc_add", &loc_data[0], 5);
    svg_write(svg, "loc_del", &loc_data[1], 5);
}

/// Read an SVG file, inject values, write it back (no trailing newline added).
#[allow(clippy::too_many_arguments)] // mirrors the original svg_overwrite signature
pub fn svg_overwrite(
    filename: &str,
    age_data: i64,
    commit_data: i64,
    star_data: i64,
    repo_data: i64,
    contrib_data: i64,
    follower_data: i64,
    loc_data: &[String; 3],
) -> Result<()> {
    let mut svg = fs::read_to_string(filename)?;
    svg_overwrite_str(
        &mut svg,
        age_data,
        commit_data,
        star_data,
        repo_data,
        contrib_data,
        follower_data,
        loc_data,
    );
    fs::write(filename, svg)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dot_string_cases() {
        assert_eq!(dot_string(7, "12345"), " .. "); // 7-5=2
        assert_eq!(dot_string(5, "12345"), ". "); // equal -> ". "
        assert_eq!(dot_string(5, "46,258"), ""); // negative -> empty
    }

    #[test]
    fn replace_inner_text_both_attr_orders() {
        // class before id
        let mut a = r#"<tspan class="value" id="repo_data">87</tspan>"#.to_string();
        find_and_replace(&mut a, "repo_data", "6");
        assert_eq!(a, r#"<tspan class="value" id="repo_data">6</tspan>"#);
        // id before class (like loc_add_dots in the real file)
        let mut b = r#"<tspan id="loc_add_dots" class="cc"></tspan>"#.to_string();
        find_and_replace(&mut b, "loc_add_dots", "");
        assert_eq!(b, r#"<tspan id="loc_add_dots" class="cc"></tspan>"#);
    }

    #[test]
    fn missing_id_is_noop() {
        let mut s = r#"<tspan class="value" id="contrib_data">6</tspan>"#.to_string();
        let before = s.clone();
        find_and_replace(&mut s, "contrib_data_dots", " . ");
        assert_eq!(s, before);
    }

    fn inner_text(svg: &str, id: &str) -> String {
        let re = Regex::new(&format!(
            r#"<tspan\b[^>]*\bid="{}"[^>]*>([^<]*)</tspan>"#,
            regex::escape(id)
        ))
        .unwrap();
        re.captures(svg)
            .map(|c| c[1].to_string())
            .unwrap_or_default()
    }

    fn num(svg: &str, id: &str) -> i64 {
        inner_text(svg, id).replace(',', "").parse().unwrap_or(0)
    }

    /// Golden parity gate: re-injecting the values ALREADY present in the
    /// committed SVG must produce a byte-identical file (no entity re-escaping,
    /// no attribute reordering, no trailing-newline change). Self-extracting so
    /// it never drifts as live stats update.
    #[test]
    fn golden_roundtrip_is_byte_identical() {
        for path in ["img/dark_mode.svg", "img/light_mode.svg"] {
            let original = std::fs::read_to_string(path).expect("read svg");
            let mut svg = original.clone();
            super::svg_overwrite_str(
                &mut svg,
                num(&original, "age_data"),
                num(&original, "commit_data"),
                num(&original, "star_data"),
                num(&original, "repo_data"),
                num(&original, "contrib_data"),
                num(&original, "follower_data"),
                &[
                    inner_text(&original, "loc_add"),
                    inner_text(&original, "loc_del"),
                    inner_text(&original, "loc_data"),
                ],
            );
            assert_eq!(svg, original, "{} changed when it should not have", path);
        }
    }
}
