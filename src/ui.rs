use colored::*;
use std::io::IsTerminal;

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum BadgeKind {
    Success,
    Warn,
    Error,
    Info,
}

pub fn should_use_color(no_color_flag: bool) -> bool {
    if no_color_flag {
        return false;
    }
    if std::env::var_os("NO_COLOR").is_some() {
        return false;
    }
    if let Ok(force) = std::env::var("CLICOLOR_FORCE") {
        if force != "0" && !force.is_empty() {
            return true;
        }
    }
    std::io::stdout().is_terminal()
}

pub fn visible_len(s: &str) -> usize {
    let mut len = 0usize;
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' && chars.peek() == Some(&'[') {
            chars.next();
            for code_ch in chars.by_ref() {
                if code_ch.is_ascii_alphabetic() {
                    break;
                }
            }
        } else {
            len += 1;
        }
    }
    len
}

pub fn pad_right_visible(s: &str, target_width: usize) -> String {
    let vis = visible_len(s);
    if vis >= target_width {
        s.to_string()
    } else {
        format!("{s}{}", " ".repeat(target_width - vis))
    }
}

pub fn format_badge(label: &str, kind: BadgeKind) -> String {
    let tag = format!(" {label} ");
    match kind {
        BadgeKind::Success => tag.black().on_bright_green().bold().to_string(),
        BadgeKind::Warn => tag.black().on_bright_yellow().bold().to_string(),
        BadgeKind::Error => tag.white().on_red().bold().to_string(),
        BadgeKind::Info => tag.black().on_bright_cyan().bold().to_string(),
    }
}

pub fn format_bytes(bytes: u64) -> String {
    const KB: f64 = 1024.0;
    const MB: f64 = KB * 1024.0;
    const GB: f64 = MB * 1024.0;
    const TB: f64 = GB * 1024.0;

    let b = bytes as f64;
    if b >= TB {
        format!("{:.2} TB", b / TB)
    } else if b >= GB {
        format!("{:.2} GB", b / GB)
    } else if b >= MB {
        format!("{:.2} MB", b / MB)
    } else if b >= KB {
        format!("{:.1} KB", b / KB)
    } else {
        format!("{bytes} B")
    }
}

