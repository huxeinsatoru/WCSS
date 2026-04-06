use crate::ast::*;
use crate::config::CompilerConfig;
use crate::error::CompilerError;

/// Validate a stylesheet against the compiler configuration.
/// Returns a list of validation errors.
pub fn validate(stylesheet: &StyleSheet, config: &CompilerConfig) -> Vec<CompilerError> {
    let mut errors = Vec::new();

    for rule in &stylesheet.rules {
        validate_rule(rule, config, &mut errors);
    }

    errors
}

fn validate_rule(rule: &Rule, config: &CompilerConfig, errors: &mut Vec<CompilerError>) {
    for decl in &rule.declarations {
        validate_declaration(decl, config, errors);
    }
    for state in &rule.states {
        for decl in &state.declarations {
            validate_declaration(decl, config, errors);
        }
    }
    for responsive in &rule.responsive {
        validate_breakpoint_ref(&responsive.breakpoint, &responsive.span, config, errors);
        for decl in &responsive.declarations {
            validate_declaration(decl, config, errors);
        }
    }
}

fn validate_declaration(decl: &Declaration, config: &CompilerConfig, errors: &mut Vec<CompilerError>) {
    validate_value(&decl.value, config, errors);
}

fn validate_value(value: &Value, config: &CompilerConfig, errors: &mut Vec<CompilerError>) {
    match value {
        Value::Token(token_ref) => {
            if config.tokens.get(&token_ref.category, &token_ref.name).is_none() {
                let available = get_available_tokens(&config.tokens, &token_ref.category);
                let suggestion = if available.is_empty() {
                    None
                } else {
                    Some(format!("Available tokens: {}", available.join(", ")))
                };
                errors.push(CompilerError::token_not_found(
                    &token_ref.name,
                    token_ref.span.clone(),
                    suggestion,
                ));
            }
        }
        Value::List(values) => {
            for v in values {
                validate_value(v, config, errors);
            }
        }
        _ => {}
    }
}

fn validate_breakpoint_ref(
    breakpoint: &str,
    span: &Span,
    config: &CompilerConfig,
    errors: &mut Vec<CompilerError>,
) {
    if !config.tokens.breakpoints.contains_key(breakpoint) {
        let available: Vec<&String> = config.tokens.breakpoints.keys().collect();
        let suggestion = if available.is_empty() {
            Some("No breakpoints defined. Add breakpoints to your wcss.config".to_string())
        } else {
            Some(format!(
                "Available breakpoints: {}",
                available.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", ")
            ))
        };
        errors.push(
            CompilerError::validation(
                format!("Undefined breakpoint '{breakpoint}'"),
                span.clone(),
            )
            .with_suggestion(suggestion.unwrap()),
        );
    }
}

fn get_available_tokens(tokens: &crate::config::DesignTokens, category: &TokenCategory) -> Vec<String> {
    match category {
        TokenCategory::Colors => tokens.colors.keys().cloned().collect(),
        TokenCategory::Spacing => tokens.spacing.keys().cloned().collect(),
        TokenCategory::Typography => tokens.typography.keys().cloned().collect(),
        TokenCategory::Breakpoints => tokens.breakpoints.keys().cloned().collect(),
        TokenCategory::Animation => Vec::new(), // Not yet implemented
        TokenCategory::Custom => tokens.spacing.keys().cloned().collect(), // Fallback
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{DesignTokens, TokenValue};
    use std::collections::HashMap;

    fn config_with_tokens() -> CompilerConfig {
        let mut colors = HashMap::new();
        colors.insert("primary".to_string(), TokenValue::Literal("#3b82f6".to_string()));
        CompilerConfig {
            tokens: DesignTokens { colors, ..Default::default() },
            ..Default::default()
        }
    }

    #[test]
    fn test_validate_defined_token() {
        let config = config_with_tokens();
        let stylesheet = StyleSheet {
            rules: vec![Rule {
                selector: Selector {
                    class_name: "btn".to_string(),
                    combinators: vec![],
                    pseudo_elements: vec![],
                    span: Span::empty(),
                },
                declarations: vec![Declaration {
                    property: Property::Standard("color".to_string()),
                    value: Value::Token(TokenRef {
                        category: TokenCategory::Colors,
                        name: "primary".to_string(),
                        span: Span::empty(),
                    }),
                    important: false,
                    span: Span::empty(),
                }],
                states: vec![],
                responsive: vec![],
                span: Span::empty(),
            }],
            span: Span::empty(),
        };
        let errors = validate(&stylesheet, &config);
        assert!(errors.is_empty());
    }

    #[test]
    fn test_validate_undefined_token() {
        let config = config_with_tokens();
        let stylesheet = StyleSheet {
            rules: vec![Rule {
                selector: Selector {
                    class_name: "btn".to_string(),
                    combinators: vec![],
                    pseudo_elements: vec![],
                    span: Span::empty(),
                },
                declarations: vec![Declaration {
                    property: Property::Standard("color".to_string()),
                    value: Value::Token(TokenRef {
                        category: TokenCategory::Colors,
                        name: "danger".to_string(),
                        span: Span::empty(),
                    }),
                    important: false,
                    span: Span::empty(),
                }],
                states: vec![],
                responsive: vec![],
                span: Span::empty(),
            }],
            span: Span::empty(),
        };
        let errors = validate(&stylesheet, &config);
        assert_eq!(errors.len(), 1);
        assert!(errors[0].message.contains("danger"));
    }
}
