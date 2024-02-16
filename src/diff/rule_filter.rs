use regex::Regex;

pub struct Rule {
    pub raw: String,
    pub pattern: Regex, 
    pub reversed: bool,
}

pub struct RuleFilter {
    pub filters: Vec<Rule>
}

impl RuleFilter {
    pub fn new() -> Self {
        Self::from_rules([""; 0].iter())
    }

    pub fn from_rules<'a>(rules: impl Iterator<Item = impl AsRef<str>>) -> RuleFilter {
        let mut regexes_compiled = Vec::<Rule>::new();

        for pattern in rules {
            let raw_pattern = pattern.as_ref();
            let mut pattern = raw_pattern;
            let reversed = pattern.starts_with("!");
            if reversed {
                pattern = &pattern[1..];
            }
            regexes_compiled.push(Rule {
                raw: raw_pattern.to_owned(),
                pattern: Regex::new(&pattern).unwrap(), 
                reversed,
            });
        }

        RuleFilter { filters: regexes_compiled }
    }

    pub fn test_any(&self, text: &str, default: bool) -> bool {
        if self.filters.is_empty() {
            return default;
        }

        self.filters.iter().any(|filter| filter.pattern.is_match(text) != filter.reversed)
    }
    
    pub fn test_all(&self, text: &str, default: bool) -> bool {
        if self.filters.is_empty() {
            return default;
        }

        self.filters.iter().all(|filter| filter.pattern.is_match(text) != filter.reversed)
    }
}