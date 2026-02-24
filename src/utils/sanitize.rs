use regex::Regex;

pub fn redact(input: &str) -> String {
    let mut text = input.to_string();
    for pattern in [
        r"(?i)(password|token|secret|apikey)\s*=\s*[^\s]+",
        r#"(?i)(password|token|secret|apikey)"?\s*:\s*"[^"]+""#,
    ] {
        if let Ok(re) = Regex::new(pattern) {
            text = re.replace_all(&text, "$1=<redacted>").into_owned();
        }
    }
    text
}

#[cfg(test)]
mod tests {
    use super::redact;

    #[test]
    fn redacts_basic_secret() {
        let out = redact("token=abc123");
        assert!(out.contains("token=<redacted>"));
        assert!(!out.contains("abc123"));
    }
}
