use qrcode::{Color, EcLevel, QrCode};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum QrEcLevel {
    L,
    M,
    Q,
    H,
}

impl QrEcLevel {
    pub fn to_qrcode_ec(self) -> EcLevel {
        match self {
            QrEcLevel::L => EcLevel::L,
            QrEcLevel::M => EcLevel::M,
            QrEcLevel::Q => EcLevel::Q,
            QrEcLevel::H => EcLevel::H,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            QrEcLevel::L => "L (7%)",
            QrEcLevel::M => "M (15%)",
            QrEcLevel::Q => "Q (25%)",
            QrEcLevel::H => "H (30%)",
        }
    }
}

pub struct QrRenderOptions {
    pub ec_level: QrEcLevel,
    pub compact: bool,
    pub invert: bool,
    pub use_color: bool,
    pub no_universal_links: bool,
}

impl Default for QrRenderOptions {
    fn default() -> Self {
        Self {
            ec_level: QrEcLevel::M,
            compact: true,
            invert: false,
            use_color: true,
            no_universal_links: false,
        }
    }
}

pub struct QrRenderOutput {
    pub terminal_art: String,
    pub module_width: usize,
    pub version_number: usize,
    pub effective_content: String,
    pub advisory: Option<String>,
}

pub fn prepare_qr_content(
    content: &str,
    no_universal_links: bool,
) -> Result<(String, Option<String>), String> {
    if content.is_empty() {
        return Err("QR content cannot be empty".to_string());
    }

    let mut effective = content.to_string();
    let mut advisory = None;

    if no_universal_links && (effective.starts_with("https://") || effective.starts_with("http://"))
    {
        if !effective.contains('#') {
            effective.push_str("#no_universal_links");
        } else if !effective.contains("#no_universal_links") {
            advisory = Some(
                "URL already has a '#' fragment, so #no_universal_links was not appended."
                    .to_string(),
            );
        }
    }

    if advisory.is_none()
        && (effective.starts_with("https://claude.ai/share/")
            || effective.starts_with("http://claude.ai/share/"))
        && !effective.contains("#no_universal_links")
    {
        advisory = Some(
            "Mobile phones with the Claude app installed may intercept claude.ai/share links and open the app home screen instead of the shared chat. Re-run with --no-universal-links to open in Safari on iOS, or share via a browser redirect link for Android."
                .to_string(),
        );
    }

    Ok((effective, advisory))
}

fn build_padded_grid(code: &QrCode, border: usize) -> (usize, Vec<bool>) {
    let width = code.width();
    let modules = code.to_colors();
    let total = width + border * 2;
    let mut grid = vec![false; total * total];

    for y in 0..width {
        for x in 0..width {
            let is_dark = modules[y * width + x] == Color::Dark;
            grid[(y + border) * total + (x + border)] = is_dark;
        }
    }

    (total, grid)
}

fn render_compact_art(total: usize, grid: &[bool], invert: bool, use_color: bool) -> String {
    let mut output = String::new();

    for y in (0..total).step_by(2) {
        if use_color {
            if invert {
                output.push_str("\x1b[97;40m");
            } else {
                output.push_str("\x1b[30;107m");
            }
        }

        for x in 0..total {
            let top_dark = grid[y * total + x];
            let bot_dark = if y + 1 < total {
                grid[(y + 1) * total + x]
            } else {
                false
            };

            let (fg_top, fg_bot) = if !use_color && !invert {
                // Plain text on dark terminals commonly expects light foreground blocks
                (!top_dark, !bot_dark)
            } else {
                (top_dark, bot_dark)
            };

            let ch = match (fg_top, fg_bot) {
                (false, false) => ' ',
                (false, true) => '\u{2584}', // ▄
                (true, false) => '\u{2580}', // ▀
                (true, true) => '\u{2588}',  // █
            };
            output.push(ch);
        }

        if use_color {
            output.push_str("\x1b[0m");
        }
        output.push('\n');
    }

    output
}

fn render_full_art(total: usize, grid: &[bool], invert: bool, use_color: bool) -> String {
    let mut output = String::new();

    for y in 0..total {
        if use_color {
            if invert {
                output.push_str("\x1b[97;40m");
            } else {
                output.push_str("\x1b[30;107m");
            }
        }

        for x in 0..total {
            let is_dark = grid[y * total + x];
            let draw_block = if use_color || invert {
                is_dark
            } else {
                !is_dark
            };

            if draw_block {
                output.push_str("██");
            } else {
                output.push_str("  ");
            }
        }

        if use_color {
            output.push_str("\x1b[0m");
        }
        output.push('\n');
    }

    output
}

