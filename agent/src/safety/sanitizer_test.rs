#[cfg(test)]
mod tests {
    use super::super::sanitizer::ContentSanitizer;

    #[test]
    fn test_sanitize_script_tags() {
        let sanitizer = ContentSanitizer::new();
        let malicious = "Hello <script>alert('xss')</script> World";
        let sanitized = sanitizer.sanitize(malicious);
        
        assert!(!sanitized.contains("<script>"));
        assert!(!sanitized.contains("alert"));
    }

    #[test]
    fn test_sanitize_javascript_protocol() {
        let sanitizer = ContentSanitizer::new();
        let malicious = "Click <a href='javascript:alert(1)'>here</a>";
        let sanitized = sanitizer.sanitize(malicious);
        
        assert!(!sanitized.contains("javascript:"));
    }

    #[test]
    fn test_sanitize_event_handlers() {
        let sanitizer = ContentSanitizer::new();
        let malicious = "<div onclick='malicious()'>Click me</div>";
        let sanitized = sanitizer.sanitize(malicious);
        
        assert!(!sanitized.contains("onclick="));
    }

    #[test]
    fn test_sanitize_clean_content() {
        let sanitizer = ContentSanitizer::new();
        let clean = "This is perfectly safe content";
        let sanitized = sanitizer.sanitize(clean);
        
        assert_eq!(sanitized, clean);
    }
}
