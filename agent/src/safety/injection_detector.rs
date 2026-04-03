use regex::Regex;

pub struct InjectionDetector {
    patterns: Vec<Regex>,
}

impl InjectionDetector {
    pub fn new() -> Self {
        let patterns = vec![
            Regex::new(r"(?i)ignore\s+previous\s+instructions").unwrap(),
            Regex::new(r"(?i)disregard\s+all\s+prior").unwrap(),
            Regex::new(r"(?i)system\s*:\s*you\s+are").unwrap(),
        ];
        
        Self { patterns }
    }
    
    pub fn detect(&self, content: &str) -> bool {
        self.patterns.iter().any(|pattern| pattern.is_match(content))
    }
}