pub fn render_qr_code(content: &str, opts: &QrRenderOptions) -> Result<QrRenderOutput, String> {
    let (effective_content, advisory) = prepare_qr_content(content, opts.no_universal_links)?;
    let code = QrCode::with_error_correction_level(
        effective_content.as_bytes(),
        opts.ec_level.to_qrcode_ec(),
    )
    .map_err(|e| format!("QR encoding failed: {e}"))?;

    let module_width = code.width();
    let version_number = module_width.saturating_sub(17) / 4;
    let border = 2;
    let (total, grid) = build_padded_grid(&code, border);

    let terminal_art = if opts.compact {
        render_compact_art(total, &grid, opts.invert, opts.use_color)
    } else {
        render_full_art(total, &grid, opts.invert, opts.use_color)
    };

    Ok(QrRenderOutput {
        terminal_art,
        module_width,
        version_number,
        effective_content,
        advisory,
    })
}

pub fn generate_qr_svg(content: &str, ec_level: QrEcLevel, invert: bool) -> Result<String, String> {
    if content.is_empty() {
        return Err("QR content cannot be empty".to_string());
    }
    let code = QrCode::with_error_correction_level(content.as_bytes(), ec_level.to_qrcode_ec())
        .map_err(|e| format!("QR encoding failed: {e}"))?;

    let width = code.width();
    let border = 4;
    let total = width + border * 2;
    let modules = code.into_colors();

    let (bg, fg) = if invert {
        ("#111111", "#ffffff")
    } else {
        ("#ffffff", "#000000")
    };

    let mut path_data = String::new();
    for y in 0..width {
        for x in 0..width {
            if modules[y * width + x] == Color::Dark {
                let px = x + border;
                let py = y + border;
                path_data.push_str(&format!("M{px},{py}h1v1h-1z"));
            }
        }
    }

    Ok(format!(
        "<svg xmlns=\"http://www.w3.org/2000/svg\" viewBox=\"0 0 {total} {total}\" shape-rendering=\"crispEdges\">\n  <rect width=\"100%\" height=\"100%\" fill=\"{bg}\"/>\n  <path fill=\"{fg}\" d=\"{path_data}\"/>\n</svg>\n"
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qr_generation() {
        let opts = QrRenderOptions {
            compact: false,
            use_color: false,
            ..Default::default()
        };
        let result = render_qr_code("https://github.com", &opts);
        assert!(result.is_ok());
        let qr = result.unwrap().terminal_art;
        assert!(qr.contains("██"));
    }

    #[test]
    fn test_qr_compact_and_empty_rejection() {
        assert!(render_qr_code("", &QrRenderOptions::default()).is_err());

        let out = render_qr_code(
            "https://github.com",
            &QrRenderOptions {
                compact: true,
                use_color: true,
                ..Default::default()
            },
        )
        .expect("compact qr");

        assert!(out.terminal_art.contains("\x1b[30;107m"));
        assert!(out.terminal_art.contains("\x1b[0m"));
        let line_count = out.terminal_art.lines().count();
        assert_eq!(line_count, (out.module_width + 4).div_ceil(2));
    }

    #[test]
    fn test_qr_universal_link_advisory_and_flag() {
        let raw = "https://claude.ai/share/90a180f5-a7c3-4cea-8829-d1999c20abb9";
        let out = render_qr_code(raw, &QrRenderOptions::default()).expect("render");
        assert_eq!(out.effective_content, raw);
        assert!(out.advisory.is_some());

        let fixed = render_qr_code(
            raw,
            &QrRenderOptions {
                no_universal_links: true,
                ..Default::default()
            },
        )
        .expect("render with flag");
        assert_eq!(
            fixed.effective_content,
            "https://claude.ai/share/90a180f5-a7c3-4cea-8829-d1999c20abb9#no_universal_links"
        );
        assert!(fixed.advisory.is_none());
    }

    #[test]
    fn test_qr_svg_export() {
        let svg = generate_qr_svg("https://github.com", QrEcLevel::M, false).expect("svg");
        assert!(svg.starts_with("<svg "));
        assert!(svg.contains("<path fill=\"#000000\""));
        assert!(svg.contains("</svg>"));
    }
}
