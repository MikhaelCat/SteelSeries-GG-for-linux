// Utility functions shared across the codebase

/// Normalize RGB values to 0-100 range
pub fn normalize_brightness(r: u8, g: u8, b: u8) -> u8 {
    // Average brightness
    ((r as u32 + g as u32 + b as u32) / 3 * 100 / 255) as u8
}

/// Convert hex color string to [u8; 3] RGB tuple
pub fn hex_to_rgb(hex: &str) -> Option<[u8; 3]> {
    if !hex.starts_with('#') || hex.len() != 7 {
        return None;
    }

    let r = u8::from_str_radix(&hex[1..3], 16).ok()?;
    let g = u8::from_str_radix(&hex[3..5], 16).ok()?;
    let b = u8::from_str_radix(&hex[5..7], 16).ok()?;

    Some([r, g, b])
}

/// Parse RGB tuple string "(255, 0, 0)" to [u8; 3]
pub fn tuple_to_rgb(s: &str) -> Option<[u8; 3]> {
    s.trim_matches(|c| c == '(' || c == ')')
        .split(',')
        .map(|s| s.trim().parse::<u8>().ok())
        .collect::<Option<Vec<u8>>>()
        .and_then(|v| {
            if v.len() == 3 {
                Some([v[0], v[1], v[2]])
            } else {
                None
            }
        })
}

/// Get program name from Cargo.toml or fallback
pub fn get_program_name() -> &'static str {
    env!("CARGO_PKG_NAME")
}

/// Get version info (for CLI --version flag)
pub fn get_version() -> String {
    format!(
        "{} {}{}",
        env!("CARGO_PKG_VERSION"),
        std::env::var("GIT_COMMIT").unwrap_or_else(|_| "unknown".to_string()),
        if cfg!(debug_assertions) { "-debug" } else { "" }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_normalize_brightness() {
        assert_eq!(normalize_brightness(255, 255, 255), 100);
        assert_eq!(normalize_brightness(128, 128, 128), 50);
        assert_eq!(normalize_brightness(0, 0, 0), 0);
    }

    #[test]
    fn test_hex_to_rgb() {
        assert_eq!(hex_to_rgb("#FF0000"), Some([255, 0, 0]));
        assert_eq!(hex_to_rgb("#00FF00"), Some([0, 255, 0]));
        assert_eq!(hex_to_rgb("#0000FF"), Some([0, 0, 255]));
        assert_eq!(hex_to_rgb("#GGG"), None);
        assert_eq!(hex_to_rgb("not-hex"), None);
    }

    #[test]
    fn test_tuple_to_rgb() {
        assert_eq!(tuple_to_rgb("(255, 0, 0)"), Some([255, 0, 0]));
        assert_eq!(tuple_to_rgb("(0, 255, 0)"), Some([0, 255, 0]));
        assert_eq!(tuple_to_rgb("(invalid)"), None);
    }
}
