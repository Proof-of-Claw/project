use regex::Regex;

pub struct ContentSanitizer {
    patterns: Vec<Regex>,
}

impl ContentSanitizer {
    pub fn new() -> Self {
        let patterns = vec![
            Regex::new(r"<script[^>]*>.*?</script>").unwrap(),
            Regex::new(r"javascript:").unwrap(),
            Regex::new(r"on\w+\s*=").unwrap(),
        ];
        
        Self { patterns }
    }
    
    pub fn sanitize(&self, content: &str) -> String {
        let mut sanitized = content.to_string();
        
        for pattern in &self.patterns {
            sanitized = pattern.replace_all(&sanitized, "").to_string();
        }
        
        sanitized
    }
}
