use qrcode::{Color, QrCode};

pub fn generate_terminal_qr(content: &str) -> Result<String, String> {
    let code = QrCode::new(content.as_bytes()).map_err(|e| format!("QR encoding failed: {e}"))?;

    let width = code.width();
    let modules = code.into_colors();

    // Use two spaces or block characters for square aspect ratio in terminal fonts
    let mut output = String::new();
    let border = 2;

    // Top quiet zone
    for _ in 0..border {
        output.push_str(&"██".repeat(width + border * 2));
        output.push('\n');
    }

    for y in 0..width {
        output.push_str(&"██".repeat(border));
        for x in 0..width {
            let color = modules[y * width + x];
            match color {
                Color::Light => output.push_str("██"),
                Color::Dark => output.push_str("  "),
            }
        }
        output.push_str(&"██".repeat(border));
        output.push('\n');
    }

    // Bottom quiet zone
    for _ in 0..border {
        output.push_str(&"██".repeat(width + border * 2));
        output.push('\n');
    }

    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_qr_generation() {
        let result = generate_terminal_qr("https://github.com");
        assert!(result.is_ok());
        let qr = result.unwrap();
        assert!(qr.contains("██"));
    }
}
