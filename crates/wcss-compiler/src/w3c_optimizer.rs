use crate::w3c_parser::{W3CToken, W3CTokenValue};
use std::collections::HashMap;

/// Optimizer for W3C Design Tokens.
pub struct W3CTokenOptimizer {
    used_classes: Vec<String>,
    enable_tree_shaking: bool,
    enable_deduplication: bool,
}

impl W3CTokenOptimizer {
    /// Create a new optimizer with the given configuration.
    pub fn new(used_classes: Vec<String>, enable_tree_shaking: bool, enable_deduplication: bool) -> Self {
        Self {
            used_classes,
            enable_tree_shaking,
            enable_deduplication,
        }
    }

    /// Optimize the given tokens.
    pub fn optimize(&self, tokens: Vec<W3CToken>) -> Vec<W3CToken> {
        let mut result = tokens;

        if self.enable_deduplication {
            result = self.deduplicate_tokens(result);
        }

        if self.enable_tree_shaking {
            result = self.tree_shake(result);
        }

        result
    }

    /// Remove duplicate tokens with identical values.
    fn deduplicate_tokens(&self, tokens: Vec<W3CToken>) -> Vec<W3CToken> {
        let mut seen: HashMap<String, String> = HashMap::new(); // value_hash -> path
        let mut result = Vec::new();

        for token in tokens {
            let value_key = Self::hash_value(&token.value);
            
            if let Some(existing_path) = seen.get(&value_key) {
                // Skip duplicate, but we could log it
                eprintln!("Warning: Token '{}' has same value as '{}'", token.path, existing_path);
            } else {
                seen.insert(value_key, token.path.clone());
                result.push(token);
            }
        }

        result
    }

    /// Create a simple hash of a token value for deduplication.
    fn hash_value(value: &W3CTokenValue) -> String {
        match value {
            W3CTokenValue::Literal(s) => format!("lit:{}", s),
            W3CTokenValue::Reference(r) => format!("ref:{}", r),
            W3CTokenValue::Composite(map) => {
                let mut parts: Vec<String> = map.iter()
                    .map(|(k, v)| format!("{}={}", k, Self::hash_value(v)))
                    .collect();
                parts.sort();
                format!("comp:{{{}}}", parts.join(","))
            }
        }
    }

    /// Remove unused tokens based on used_classes.
    fn tree_shake(&self, tokens: Vec<W3CToken>) -> Vec<W3CToken> {
        if self.used_classes.is_empty() {
            return tokens;
        }

        let used_set: std::collections::HashSet<String> = self.used_classes.iter().cloned().collect();
        
        tokens.into_iter()
            .filter(|token| {
                // Keep token if it's referenced in used_classes
                // or if it's referenced by another used token
                let token_class = token.path.replace('.', "-");
                used_set.contains(&token_class) || used_set.contains(&token.path)
            })
            .collect()
    }

    /// Detect unused tokens and return warnings.
    pub fn detect_unused_tokens(&self, tokens: &[W3CToken]) -> Vec<String> {
        if self.used_classes.is_empty() {
            return Vec::new();
        }

        let used_set: std::collections::HashSet<String> = self.used_classes.iter().cloned().collect();
        let mut warnings = Vec::new();

        for token in tokens {
            let token_class = token.path.replace('.', "-");
            if !used_set.contains(&token_class) && !used_set.contains(&token.path) {
                warnings.push(format!("Token '{}' is defined but never used", token.path));
            }
        }

        warnings
    }

    /// Sort tokens alphabetically by path for consistent output.
    pub fn sort_tokens(&self, mut tokens: Vec<W3CToken>) -> Vec<W3CToken> {
        tokens.sort_by(|a, b| a.path.cmp(&b.path));
        tokens
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::w3c_parser::{W3CTokenType, W3CTokenValue};

    fn create_token(path: &str, value: &str) -> W3CToken {
        W3CToken {
            path: path.to_string(),
            value: W3CTokenValue::Literal(value.to_string()),
            token_type: Some(W3CTokenType::Color),
            description: None,
        }
    }

    #[test]
    fn test_tree_shaking() {
        let tokens = vec![
            create_token("color.primary", "#3b82f6"),
            create_token("color.secondary", "#64748b"),
            create_token("spacing.base", "16px"),
        ];

        let optimizer = W3CTokenOptimizer::new(
            vec!["color-primary".to_string()],
            true,
            false,
        );

        let optimized = optimizer.optimize(tokens);
        assert_eq!(optimized.len(), 1);
        assert_eq!(optimized[0].path, "color.primary");
    }

    #[test]
    fn test_deduplication() {
        let tokens = vec![
            create_token("color.primary", "#3b82f6"),
            create_token("color.secondary", "#3b82f6"), // Same value
            create_token("spacing.base", "16px"),
        ];

        let optimizer = W3CTokenOptimizer::new(
            Vec::new(),
            false,
            true,
        );

        let optimized = optimizer.optimize(tokens);
        assert_eq!(optimized.len(), 2); // One duplicate removed
    }

    #[test]
    fn test_detect_unused() {
        let tokens = vec![
            create_token("color.primary", "#3b82f6"),
            create_token("color.unused", "#64748b"),
        ];

        let optimizer = W3CTokenOptimizer::new(
            vec!["color-primary".to_string()],
            true,
            false,
        );

        let warnings = optimizer.detect_unused_tokens(&tokens);
        assert_eq!(warnings.len(), 1);
        assert!(warnings[0].contains("color.unused"));
    }

    #[test]
    fn test_sort_tokens() {
        let tokens = vec![
            create_token("z.token", "#000"),
            create_token("a.token", "#fff"),
            create_token("m.token", "#ccc"),
        ];

        let optimizer = W3CTokenOptimizer::new(Vec::new(), false, false);
        let sorted = optimizer.sort_tokens(tokens);
        
        assert_eq!(sorted[0].path, "a.token");
        assert_eq!(sorted[1].path, "m.token");
        assert_eq!(sorted[2].path, "z.token");
    }
}
