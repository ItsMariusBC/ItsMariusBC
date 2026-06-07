//! Timing printout. Port of `functions/infos/perfs.ts`.

/// `difference` is in seconds. Matches the original ms/s formatting + padding.
pub fn print_time_to_run(query_type: &str, difference: f64) {
    let message = if difference > 1.0 {
        pad_start(&format!("{:.4} s ", difference), 12)
    } else {
        pad_start(&format!("{:.4} ms", difference * 1000.0), 12)
    };
    println!("{} {}", pad_end(&format!("   {}:", query_type), 23), message);
}

fn pad_start(s: &str, width: usize) -> String {
    if s.chars().count() >= width {
        s.to_string()
    } else {
        format!("{}{}", " ".repeat(width - s.chars().count()), s)
    }
}

fn pad_end(s: &str, width: usize) -> String {
    if s.chars().count() >= width {
        s.to_string()
    } else {
        format!("{}{}", s, " ".repeat(width - s.chars().count()))
    }
}
