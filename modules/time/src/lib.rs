use chrono::Local;

/// Default pattern used when formatting the current local time.
pub const DEFAULT_PATTERN: &str = "%a %d %b %H:%M";

/// Returns the current local time formatted with the given chrono pattern.
pub fn format_local(pattern: &str) -> String {
    Local::now().format(pattern).to_string()
}

/// Returns the current local time using the default pattern.
pub fn format_local_default() -> String {
    format_local(DEFAULT_PATTERN)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn formats_with_custom_pattern() {
        let out = format_local("%Y");
        assert!(out.chars().all(|c| c.is_ascii_digit()));
        assert!(out.len() == 4);
    }

    #[test]
    fn formats_with_default_pattern() {
        let out = format_local_default();
        assert!(!out.is_empty());
    }
}
