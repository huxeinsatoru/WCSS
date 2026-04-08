use crate::ast::StyleSheet;

/// Generate a Source Map v3 JSON string from a stylesheet and original source.
pub fn generate(stylesheet: &StyleSheet, original_source: &str) -> String {
    let mut mappings = Vec::new();

    // Build mapping entries: each rule maps to its source span
    for rule in &stylesheet.rules {
        let line = rule.span.line;
        let column = rule.span.column;
        mappings.push(MappingEntry {
            generated_line: mappings.len() + 1,
            generated_column: 0,
            source_line: line,
            source_column: column,
        });
    }

    // Build Source Map v3 JSON
    let encoded_mappings = encode_mappings(&mappings);

    format!(
        r#"{{"version":3,"file":"output.css","sourceRoot":"","sources":["input.wcss"],"sourcesContent":[{source_content}],"names":[],"mappings":"{encoded_mappings}"}}"#,
        source_content = serde_json_escape(original_source),
        encoded_mappings = encoded_mappings
    )
}

struct MappingEntry {
    generated_line: usize,
    generated_column: usize,
    source_line: usize,
    source_column: usize,
}

/// Encode mappings using VLQ Base64 (simplified).
fn encode_mappings(mappings: &[MappingEntry]) -> String {
    // Simplified: for each mapping, emit a segment per line
    let mut result = String::new();
    let mut prev_gen_col = 0i64;
    let mut prev_src_line = 0i64;
    let mut prev_src_col = 0i64;
    let mut current_gen_line = 1;

    for entry in mappings {
        // Add semicolons for line separators
        while current_gen_line < entry.generated_line {
            result.push(';');
            current_gen_line += 1;
            prev_gen_col = 0;
        }

        let gen_col = entry.generated_column as i64 - prev_gen_col;
        let src_file = 0i64; // Always source file 0
        let src_line = entry.source_line as i64 - 1 - prev_src_line;
        let src_col = entry.source_column as i64 - 1 - prev_src_col;

        result.push_str(&vlq_encode(gen_col));
        result.push_str(&vlq_encode(src_file));
        result.push_str(&vlq_encode(src_line));
        result.push_str(&vlq_encode(src_col));

        prev_gen_col = entry.generated_column as i64;
        prev_src_line = entry.source_line as i64 - 1;
        prev_src_col = entry.source_column as i64 - 1;
    }

    result
}

const BASE64_CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";

fn vlq_encode(value: i64) -> String {
    let mut vlq = if value < 0 {
        ((-value) << 1) | 1
    } else {
        value << 1
    };

    let mut result = String::new();
    loop {
        let mut digit = vlq & 0x1f;
        vlq >>= 5;
        if vlq > 0 {
            digit |= 0x20; // continuation bit
        }
        result.push(BASE64_CHARS[digit as usize] as char);
        if vlq == 0 {
            break;
        }
    }

    result
}

fn serde_json_escape(s: &str) -> String {
    let mut result = String::from("\"");
    for ch in s.chars() {
        match ch {
            '"' => result.push_str("\\\""),
            '\\' => result.push_str("\\\\"),
            '\n' => result.push_str("\\n"),
            '\r' => result.push_str("\\r"),
            '\t' => result.push_str("\\t"),
            c => result.push(c),
        }
    }
    result.push('"');
    result
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ast::*;

    #[test]
    fn test_vlq_encode() {
        assert_eq!(vlq_encode(0), "A");
        assert_eq!(vlq_encode(1), "C");
        assert_eq!(vlq_encode(-1), "D");
    }

    #[test]
    fn test_generate_sourcemap() {
        let stylesheet = StyleSheet {
            rules: vec![Rule {
                selector: Selector {
                    class_name: "btn".to_string(),
                    kind: SelectorKind::Class,
                    combinators: vec![],
                    pseudo_elements: vec![],
                    pseudo_classes: vec![],
                    attributes: vec![],
                    span: Span::new(0, 10, 1, 1),
                },
                selectors: vec![],
                declarations: vec![],
                states: vec![],
                responsive: vec![],
                nested_rules: vec![],
                nested_at_rules: vec![],
                span: Span::new(0, 30, 1, 1),
            }],
            at_rules: vec![],
            span: Span::new(0, 30, 1, 1),
        };

        let sourcemap = generate(&stylesheet, ".btn { color: red; }");
        assert!(sourcemap.contains("\"version\":3"));
        assert!(sourcemap.contains("\"sources\":[\"input.wcss\"]"));
    }
}
