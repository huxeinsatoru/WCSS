use crate::w3c_parser::{W3CToken, W3CTokenType, W3CTokenValue};
use std::collections::HashMap;

/// Generator for HTML documentation from W3C Design Tokens.
pub struct DocsGenerator;

impl DocsGenerator {
    /// Generate HTML documentation from W3C tokens.
    pub fn generate(tokens: &[W3CToken]) -> String {
        let mut output = String::with_capacity(tokens.len() * 500);
        output.push_str(&Self::generate_html_head());
        output.push_str("<body>\n");
        output.push_str(&Self::generate_header());
        output.push_str(&Self::generate_search_bar());
        output.push_str(&Self::generate_token_grid(tokens));
        output.push_str(&Self::generate_footer());
        output.push_str(&Self::generate_search_script());
        output.push_str("</body>\n</html>");
        output
    }

    fn generate_html_head() -> String {
        String::from(r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Design Tokens Documentation</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body { font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif; background: #f8fafc; color: #334155; }
        .container { max-width: 1400px; margin: 0 auto; padding: 2rem; }
        header { background: linear-gradient(135deg, #3b82f6, #8b5cf6); color: white; padding: 3rem 2rem; text-align: center; }
        .token-grid { display: grid; grid-template-columns: repeat(auto-fill, minmax(300px, 1fr)); gap: 1.5rem; }
        .token-card { background: white; border-radius: 12px; padding: 1.5rem; box-shadow: 0 1px 3px rgba(0,0,0,0.1); }
        .token-card.hidden { display: none; }
        .token-path { font-family: monospace; font-size: 0.9rem; color: #3b82f6; margin-bottom: 0.5rem; }
        .token-type { display: inline-block; font-size: 0.75rem; text-transform: uppercase; padding: 0.25rem 0.5rem; background: #f1f5f9; border-radius: 4px; color: #64748b; margin-bottom: 0.75rem; }
        .token-value { font-family: monospace; font-size: 0.85rem; background: #f8fafc; padding: 0.75rem; border-radius: 6px; }
        .color-swatch { width: 100%; height: 80px; border-radius: 8px; margin-bottom: 0.75rem; border: 1px solid #e2e8f0; }
        .search-container { background: white; padding: 1.5rem; border-radius: 12px; box-shadow: 0 1px 3px rgba(0,0,0,0.1); margin-bottom: 2rem; position: sticky; top: 1rem; z-index: 100; }
        .search-input { width: 100%; padding: 1rem 1.5rem; font-size: 1rem; border: 2px solid #e2e8f0; border-radius: 8px; outline: none; }
        .search-input:focus { border-color: #3b82f6; }
    </style>
</head>
"#)
    }

    fn generate_header() -> String {
        String::from(r#"<header><h1>Design Tokens</h1><p>Generated documentation</p></header><div class="container">"#)
    }

    fn generate_search_bar() -> String {
        String::from(r#"<div class="search-container"><input type="text" class="search-input" id="searchInput" placeholder="Search tokens..."></div>"#)
    }

    fn generate_token_grid(tokens: &[W3CToken]) -> String {
        let mut output = String::new();
        let groups = Self::group_tokens_by_category(tokens);
        for (category, group_tokens) in groups {
            output.push_str(&format!("<h2 style='margin: 2rem 0 1rem;'>{}</h2>", category));
            output.push_str("<div class='token-grid'>");
            for token in group_tokens {
                output.push_str(&Self::generate_token_card(token));
            }
            output.push_str("</div>");
        }
        output
    }

    fn group_tokens_by_category(tokens: &[W3CToken]) -> HashMap<String, Vec<&W3CToken>> {
        let mut groups: HashMap<String, Vec<&W3CToken>> = HashMap::new();
        for token in tokens {
            let category = token.path.split('.').next().unwrap_or("uncategorized").to_string();
            groups.entry(category).or_default().push(token);
        }
        groups
    }

    fn generate_token_card(token: &W3CToken) -> String {
        let path = &token.path;
        let type_name = token.token_type.as_ref().map(|t| t.as_str()).unwrap_or("unknown");
        let value = Self::extract_display_value(&token.value);
        let mut card = format!("<div class='token-card'><div class='token-path'>{}</div>", path);
        if token.token_type == Some(W3CTokenType::Color) {
            if let Some(color_css) = Self::color_to_css(&value) {
                card.push_str(&format!("<div class='color-swatch' style='background: {};'></div>", color_css));
            }
        }
        card.push_str(&format!("<span class='token-type'>{}</span>", type_name));
        card.push_str(&format!("<div class='token-value'>{}</div></div>", value));
        card
    }

    fn extract_display_value(value: &W3CTokenValue) -> String {
        match value {
            W3CTokenValue::Literal(s) => s.clone(),
            W3CTokenValue::Reference(r) => format!("{{{{{}}}}}", r),
            W3CTokenValue::Composite(map) => {
                let parts: Vec<String> = map.iter().map(|(k, v)| format!("{}: {}", k, Self::extract_display_value(v))).collect();
                format!("{{ {} }}", parts.join(", "))
            }
        }
    }

    fn color_to_css(value: &str) -> Option<String> {
        let trimmed = value.trim();
        if trimmed.starts_with('#') || trimmed.starts_with("rgb") || trimmed.starts_with("hsl") || trimmed.starts_with("oklch") {
            Some(trimmed.to_string())
        } else {
            Some(trimmed.to_string())
        }
    }

    fn generate_footer() -> String {
        String::from("</div><footer><p>Generated by Euis</p></footer>")
    }

    fn generate_search_script() -> String {
        String::from(r#"<script>
(function() {
    const searchInput = document.getElementById('searchInput');
    const cards = document.querySelectorAll('.token-card');
    searchInput.addEventListener('input', function(e) {
        const query = e.target.value.toLowerCase();
        cards.forEach(card => {
            const text = card.textContent.toLowerCase();
            card.classList.toggle('hidden', !text.includes(query));
        });
    });
})();
</script>"#)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::w3c_parser::{W3CTokenValue, W3CTokenType};

    fn create_token(path: &str, value: &str, token_type: W3CTokenType) -> W3CToken {
        W3CToken {
            path: path.to_string(),
            value: W3CTokenValue::Literal(value.to_string()),
            token_type: Some(token_type),
            description: None,
        }
    }

    #[test]
    fn test_generate_html() {
        let tokens = vec![
            create_token("color.primary", "#3b82f6", W3CTokenType::Color),
            create_token("spacing.base", "16px", W3CTokenType::Dimension),
        ];
        let html = DocsGenerator::generate(&tokens);
        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("color.primary"));
        assert!(html.contains("#3b82f6"));
    }

    #[test]
    fn test_group_tokens() {
        let tokens = vec![
            create_token("color.primary", "#3b82f6", W3CTokenType::Color),
            create_token("color.secondary", "#64748b", W3CTokenType::Color),
            create_token("spacing.base", "16px", W3CTokenType::Dimension),
        ];
        let groups = DocsGenerator::group_tokens_by_category(&tokens);
        assert_eq!(groups.get("color").unwrap().len(), 2);
        assert_eq!(groups.get("spacing").unwrap().len(), 1);
    }
}
