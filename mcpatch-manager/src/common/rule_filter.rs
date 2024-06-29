//! 规则过滤

use regex::Regex;

/// 代表单个过滤规则
pub struct Rule {
    /// 原始字符串，主要是调试输出用途
    pub raw: String,
    
    /// 编译后的规则对象
    pub pattern: Regex, 
}

/// 代表一组过滤规则
pub struct RuleFilter {
    pub filters: Vec<Rule>
}

impl RuleFilter {
    /// 创建一个空的规则过滤器，空的规则过滤器会通过任何测试的文件
    pub fn new() -> Self {
        Self::from_rules([""; 0].iter())
    }

    /// 从一个字符串引用迭代器创建规则过滤器
    pub fn from_rules<'a>(rules: impl Iterator<Item = impl AsRef<str>>) -> RuleFilter {
        let mut regexes_compiled = Vec::<Rule>::new();

        for pattern in rules {
            let raw_pattern = pattern.as_ref();
            let pattern = raw_pattern;
            regexes_compiled.push(Rule {
                raw: raw_pattern.to_owned(),
                pattern: Regex::new(&pattern).unwrap(), 
            });
        }

        RuleFilter { filters: regexes_compiled }
    }

    /// 测试一段字符串能否通过任何一个规则测试，如果不能通过或者规则列表为空，返回`default`。
    pub fn test_any(&self, text: &str, default: bool) -> bool {
        if self.filters.is_empty() {
            return default;
        }

        self.filters.iter().any(|filter| filter.pattern.is_match(text))
    }
    
    /// 测试一段字符串能否通过所有的规则测试，如果不能通过或者规则列表为空，返回`default`。
    pub fn test_all(&self, text: &str, default: bool) -> bool {
        if self.filters.is_empty() {
            return default;
        }

        self.filters.iter().all(|filter| filter.pattern.is_match(text))
    }
}