pub fn format_number_commas(n: u64) -> String {
    let s = n.to_string();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    for (idx, ch) in s.chars().rev().enumerate() {
        if idx > 0 && idx % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    out.chars().rev().collect()
}

pub fn format_duration_human(abs_secs: u64) -> String {
    let days = abs_secs / 86400;
    let hours = (abs_secs % 86400) / 3600;
    let mins = (abs_secs % 3600) / 60;
    let secs = abs_secs % 60;

    if days > 0 {
        format!("{days}d {hours}h")
    } else if hours > 0 {
        format!("{hours}h {mins}m")
    } else if mins > 0 {
        format!("{mins}m {secs}s")
    } else {
        format!("{secs}s")
    }
}

pub fn progress_bar(ratio: f64, width: usize) -> String {
    let clamped = ratio.clamp(0.0, 1.0);
    let filled = (clamped * width as f64).round() as usize;
    let empty = width.saturating_sub(filled);

    let bar_fill = "█".repeat(filled);
    let bar_empty = "░".repeat(empty);

    let colored_fill = if clamped >= 0.85 {
        bar_fill.bright_red().to_string()
    } else if clamped >= 0.65 {
        bar_fill.bright_yellow().to_string()
    } else {
        bar_fill.bright_green().to_string()
    };

    format!(
        "{}{}{}{} {:>5.1}%",
        "[".dimmed(),
        colored_fill,
        bar_empty.dimmed(),
        "]".dimmed(),
        clamped * 100.0
    )
}

/// Renders a closed rounded box card for fixed-width structured dashboards and tables.
pub fn print_boxed_lines(title: &str, lines: &[String]) {
    let min_inner = 56usize;
    let title_len = visible_len(title) + 4;
    let max_line = lines.iter().map(|l| visible_len(l)).max().unwrap_or(0) + 4;
    let inner_width = min_inner.max(title_len).max(max_line);

    let dash_fill = inner_width.saturating_sub(visible_len(title) + 3);
    println!(
        "{} {} {}{}",
        "╭─".bright_cyan(),
        title.bold().bright_white(),
        "─".repeat(dash_fill).bright_cyan(),
        "╮".bright_cyan()
    );

    for line in lines {
        let padded = pad_right_visible(line, inner_width - 4);
        println!("{}  {}  {}", "│".bright_cyan(), padded, "│".bright_cyan());
    }

    println!(
        "{}{}{}",
        "╰".bright_cyan(),
        "─".repeat(inner_width).bright_cyan(),
        "╯".bright_cyan()
    );
}

/// Renders an open-right accent panel for outputs with variable-length copyable values
/// (hashes, tokens, base64, hex, URLs) so terminal text selection never captures a right border.
pub fn print_open_panel(title: &str, rows: &[(&str, String)]) {
    let label_width = rows.iter().map(|(k, _)| k.len()).max().unwrap_or(8).max(8);
    let line_len = (visible_len(title) + 6).max(52);
    let tail = line_len.saturating_sub(visible_len(title) + 3);

    println!(
        "{} {} {}",
        "╭─".bright_cyan(),
        title.bold().bright_white(),
        "─".repeat(tail).bright_cyan()
    );

    for (key, val) in rows {
        let key_padded = format!("{key:<label_width$}");
        println!(
            "{}  {}  {}",
            "│".bright_cyan(),
            key_padded.dimmed().bold(),
            val
        );
    }

    println!(
        "{}{}",
        "╰".bright_cyan(),
        "─".repeat(line_len).bright_cyan()
    );
}

pub fn print_kv_box(title: &str, rows: &[(&str, String)]) {
    let label_width = rows.iter().map(|(k, _)| k.len()).max().unwrap_or(8).max(8);
    let lines: Vec<String> = rows
        .iter()
        .map(|(k, v)| {
            let k_pad = format!("{k:<label_width$}");
            format!("{}  {}", k_pad.dimmed().bold(), v)
        })
        .collect();
    print_boxed_lines(title, &lines);
}

pub fn print_section_divider(title: &str) {
    println!(
        "\n{} {}",
        "▸".bright_cyan().bold(),
        title.bold().bright_white()
    );
}

pub fn print_error(msg: &str) {
    eprintln!("{} {}", format_badge("ERROR", BadgeKind::Error), msg.red());
}

pub fn print_tip(msg: &str) {
    println!(
        "{} {}",
        format_badge("NOTE", BadgeKind::Warn),
        msg.bright_yellow()
    );
}

pub fn print_banner(version: &str) {
    let lines = vec![
        format!(
            "{} {}",
            "All-in-one developer utility suite".bright_white(),
            format!("v{version}").dimmed()
        ),
        "Crypto • Codecs • Time • Text • QR • JSON • System • Network"
            .dimmed()
            .to_string(),
    ];
    print_boxed_lines("OMNITOOL", &lines);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_visible_len_ignores_ansi() {
        let colored = format!("{} {}", "hello".green().bold(), "world".cyan());
        assert_eq!(visible_len(&colored), 11);
    }

    #[test]
    fn test_format_bytes_and_commas() {
        assert_eq!(format_bytes(512), "512 B");
        assert_eq!(format_bytes(2048), "2.0 KB");
        assert_eq!(format_bytes(5 * 1024 * 1024), "5.00 MB");
        assert_eq!(format_number_commas(7680), "7,680");
        assert_eq!(format_number_commas(1234567), "1,234,567");
    }

    #[test]
    fn test_format_duration_human() {
        assert_eq!(format_duration_human(45), "45s");
        assert_eq!(format_duration_human(125), "2m 5s");
        assert_eq!(format_duration_human(90000), "1d 1h");
    }
}
