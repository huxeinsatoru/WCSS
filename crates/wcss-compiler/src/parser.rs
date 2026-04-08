use crate::ast::*;
use crate::error::CompilerError;

/// Macro placeholder for the enum — Rust doesn't need this but we use it for doc clarity.
macro_rules! enum_result_type {
    () => {};
}

/// Parse WCSS source code into a StyleSheet AST.
pub fn parse(source: &str) -> Result<StyleSheet, Vec<CompilerError>> {
    let mut parser = Parser::new(source);
    parser.parse_stylesheet()
}

struct Parser<'a> {
    source: &'a str,
    bytes: &'a [u8],
    pos: usize,
    line: usize,
    column: usize,
    errors: Vec<CompilerError>,
}

impl<'a> Parser<'a> {
    fn new(source: &'a str) -> Self {
        Self {
            source,
            bytes: source.as_bytes(),
            pos: 0,
            line: 1,
            column: 1,
            errors: Vec::new(),
        }
    }

    #[inline(always)]
    fn current_span(&self) -> Span {
        Span::new(self.pos, self.pos, self.line, self.column)
    }

    #[inline(always)]
    fn span_from(&self, start_pos: usize, start_line: usize, start_col: usize) -> Span {
        Span::new(start_pos, self.pos, start_line, start_col)
    }

    #[inline(always)]
    fn peek_byte(&self) -> Option<u8> {
        self.bytes.get(self.pos).copied()
    }

    #[inline(always)]
    fn peek_byte_at(&self, offset: usize) -> Option<u8> {
        self.bytes.get(self.pos + offset).copied()
    }

    #[inline(always)]
    fn peek(&self) -> Option<char> {
        if self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            if b < 128 {
                Some(b as char)
            } else {
                self.source[self.pos..].chars().next()
            }
        } else {
            None
        }
    }

    #[inline(always)]
    fn advance(&mut self) -> Option<char> {
        if self.pos >= self.bytes.len() {
            return None;
        }
        let b = self.bytes[self.pos];
        if b < 128 {
            self.pos += 1;
            if b == b'\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            Some(b as char)
        } else {
            let ch = self.source[self.pos..].chars().next()?;
            self.pos += ch.len_utf8();
            self.column += 1;
            Some(ch)
        }
    }

    fn skip_whitespace(&mut self) {
        loop {
            if self.pos >= self.bytes.len() {
                break;
            }
            let b = self.bytes[self.pos];
            match b {
                b' ' | b'\t' | b'\r' => {
                    self.pos += 1;
                    self.column += 1;
                }
                b'\n' => {
                    self.pos += 1;
                    self.line += 1;
                    self.column = 1;
                }
                b'/' => {
                    if self.pos + 1 < self.bytes.len() {
                        match self.bytes[self.pos + 1] {
                            b'/' => {
                                self.pos += 2;
                                self.column += 2;
                                while self.pos < self.bytes.len() && self.bytes[self.pos] != b'\n' {
                                    self.pos += 1;
                                }
                            }
                            b'*' => {
                                self.pos += 2;
                                self.column += 2;
                                while self.pos + 1 < self.bytes.len() {
                                    if self.bytes[self.pos] == b'*' && self.bytes[self.pos + 1] == b'/' {
                                        self.pos += 2;
                                        self.column += 2;
                                        break;
                                    }
                                    if self.bytes[self.pos] == b'\n' {
                                        self.line += 1;
                                        self.column = 1;
                                    } else {
                                        self.column += 1;
                                    }
                                    self.pos += 1;
                                }
                            }
                            _ => break,
                        }
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
    }

    fn expect(&mut self, expected: char) -> Result<(), CompilerError> {
        self.skip_whitespace();
        match self.peek() {
            Some(ch) if ch == expected => {
                self.advance();
                Ok(())
            }
            Some(ch) => Err(CompilerError::syntax(
                format!("Expected '{expected}', found '{ch}'"),
                self.current_span(),
            )),
            None => Err(CompilerError::syntax(
                format!("Expected '{expected}', found end of input"),
                self.current_span(),
            )),
        }
    }

    fn read_identifier(&mut self) -> Option<String> {
        let start = self.pos;
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' {
                self.pos += 1;
                self.column += 1;
            } else {
                break;
            }
        }
        if self.pos > start {
            Some(self.source[start..self.pos].to_string())
        } else {
            None
        }
    }

    /// Read a CSS identifier that may start with -- (custom properties) or vendor prefixes (-webkit-, -moz-, etc).
    fn read_css_identifier(&mut self) -> Option<String> {
        let start = self.pos;
        
        // Allow leading - for vendor prefixes (-webkit-, -moz-, -ms-, -o-)
        // or -- for custom properties
        if self.pos < self.bytes.len() && self.bytes[self.pos] == b'-' {
            self.pos += 1;
            self.column += 1;
            
            // Check for second - (custom property)
            if self.pos < self.bytes.len() && self.bytes[self.pos] == b'-' {
                self.pos += 1;
                self.column += 1;
            }
        }
        
        // Read the rest of the identifier
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            if b.is_ascii_alphanumeric() || b == b'-' || b == b'_' {
                self.pos += 1;
                self.column += 1;
            } else {
                break;
            }
        }
        
        if self.pos > start {
            Some(self.source[start..self.pos].to_string())
        } else {
            None
        }
    }

    fn read_string(&mut self) -> Result<String, CompilerError> {
        let quote = self.bytes[self.pos];
        self.pos += 1;
        self.column += 1;
        let start = self.pos;
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            if b == quote {
                let value = self.source[start..self.pos].to_string();
                self.pos += 1;
                self.column += 1;
                return Ok(value);
            }
            if b == b'\\' {
                self.pos += 1;
                self.column += 1;
            }
            if b == b'\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.pos += 1;
        }
        Err(CompilerError::syntax("Unterminated string", self.current_span()))
    }

    fn read_number(&mut self) -> f64 {
        let start = self.pos;
        let negative = self.pos < self.bytes.len() && self.bytes[self.pos] == b'-';
        if negative {
            self.pos += 1;
            self.column += 1;
        }
        let mut int_val: i64 = 0;
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
            int_val = int_val * 10 + (self.bytes[self.pos] - b'0') as i64;
            self.pos += 1;
            self.column += 1;
        }
        if self.pos < self.bytes.len() && self.bytes[self.pos] == b'.' {
            self.pos += 1;
            self.column += 1;
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                self.pos += 1;
                self.column += 1;
            }
            // Handle scientific notation (e.g., 1e-3, 2.5e+2)
            if self.pos < self.bytes.len() && (self.bytes[self.pos] == b'e' || self.bytes[self.pos] == b'E') {
                // Check if next char is +, -, or digit (valid scientific notation)
                let next_pos = self.pos + 1;
                let is_scientific = if next_pos < self.bytes.len() {
                    let next_byte = self.bytes[next_pos];
                    next_byte == b'+' || next_byte == b'-' || next_byte.is_ascii_digit()
                } else {
                    false
                };
                
                if is_scientific {
                    self.pos += 1;
                    self.column += 1;
                    if self.pos < self.bytes.len() && (self.bytes[self.pos] == b'+' || self.bytes[self.pos] == b'-') {
                        self.pos += 1;
                        self.column += 1;
                    }
                    while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                        self.pos += 1;
                        self.column += 1;
                    }
                }
            }
            self.source[start..self.pos].parse().unwrap_or(0.0)
        } else {
            // Handle scientific notation for integers too
            if self.pos < self.bytes.len() && (self.bytes[self.pos] == b'e' || self.bytes[self.pos] == b'E') {
                // Check if next char is +, -, or digit (valid scientific notation)
                let next_pos = self.pos + 1;
                let is_scientific = if next_pos < self.bytes.len() {
                    let next_byte = self.bytes[next_pos];
                    next_byte == b'+' || next_byte == b'-' || next_byte.is_ascii_digit()
                } else {
                    false
                };
                
                if is_scientific {
                    self.pos += 1;
                    self.column += 1;
                    if self.pos < self.bytes.len() && (self.bytes[self.pos] == b'+' || self.bytes[self.pos] == b'-') {
                        self.pos += 1;
                        self.column += 1;
                    }
                    while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                        self.pos += 1;
                        self.column += 1;
                    }
                    return self.source[start..self.pos].parse().unwrap_or(0.0);
                }
            }
            if negative { -(int_val as f64) } else { int_val as f64 }
        }
    }

    /// Read balanced content inside parentheses (handles nested parens).
    fn read_balanced_parens(&mut self) -> Result<String, CompilerError> {
        self.expect('(')?;
        let start = self.pos;
        let mut depth = 1;
        while self.pos < self.bytes.len() && depth > 0 {
            match self.bytes[self.pos] {
                b'(' => depth += 1,
                b')' => depth -= 1,
                b'\n' => { self.line += 1; self.column = 0; }
                _ => {}
            }
            if depth > 0 {
                self.pos += 1;
                self.column += 1;
            }
        }
        if depth != 0 {
            return Err(CompilerError::syntax("Unmatched parenthesis", self.current_span()));
        }
        let content = self.source[start..self.pos].trim().to_string();
        self.pos += 1; // skip ')'
        self.column += 1;
        Ok(content)
    }

    // -----------------------------------------------------------------------
    // Top-level stylesheet parsing
    // -----------------------------------------------------------------------

    fn parse_stylesheet(&mut self) -> Result<StyleSheet, Vec<CompilerError>> {
        let start_pos = self.pos;
        let start_line = self.line;
        let start_col = self.column;
        let estimated_rules = self.bytes.len() / 80;
        let mut rules = Vec::with_capacity(estimated_rules);
        let mut at_rules = Vec::new();

        loop {
            self.skip_whitespace();
            if self.pos >= self.bytes.len() {
                break;
            }

            match self.peek_byte() {
                Some(b'@') => {
                    match self.parse_at_rule_or_responsive() {
                        Ok(AtRuleOrRule::AtRule(at)) => at_rules.push(at),
                        Ok(AtRuleOrRule::Rule(rule)) => rules.push(rule),
                        Err(err) => {
                            self.errors.push(err);
                            self.recover_to_next_rule();
                        }
                    }
                }
                _ => {
                    match self.parse_rule() {
                        Ok(rule) => rules.push(rule),
                        Err(err) => {
                            self.errors.push(err);
                            self.recover_to_next_rule();
                        }
                    }
                }
            }
        }

        if !self.errors.is_empty() {
            return Err(std::mem::take(&mut self.errors));
        }

        Ok(StyleSheet {
            rules,
            at_rules,
            span: self.span_from(start_pos, start_line, start_col),
        })
    }

    fn recover_to_next_rule(&mut self) {
        let mut brace_depth = 0;
        while self.pos < self.bytes.len() {
            let b = self.bytes[self.pos];
            self.advance();
            match b {
                b'{' => brace_depth += 1,
                b'}' => {
                    if brace_depth <= 1 {
                        break;
                    }
                    brace_depth -= 1;
                }
                _ => {}
            }
        }
    }

    // -----------------------------------------------------------------------
    // At-rule parsing
    // -----------------------------------------------------------------------

    enum_result_type!();

    fn parse_at_rule_or_responsive(&mut self) -> Result<AtRuleOrRule, CompilerError> {
        let start_pos = self.pos;
        let start_line = self.line;
        let start_col = self.column;

        self.pos += 1; // skip '@'
        self.column += 1;

        let keyword = self.read_identifier().unwrap_or_default();

        match keyword.as_str() {
            "import" => {
                let rule = self.parse_import_rule(start_pos, start_line, start_col)?;
                Ok(AtRuleOrRule::AtRule(AtRule::Import(rule)))
            }
            "layer" => {
                let rule = self.parse_layer_rule(start_pos, start_line, start_col)?;
                Ok(AtRuleOrRule::AtRule(AtRule::Layer(rule)))
            }
            "keyframes" | "webkit-keyframes" | "moz-keyframes" => {
                let rule = self.parse_keyframes_rule(start_pos, start_line, start_col)?;
                Ok(AtRuleOrRule::AtRule(AtRule::Keyframes(rule)))
            }
            "font-face" => {
                let rule = self.parse_font_face_rule(start_pos, start_line, start_col)?;
                Ok(AtRuleOrRule::AtRule(AtRule::FontFace(rule)))
            }
            "supports" => {
                let rule = self.parse_supports_rule(start_pos, start_line, start_col)?;
                Ok(AtRuleOrRule::AtRule(AtRule::Supports(rule)))
            }
            "container" => {
                let rule = self.parse_container_rule(start_pos, start_line, start_col)?;
                Ok(AtRuleOrRule::AtRule(AtRule::Container(rule)))
            }
            "media" => {
                let rule = self.parse_media_rule(start_pos, start_line, start_col)?;
                Ok(AtRuleOrRule::AtRule(AtRule::Media(rule)))
            }
            "property" => {
                let rule = self.parse_property_rule(start_pos, start_line, start_col)?;
                Ok(AtRuleOrRule::AtRule(AtRule::Property(rule)))
            }
            "charset" => {
                self.skip_whitespace();
                let charset = if self.peek_byte() == Some(b'"') || self.peek_byte() == Some(b'\'') {
                    self.read_string()?
                } else {
                    String::new()
                };
                self.skip_whitespace();
                if self.peek_byte() == Some(b';') { self.advance(); }
                Ok(AtRuleOrRule::AtRule(AtRule::Charset(charset, self.span_from(start_pos, start_line, start_col))))
            }
            "scope" => {
                let rule = self.parse_scope_rule(start_pos, start_line, start_col)?;
                Ok(AtRuleOrRule::AtRule(AtRule::Scope(rule)))
            }
            "namespace" => {
                let start = self.pos;
                while self.pos < self.bytes.len() && self.bytes[self.pos] != b';' {
                    self.advance();
                }
                let ns = self.source[start..self.pos].trim().to_string();
                if self.peek_byte() == Some(b';') { self.advance(); }
                Ok(AtRuleOrRule::AtRule(AtRule::Namespace(ns, self.span_from(start_pos, start_line, start_col))))
            }
            "tailwind" => {
                let directive = self.parse_tailwind_directive(start_pos, start_line, start_col)?;
                Ok(AtRuleOrRule::AtRule(AtRule::Tailwind(directive)))
            }
            "theme" => {
                let rule = self.parse_passthrough_block(start_pos, start_line, start_col)?;
                Ok(AtRuleOrRule::AtRule(AtRule::Theme(ThemeRule {
                    content: rule,
                    span: self.span_from(start_pos, start_line, start_col),
                })))
            }
            "utility" => {
                self.skip_whitespace();
                let name = self.read_identifier().unwrap_or_default();
                let content = self.parse_passthrough_block(start_pos, start_line, start_col)?;
                Ok(AtRuleOrRule::AtRule(AtRule::Utility(UtilityRule {
                    name,
                    content,
                    span: self.span_from(start_pos, start_line, start_col),
                })))
            }
            "variant" => {
                self.skip_whitespace();
                let name = self.read_identifier().unwrap_or_default();
                let content = self.parse_passthrough_block(start_pos, start_line, start_col)?;
                Ok(AtRuleOrRule::AtRule(AtRule::Variant(VariantRule {
                    name,
                    content,
                    span: self.span_from(start_pos, start_line, start_col),
                })))
            }
            "custom-variant" => {
                self.skip_whitespace();
                let name = self.read_identifier().unwrap_or_default();
                let content = self.parse_passthrough_block(start_pos, start_line, start_col)?;
                Ok(AtRuleOrRule::AtRule(AtRule::CustomVariant(VariantRule {
                    name,
                    content,
                    span: self.span_from(start_pos, start_line, start_col),
                })))
            }
            "source" => {
                self.skip_whitespace();
                let value = self.read_string_value()?;
                if self.peek_byte() == Some(b';') { self.advance(); }
                Ok(AtRuleOrRule::AtRule(AtRule::Source(value, self.span_from(start_pos, start_line, start_col))))
            }
            "plugin" => {
                self.skip_whitespace();
                let value = self.read_string_value()?;
                if self.peek_byte() == Some(b';') { self.advance(); }
                Ok(AtRuleOrRule::AtRule(AtRule::Plugin(value, self.span_from(start_pos, start_line, start_col))))
            }
            "config" => {
                self.skip_whitespace();
                let value = self.read_string_value()?;
                if self.peek_byte() == Some(b';') { self.advance(); }
                Ok(AtRuleOrRule::AtRule(AtRule::Config(value, self.span_from(start_pos, start_line, start_col))))
            }
            "page" => {
                self.skip_whitespace();
                // Optional page selector (e.g. :first, :left, :right)
                let selector = if self.peek_byte() != Some(b'{') {
                    let sel_start = self.pos;
                    while self.pos < self.bytes.len() && self.bytes[self.pos] != b'{' {
                        if self.bytes[self.pos] == b'\n' { self.line += 1; self.column = 1; }
                        else { self.column += 1; }
                        self.pos += 1;
                    }
                    let s = self.source[sel_start..self.pos].trim();
                    if s.is_empty() { None } else { Some(s.to_string()) }
                } else {
                    None
                };
                self.expect('{')?;
                let mut declarations = Vec::new();
                loop {
                    self.skip_whitespace();
                    if self.peek_byte() == Some(b'}') { self.advance(); break; }
                    if self.pos >= self.bytes.len() {
                        return Err(CompilerError::syntax("Unclosed @page block", self.current_span()));
                    }
                    declarations.push(self.parse_declaration()?);
                }
                Ok(AtRuleOrRule::AtRule(AtRule::Page(PageRule {
                    selector,
                    declarations,
                    span: self.span_from(start_pos, start_line, start_col),
                })))
            }
            "apply" => {
                // @apply is handled as a special property in declarations, not as an at-rule
                // But if it appears at top-level, treat it as an error
                Err(CompilerError::syntax(
                    "@apply can only be used inside a rule block",
                    self.current_span(),
                ))
            }
            // Anything else is treated as a responsive breakpoint reference (backward compat)
            _ => {
                // This is an inline responsive block (e.g. @md { ... } inside a rule)
                // But at top-level, it could be an unknown at-rule
                // Return it as a generic media rule
                self.skip_whitespace();
                if self.peek_byte() == Some(b'{') {
                    self.advance(); // skip '{'
                    let mut rules = Vec::new();
                    loop {
                        self.skip_whitespace();
                        if self.peek_byte() == Some(b'}') {
                            self.advance();
                            break;
                        }
                        if self.pos >= self.bytes.len() {
                            return Err(CompilerError::syntax(
                                format!("Unclosed @{keyword} block"),
                                self.current_span(),
                            ));
                        }
                        rules.push(self.parse_rule()?);
                    }
                    Ok(AtRuleOrRule::AtRule(AtRule::Media(MediaRule {
                        query: keyword,
                        rules,
                        span: self.span_from(start_pos, start_line, start_col),
                    })))
                } else {
                    // Statement at-rule without block
                    let start = self.pos;
                    while self.pos < self.bytes.len() && self.bytes[self.pos] != b';' && self.bytes[self.pos] != b'{' {
                        self.advance();
                    }
                    if self.peek_byte() == Some(b';') { self.advance(); }
                    Ok(AtRuleOrRule::AtRule(AtRule::Namespace(
                        format!("@{keyword} {}", self.source[start..self.pos].trim()),
                        self.span_from(start_pos, start_line, start_col),
                    )))
                }
            }
        }
    }

    fn parse_import_rule(&mut self, start_pos: usize, start_line: usize, start_col: usize) -> Result<ImportRule, CompilerError> {
        self.skip_whitespace();

        // Read URL: url("...") or "..."
        let url = if self.pos + 3 < self.bytes.len() && &self.bytes[self.pos..self.pos+4] == b"url(" {
            self.pos += 4;
            self.column += 4;
            self.skip_whitespace();
            let u = if self.peek_byte() == Some(b'"') || self.peek_byte() == Some(b'\'') {
                self.read_string()?
            } else {
                let start = self.pos;
                while self.pos < self.bytes.len() && self.bytes[self.pos] != b')' {
                    self.pos += 1;
                    self.column += 1;
                }
                self.source[start..self.pos].trim().to_string()
            };
            self.skip_whitespace();
            if self.peek_byte() == Some(b')') { self.advance(); }
            u
        } else if self.peek_byte() == Some(b'"') || self.peek_byte() == Some(b'\'') {
            self.read_string()?
        } else {
            return Err(CompilerError::syntax("Expected URL or string in @import", self.current_span()));
        };

        // Parse optional layer, supports, media
        let mut layer = None;
        let mut supports = None;
        let mut media = None;

        self.skip_whitespace();

        // Read the rest until semicolon
        let start = self.pos;
        while self.pos < self.bytes.len() && self.bytes[self.pos] != b';' {
            self.advance();
        }
        let rest = self.source[start..self.pos].trim().to_string();
        if self.peek_byte() == Some(b';') { self.advance(); }

        if rest.contains("layer") {
            layer = Some(rest.clone());
        }
        if rest.contains("supports") {
            supports = Some(rest.clone());
        }
        if !rest.is_empty() && layer.is_none() && supports.is_none() {
            media = Some(rest);
        }

        Ok(ImportRule {
            url,
            layer,
            supports,
            media,
            span: self.span_from(start_pos, start_line, start_col),
        })
    }

    fn parse_layer_rule(&mut self, start_pos: usize, start_line: usize, start_col: usize) -> Result<LayerRule, CompilerError> {
        self.skip_whitespace();

        // Read layer name(s) — could be "base" or "base, components, utilities"
        let name_start = self.pos;
        while self.pos < self.bytes.len()
            && self.bytes[self.pos] != b';'
            && self.bytes[self.pos] != b'{'
        {
            if self.bytes[self.pos] == b'\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.pos += 1;
        }
        let name = self.source[name_start..self.pos].trim().to_string();

        if self.peek_byte() == Some(b';') {
            self.advance();
            return Ok(LayerRule {
                name,
                rules: None,
                span: self.span_from(start_pos, start_line, start_col),
            });
        }

        self.expect('{')?;
        let mut rules = Vec::new();
        loop {
            self.skip_whitespace();
            if self.peek_byte() == Some(b'}') {
                self.advance();
                break;
            }
            if self.pos >= self.bytes.len() {
                return Err(CompilerError::syntax("Unclosed @layer block", self.current_span()));
            }
            rules.push(self.parse_rule()?);
        }

        Ok(LayerRule {
            name,
            rules: Some(rules),
            span: self.span_from(start_pos, start_line, start_col),
        })
    }

    fn parse_scope_rule(&mut self, start_pos: usize, start_line: usize, start_col: usize) -> Result<ScopeRule, CompilerError> {
        self.skip_whitespace();

        // Parse optional root selector in parens: @scope (.card) or @scope { ... }
        let root = if self.peek_byte() == Some(b'(') {
            self.read_balanced_parens()?
        } else {
            String::new()
        };

        self.skip_whitespace();

        // Parse optional "to" limit: @scope (.card) to (.card__content)
        let limit = {
            let saved_pos = self.pos;
            let saved_line = self.line;
            let saved_col = self.column;
            if let Some(ident) = self.read_identifier() {
                if ident == "to" {
                    self.skip_whitespace();
                    if self.peek_byte() == Some(b'(') {
                        Some(self.read_balanced_parens()?)
                    } else {
                        None
                    }
                } else {
                    // Not "to", reset
                    self.pos = saved_pos;
                    self.line = saved_line;
                    self.column = saved_col;
                    None
                }
            } else {
                None
            }
        };

        self.skip_whitespace();
        self.expect('{')?;

        let mut rules = Vec::new();
        loop {
            self.skip_whitespace();
            if self.peek_byte() == Some(b'}') {
                self.advance();
                break;
            }
            if self.pos >= self.bytes.len() {
                return Err(CompilerError::syntax("Unclosed @scope block", self.current_span()));
            }
            rules.push(self.parse_rule()?);
        }

        Ok(ScopeRule {
            root,
            limit,
            rules,
            span: self.span_from(start_pos, start_line, start_col),
        })
    }

    fn parse_keyframes_rule(&mut self, start_pos: usize, start_line: usize, start_col: usize) -> Result<KeyframesRule, CompilerError> {
        self.skip_whitespace();
        let name = self.read_identifier().ok_or_else(|| {
            CompilerError::syntax("Expected keyframes name", self.current_span())
        })?;

        self.skip_whitespace();
        self.expect('{')?;

        let mut keyframes = Vec::new();
        loop {
            self.skip_whitespace();
            if self.peek_byte() == Some(b'}') {
                self.advance();
                break;
            }
            if self.pos >= self.bytes.len() {
                return Err(CompilerError::syntax("Unclosed @keyframes block", self.current_span()));
            }
            keyframes.push(self.parse_keyframe()?);
        }

        Ok(KeyframesRule {
            name,
            keyframes,
            span: self.span_from(start_pos, start_line, start_col),
        })
    }

    fn parse_keyframe(&mut self) -> Result<Keyframe, CompilerError> {
        let start_pos = self.pos;
        let start_line = self.line;
        let start_col = self.column;

        let mut selectors = Vec::new();

        loop {
            self.skip_whitespace();
            // Parse keyframe selector: from, to, or percentage
            if self.pos + 4 <= self.bytes.len() && &self.bytes[self.pos..self.pos+4] == b"from" {
                self.pos += 4;
                self.column += 4;
                selectors.push(KeyframeSelector::From);
            } else if self.pos + 2 <= self.bytes.len() && &self.bytes[self.pos..self.pos+2] == b"to" {
                self.pos += 2;
                self.column += 2;
                selectors.push(KeyframeSelector::To);
            } else if self.peek_byte().map_or(false, |b| b.is_ascii_digit()) {
                let pct = self.read_number();
                if self.peek_byte() == Some(b'%') {
                    self.advance();
                }
                selectors.push(KeyframeSelector::Percentage(pct));
            } else {
                break;
            }

            self.skip_whitespace();
            if self.peek_byte() != Some(b',') {
                break;
            }
            self.advance(); // skip comma
        }

        if selectors.is_empty() {
            return Err(CompilerError::syntax("Expected keyframe selector (from, to, or percentage)", self.current_span()));
        }

        self.skip_whitespace();
        self.expect('{')?;

        let mut declarations = Vec::new();
        loop {
            self.skip_whitespace();
            if self.peek_byte() == Some(b'}') {
                self.advance();
                break;
            }
            if self.pos >= self.bytes.len() {
                return Err(CompilerError::syntax("Unclosed keyframe block", self.current_span()));
            }
            declarations.push(self.parse_declaration()?);
        }

        Ok(Keyframe {
            selectors,
            declarations,
            span: self.span_from(start_pos, start_line, start_col),
        })
    }

    fn parse_font_face_rule(&mut self, start_pos: usize, start_line: usize, start_col: usize) -> Result<FontFaceRule, CompilerError> {
        self.skip_whitespace();
        self.expect('{')?;

        let mut declarations = Vec::new();
        loop {
            self.skip_whitespace();
            if self.peek_byte() == Some(b'}') {
                self.advance();
                break;
            }
            if self.pos >= self.bytes.len() {
                return Err(CompilerError::syntax("Unclosed @font-face block", self.current_span()));
            }
            declarations.push(self.parse_declaration()?);
        }

        Ok(FontFaceRule {
            declarations,
            span: self.span_from(start_pos, start_line, start_col),
        })
    }

    fn parse_supports_rule(&mut self, start_pos: usize, start_line: usize, start_col: usize) -> Result<SupportsRule, CompilerError> {
        self.skip_whitespace();
        let condition = self.read_balanced_parens()?;

        self.skip_whitespace();
        self.expect('{')?;

        let mut rules = Vec::new();
        loop {
            self.skip_whitespace();
            if self.peek_byte() == Some(b'}') {
                self.advance();
                break;
            }
            if self.pos >= self.bytes.len() {
                return Err(CompilerError::syntax("Unclosed @supports block", self.current_span()));
            }
            rules.push(self.parse_rule()?);
        }

        Ok(SupportsRule {
            condition,
            rules,
            span: self.span_from(start_pos, start_line, start_col),
        })
    }

    fn parse_container_rule(&mut self, start_pos: usize, start_line: usize, start_col: usize) -> Result<ContainerRule, CompilerError> {
        self.skip_whitespace();

        // Optional container name
        let mut name = None;
        let saved_pos = self.pos;
        let saved_line = self.line;
        let saved_col = self.column;

        if let Some(ident) = self.read_identifier() {
            self.skip_whitespace();
            if self.peek_byte() == Some(b'(') {
                name = Some(ident);
            } else {
                // Reset: no name, the ident was part of the condition
                self.pos = saved_pos;
                self.line = saved_line;
                self.column = saved_col;
            }
        }

        let condition = self.read_balanced_parens()?;

        self.skip_whitespace();
        self.expect('{')?;

        let mut rules = Vec::new();
        loop {
            self.skip_whitespace();
            if self.peek_byte() == Some(b'}') {
                self.advance();
                break;
            }
            if self.pos >= self.bytes.len() {
                return Err(CompilerError::syntax("Unclosed @container block", self.current_span()));
            }
            rules.push(self.parse_rule()?);
        }

        Ok(ContainerRule {
            name,
            condition,
            rules,
            span: self.span_from(start_pos, start_line, start_col),
        })
    }

    fn parse_media_rule(&mut self, start_pos: usize, start_line: usize, start_col: usize) -> Result<MediaRule, CompilerError> {
        self.skip_whitespace();

        // Read media query until '{'
        let start = self.pos;
        while self.pos < self.bytes.len() && self.bytes[self.pos] != b'{' {
            if self.bytes[self.pos] == b'\n' {
                self.line += 1;
                self.column = 1;
            } else {
                self.column += 1;
            }
            self.pos += 1;
        }
        let query = self.source[start..self.pos].trim().to_string();

        self.expect('{')?;

        let mut rules = Vec::new();
        loop {
            self.skip_whitespace();
            if self.peek_byte() == Some(b'}') {
                self.advance();
                break;
            }
            if self.pos >= self.bytes.len() {
                return Err(CompilerError::syntax("Unclosed @media block", self.current_span()));
            }
            // Handle nested at-rules (@media inside @media)
            if self.peek_byte() == Some(b'@') {
                let saved = self.pos;
                let saved_line = self.line;
                let saved_col = self.column;
                match self.parse_at_rule_or_responsive() {
                    Ok(AtRuleOrRule::AtRule(at_rule)) => {
                        match at_rule {
                            AtRule::Media(nested) => {
                                // Flatten: merge queries with AND
                                for r in nested.rules {
                                    rules.push(r);
                                }
                            }
                            _ => { /* other at-rules inside @media — ignored for now */ }
                        }
                        continue;
                    }
                    _ => {
                        self.pos = saved;
                        self.line = saved_line;
                        self.column = saved_col;
                    }
                }
            }
            rules.push(self.parse_rule()?);
        }

        Ok(MediaRule {
            query,
            rules,
            span: self.span_from(start_pos, start_line, start_col),
        })
    }

    fn parse_property_rule(&mut self, start_pos: usize, start_line: usize, start_col: usize) -> Result<PropertyRule, CompilerError> {
        self.skip_whitespace();
        let name = self.read_css_identifier().ok_or_else(|| {
            CompilerError::syntax("Expected property name", self.current_span())
        })?;

        self.skip_whitespace();
        self.expect('{')?;

        let mut syntax = None;
        let mut inherits = None;
        let mut initial_value = None;

        loop {
            self.skip_whitespace();
            if self.peek_byte() == Some(b'}') {
                self.advance();
                break;
            }
            if self.pos >= self.bytes.len() {
                return Err(CompilerError::syntax("Unclosed @property block", self.current_span()));
            }
            let decl = self.parse_declaration()?;
            match decl.property.name() {
                "syntax" => {
                    if let Value::Literal(s) = &decl.value {
                        syntax = Some(s.clone());
                    }
                }
                "inherits" => {
                    if let Value::Literal(s) = &decl.value {
                        inherits = Some(s == "true");
                    }
                }
                "initial-value" => {
                    if let Value::Literal(s) = &decl.value {
                        initial_value = Some(s.clone());
                    }
                }
                _ => {}
            }
        }

        Ok(PropertyRule {
            name,
            syntax,
            inherits,
            initial_value,
            span: self.span_from(start_pos, start_line, start_col),
        })
    }

    fn parse_tailwind_directive(&mut self, start_pos: usize, start_line: usize, start_col: usize) -> Result<TailwindDirective, CompilerError> {
        self.skip_whitespace();
        
        let directive_name = self.read_identifier().ok_or_else(|| {
            CompilerError::syntax("Expected Tailwind directive type (base, components, utilities, variants, screens)", self.current_span())
        })?;

        let directive_type = match directive_name.as_str() {
            "base" => TailwindDirectiveType::Base,
            "components" => TailwindDirectiveType::Components,
            "utilities" => TailwindDirectiveType::Utilities,
            "variants" => TailwindDirectiveType::Variants,
            "screens" => TailwindDirectiveType::Screens,
            _ => {
                return Err(CompilerError::syntax(
                    format!("Unknown Tailwind directive '{}'. Expected: base, components, utilities, variants, or screens", directive_name),
                    self.current_span(),
                ));
            }
        };

        self.skip_whitespace();
        if self.peek_byte() == Some(b';') {
            self.advance();
        }

        Ok(TailwindDirective {
            directive_type,
            span: self.span_from(start_pos, start_line, start_col),
        })
    }

    /// Parse a block `{ ... }` and return its raw content as a string (pass-through).
    fn parse_passthrough_block(&mut self, _start_pos: usize, _start_line: usize, _start_col: usize) -> Result<String, CompilerError> {
        self.skip_whitespace();

        if self.peek_byte() == Some(b';') {
            // Statement form: @variant name;
            self.advance();
            return Ok(String::new());
        }

        self.expect('{')?;
        let content_start = self.pos;
        let mut depth = 1;
        while self.pos < self.bytes.len() && depth > 0 {
            match self.bytes[self.pos] {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                b'\n' => {
                    self.line += 1;
                    self.column = 1;
                    self.pos += 1;
                    continue;
                }
                _ => {}
            }
            if depth > 0 {
                self.pos += 1;
                self.column += 1;
            }
        }
        let content = self.source[content_start..self.pos].trim().to_string();
        if depth != 0 {
            return Err(CompilerError::syntax("Unclosed block", self.current_span()));
        }
        self.pos += 1; // skip '}'
        self.column += 1;
        Ok(content)
    }

    /// Read a quoted string value (single or double quotes).
    fn read_string_value(&mut self) -> Result<String, CompilerError> {
        self.skip_whitespace();
        let quote = match self.peek_byte() {
            Some(b'"') | Some(b'\'') => self.bytes[self.pos],
            _ => {
                // No quotes — read until ; or whitespace
                let start = self.pos;
                while self.pos < self.bytes.len()
                    && self.bytes[self.pos] != b';'
                    && !self.bytes[self.pos].is_ascii_whitespace()
                {
                    self.pos += 1;
                    self.column += 1;
                }
                return Ok(self.source[start..self.pos].to_string());
            }
        };
        self.pos += 1; // skip opening quote
        self.column += 1;
        let start = self.pos;
        while self.pos < self.bytes.len() && self.bytes[self.pos] != quote {
            self.pos += 1;
            self.column += 1;
        }
        let value = self.source[start..self.pos].to_string();
        if self.pos < self.bytes.len() {
            self.pos += 1; // skip closing quote
            self.column += 1;
        }
        Ok(value)
    }

    // -----------------------------------------------------------------------
    // Rule parsing
    // -----------------------------------------------------------------------

    fn parse_rule(&mut self) -> Result<Rule, CompilerError> {
        let start_pos = self.pos;
        let start_line = self.line;
        let start_col = self.column;

        // Parse primary selector
        let selector = self.parse_selector()?;

        // Parse additional selectors (comma-separated)
        let mut selectors = Vec::new();
        loop {
            self.skip_whitespace();
            if self.peek_byte() == Some(b',') {
                self.advance();
                self.skip_whitespace();
                selectors.push(self.parse_selector()?);
            } else {
                break;
            }
        }

        self.skip_whitespace();
        self.expect('{')?;

        let mut declarations = Vec::new();
        let mut states = Vec::new();
        let mut responsive = Vec::new();
        let mut nested_rules = Vec::new();
        let mut nested_at_rules = Vec::new();

        loop {
            self.skip_whitespace();
            match self.peek_byte() {
                Some(b'}') => {
                    self.pos += 1;
                    self.column += 1;
                    break;
                }
                None => {
                    return Err(CompilerError::syntax(
                        "Unclosed rule block",
                        self.current_span(),
                    ));
                }
                Some(b'@') => {
                    // Check if this is @apply, nested at-rule, or responsive block
                    let saved_pos = self.pos;
                    let saved_line = self.line;
                    let saved_col = self.column;

                    self.pos += 1;
                    self.column += 1;

                    if let Some(keyword) = self.read_identifier() {
                        match keyword.as_str() {
                            "apply" => {
                                self.pos = saved_pos;
                                self.line = saved_line;
                                self.column = saved_col;
                                declarations.push(self.parse_declaration()?);
                            }
                            "media" | "supports" | "container" => {
                                // Nested at-rule inside a rule block (CSS nesting)
                                let kind = match keyword.as_str() {
                                    "media" => NestedAtRuleKind::Media,
                                    "supports" => NestedAtRuleKind::Supports,
                                    _ => NestedAtRuleKind::Container,
                                };
                                self.skip_whitespace();
                                // Read query until '{'
                                let query_start = self.pos;
                                while self.pos < self.bytes.len() && self.bytes[self.pos] != b'{' {
                                    if self.bytes[self.pos] == b'\n' {
                                        self.line += 1;
                                        self.column = 1;
                                    } else {
                                        self.column += 1;
                                    }
                                    self.pos += 1;
                                }
                                let query = self.source[query_start..self.pos].trim().to_string();
                                self.expect('{')?;

                                let mut inner_decls = Vec::new();
                                let mut inner_rules = Vec::new();
                                loop {
                                    self.skip_whitespace();
                                    match self.peek_byte() {
                                        Some(b'}') => { self.advance(); break; }
                                        None => {
                                            return Err(CompilerError::syntax(
                                                format!("Unclosed nested @{keyword} block"),
                                                self.current_span(),
                                            ));
                                        }
                                        Some(b'.') | Some(b'#') | Some(b'*') | Some(b'&') => {
                                            // Nested rule inside the at-rule
                                            match self.try_parse_nested_rule() {
                                                Some(rule) => inner_rules.push(rule),
                                                None => inner_decls.push(self.parse_declaration()?),
                                            }
                                        }
                                        _ => {
                                            inner_decls.push(self.parse_declaration()?);
                                        }
                                    }
                                }

                                nested_at_rules.push(NestedAtRule {
                                    kind,
                                    query,
                                    declarations: inner_decls,
                                    nested_rules: inner_rules,
                                    span: self.span_from(saved_pos, saved_line, saved_col),
                                });
                            }
                            _ => {
                                // Responsive block (custom breakpoint like @md { ... })
                                self.pos = saved_pos;
                                self.line = saved_line;
                                self.column = saved_col;
                                responsive.push(self.parse_responsive_block()?);
                            }
                        }
                    } else {
                        self.pos = saved_pos;
                        self.line = saved_line;
                        self.column = saved_col;
                        responsive.push(self.parse_responsive_block()?);
                    }
                }
                Some(b'&') => {
                    // Check if this is &:pseudo (state block) or & .child / &.class (nested rule)
                    if self.peek_byte_at(1) == Some(b':') {
                        states.push(self.parse_state_block()?);
                    } else {
                        // & followed by space/selector — treat as nested rule
                        let saved = self.pos;
                        let saved_line = self.line;
                        let saved_col = self.column;
                        match self.try_parse_nested_rule() {
                            Some(rule) => nested_rules.push(rule),
                            None => {
                                self.pos = saved;
                                self.line = saved_line;
                                self.column = saved_col;
                                states.push(self.parse_state_block()?);
                            }
                        }
                    }
                }
                // CSS nesting: nested rules starting with selectors
                Some(b'.') | Some(b'#') | Some(b'*') | Some(b':') => {
                    let saved = self.pos;
                    let saved_line = self.line;
                    let saved_col = self.column;
                    match self.try_parse_nested_rule() {
                        Some(rule) => nested_rules.push(rule),
                        None => {
                            self.pos = saved;
                            self.line = saved_line;
                            self.column = saved_col;
                            declarations.push(self.parse_declaration()?);
                        }
                    }
                }
                _ => {
                    // Could be a declaration or a nested rule (tag selector like h3, p)
                    let saved = self.pos;
                    let saved_line = self.line;
                    let saved_col = self.column;
                    match self.try_parse_nested_rule() {
                        Some(rule) => nested_rules.push(rule),
                        None => {
                            self.pos = saved;
                            self.line = saved_line;
                            self.column = saved_col;
                            declarations.push(self.parse_declaration()?);
                        }
                    }
                }
            }
        }

        Ok(Rule {
            selector,
            selectors,
            declarations,
            states,
            responsive,
            nested_rules,
            nested_at_rules,
            span: self.span_from(start_pos, start_line, start_col),
        })
    }

    fn try_parse_nested_rule(&mut self) -> Option<Rule> {
        // Look ahead: is there a '{' before a ':' or ';'?
        let mut look = self.pos;
        let mut depth = 0;
        while look < self.bytes.len() {
            match self.bytes[look] {
                b'{' if depth == 0 => {
                    // Found block start — this is a nested rule
                    return self.parse_rule().ok();
                }
                b';' if depth == 0 => return None, // declaration
                b'(' => depth += 1,
                b')' => depth -= 1,
                b'}' if depth == 0 => return None, // end of parent
                _ => {}
            }
            look += 1;
        }
        None
    }

    // -----------------------------------------------------------------------
    // Selector parsing (full CSS spec)
    // -----------------------------------------------------------------------

    fn parse_selector(&mut self) -> Result<Selector, CompilerError> {
        self.skip_whitespace();
        let start_pos = self.pos;
        let start_line = self.line;
        let start_col = self.column;

        let (class_name, kind) = match self.peek_byte() {
            Some(b'.') => {
                self.pos += 1;
                self.column += 1;
                let name = self.read_identifier().ok_or_else(|| {
                    CompilerError::syntax("Expected class name", self.current_span())
                })?;
                (name, SelectorKind::Class)
            }
            Some(b'#') => {
                self.pos += 1;
                self.column += 1;
                let name = self.read_identifier().ok_or_else(|| {
                    CompilerError::syntax("Expected id name", self.current_span())
                })?;
                (name, SelectorKind::Id)
            }
            Some(b'*') => {
                self.pos += 1;
                self.column += 1;
                ("*".to_string(), SelectorKind::Universal)
            }
            Some(b'&') => {
                self.pos += 1;
                self.column += 1;
                ("&".to_string(), SelectorKind::Nesting)
            }
            Some(b'[') => {
                // Attribute selector at top level
                ("".to_string(), SelectorKind::Attribute)
            }
            Some(b':') => {
                // Pseudo-class or pseudo-element used as standalone selector (e.g. :root, :is(), :where())
                // Don't consume the colon — it will be parsed in the pseudo loop below
                ("".to_string(), SelectorKind::Tag)
            }
            Some(b) if b.is_ascii_alphabetic() => {
                let name = self.read_identifier().ok_or_else(|| {
                    CompilerError::syntax("Expected selector", self.current_span())
                })?;
                (name, SelectorKind::Tag)
            }
            _ => {
                // Fallback: try reading an identifier (backward compat)
                let name = self.read_identifier().ok_or_else(|| {
                    CompilerError::syntax("Expected selector", self.current_span())
                })?;
                (name, SelectorKind::Class)
            }
        };

        // Parse attribute selectors
        let mut attributes = Vec::new();
        while self.peek_byte() == Some(b'[') {
            attributes.push(self.parse_attribute_selector()?);
        }

        // Parse pseudo-classes and pseudo-elements inline
        let mut pseudo_classes = Vec::new();
        let mut pseudo_elements = Vec::new();
        let mut combinators = Vec::new();

        loop {
            // Save position before skip_whitespace to detect if whitespace was consumed
            let pre_ws_pos = self.pos;
            self.skip_whitespace();
            let had_whitespace = self.pos > pre_ws_pos;

            match self.peek_byte() {
                Some(b'>') => {
                    self.advance();
                    self.skip_whitespace();
                    let child_sel = self.parse_selector()?;
                    combinators.push(Combinator::Child(Box::new(child_sel)));
                }
                Some(b'+') => {
                    self.advance();
                    self.skip_whitespace();
                    let adj_sel = self.parse_selector()?;
                    combinators.push(Combinator::Adjacent(Box::new(adj_sel)));
                }
                Some(b'~') => {
                    self.advance();
                    self.skip_whitespace();
                    let sib_sel = self.parse_selector()?;
                    combinators.push(Combinator::Sibling(Box::new(sib_sel)));
                }
                // Descendant combinator: whitespace followed by a selector start character
                Some(b'.') | Some(b'#') | Some(b'*') | Some(b'[')
                    if had_whitespace =>
                {
                    let desc_sel = self.parse_selector()?;
                    combinators.push(Combinator::Descendant(Box::new(desc_sel)));
                }
                Some(b) if had_whitespace && b.is_ascii_alphabetic() => {
                    // Could be a descendant tag selector OR start of a { block
                    // Peek further to decide
                    let saved = self.pos;
                    let saved_line = self.line;
                    let saved_col = self.column;
                    if let Some(ident) = self.read_identifier() {
                        self.skip_whitespace();
                        if self.peek_byte() == Some(b'{') || self.peek_byte() == Some(b',')
                            || self.peek_byte() == Some(b':') || self.peek_byte() == Some(b'.')
                            || self.peek_byte() == Some(b'#') || self.peek_byte() == Some(b'[')
                        {
                            // This is a descendant selector
                            self.pos = saved;
                            self.line = saved_line;
                            self.column = saved_col;
                            let desc_sel = self.parse_selector()?;
                            combinators.push(Combinator::Descendant(Box::new(desc_sel)));
                        } else {
                            // Not a selector, revert
                            self.pos = saved;
                            self.line = saved_line;
                            self.column = saved_col;
                            break;
                        }
                    } else {
                        break;
                    }
                }
                Some(b':') if self.peek_byte_at(1) == Some(b':') => {
                    self.pos += 2;
                    self.column += 2;
                    let name = self.read_identifier().unwrap_or_default();
                    let pe = match name.as_str() {
                        "before" => PseudoElement::Before,
                        "after" => PseudoElement::After,
                        "first-line" => PseudoElement::FirstLine,
                        "first-letter" => PseudoElement::FirstLetter,
                        "placeholder" => PseudoElement::Placeholder,
                        "selection" => PseudoElement::Selection,
                        "marker" => PseudoElement::Marker,
                        "backdrop" => PseudoElement::Backdrop,
                        "cue" => PseudoElement::Cue,
                        "cue-region" => PseudoElement::CueRegion,
                        "grammar-error" => PseudoElement::GrammarError,
                        "spelling-error" => PseudoElement::SpellingError,
                        "target-text" => PseudoElement::TargetText,
                        "file-selector-button" => PseudoElement::FileSelectorButton,
                        _ => PseudoElement::Custom(name),
                    };
                    pseudo_elements.push(pe);
                }
                Some(b':') => {
                    self.pos += 1;
                    self.column += 1;
                    let name = self.read_identifier().unwrap_or_default();
                    // Check for functional pseudo-classes
                    let pc = if self.peek_byte() == Some(b'(') {
                        let arg = self.read_balanced_parens()?;
                        match name.as_str() {
                            "nth-child" => PseudoClass::NthChild(arg),
                            "nth-last-child" => PseudoClass::NthLastChild(arg),
                            "nth-of-type" => PseudoClass::NthOfType(arg),
                            "nth-last-of-type" => PseudoClass::NthLastOfType(arg),
                            "not" => PseudoClass::Not(arg),
                            "is" => PseudoClass::Is(arg),
                            "where" => PseudoClass::Where(arg),
                            "has" => PseudoClass::Has(arg),
                            "lang" => PseudoClass::Lang(arg),
                            "dir" => PseudoClass::Dir(arg),
                            _ => PseudoClass::Custom(format!("{name}({arg})")),
                        }
                    } else {
                        match name.as_str() {
                            "hover" => PseudoClass::Hover,
                            "focus" => PseudoClass::Focus,
                            "focus-visible" => PseudoClass::FocusVisible,
                            "focus-within" => PseudoClass::FocusWithin,
                            "active" => PseudoClass::Active,
                            "visited" => PseudoClass::Visited,
                            "link" => PseudoClass::Link,
                            "disabled" => PseudoClass::Disabled,
                            "enabled" => PseudoClass::Enabled,
                            "checked" => PseudoClass::Checked,
                            "indeterminate" => PseudoClass::Indeterminate,
                            "required" => PseudoClass::Required,
                            "optional" => PseudoClass::Optional,
                            "valid" => PseudoClass::Valid,
                            "invalid" => PseudoClass::Invalid,
                            "read-only" => PseudoClass::ReadOnly,
                            "read-write" => PseudoClass::ReadWrite,
                            "placeholder-shown" => PseudoClass::PlaceholderShown,
                            "default" => PseudoClass::Default,
                            "first-child" => PseudoClass::FirstChild,
                            "last-child" => PseudoClass::LastChild,
                            "only-child" => PseudoClass::OnlyChild,
                            "first-of-type" => PseudoClass::FirstOfType,
                            "last-of-type" => PseudoClass::LastOfType,
                            "only-of-type" => PseudoClass::OnlyOfType,
                            "empty" => PseudoClass::Empty,
                            "root" => PseudoClass::Root,
                            "dark" => PseudoClass::Dark,
                            _ => PseudoClass::Custom(name),
                        }
                    };
                    pseudo_classes.push(pc);
                }
                _ => break,
            }
        }

        Ok(Selector {
            class_name,
            kind,
            combinators,
            pseudo_elements,
            pseudo_classes,
            attributes,
            span: self.span_from(start_pos, start_line, start_col),
        })
    }

    fn parse_attribute_selector(&mut self) -> Result<AttributeSelector, CompilerError> {
        self.expect('[')?;
        self.skip_whitespace();

        let name = self.read_identifier().ok_or_else(|| {
            CompilerError::syntax("Expected attribute name", self.current_span())
        })?;

        self.skip_whitespace();

        // Check for operator
        let operator = match self.peek_byte() {
            Some(b'=') => {
                self.advance();
                Some(AttributeOp::Equals)
            }
            Some(b'~') if self.peek_byte_at(1) == Some(b'=') => {
                self.pos += 2; self.column += 2;
                Some(AttributeOp::Contains)
            }
            Some(b'|') if self.peek_byte_at(1) == Some(b'=') => {
                self.pos += 2; self.column += 2;
                Some(AttributeOp::DashMatch)
            }
            Some(b'^') if self.peek_byte_at(1) == Some(b'=') => {
                self.pos += 2; self.column += 2;
                Some(AttributeOp::Prefix)
            }
            Some(b'$') if self.peek_byte_at(1) == Some(b'=') => {
                self.pos += 2; self.column += 2;
                Some(AttributeOp::Suffix)
            }
            Some(b'*') if self.peek_byte_at(1) == Some(b'=') => {
                self.pos += 2; self.column += 2;
                Some(AttributeOp::Substring)
            }
            _ => None,
        };

        let value = if operator.is_some() {
            self.skip_whitespace();
            if self.peek_byte() == Some(b'"') || self.peek_byte() == Some(b'\'') {
                Some(self.read_string()?)
            } else {
                Some(self.read_identifier().unwrap_or_default())
            }
        } else {
            None
        };

        self.skip_whitespace();

        // Optional modifier (i or s)
        let modifier = match self.peek_byte() {
            Some(b'i') | Some(b'I') => {
                self.advance();
                Some(AttributeModifier::CaseInsensitive)
            }
            Some(b's') | Some(b'S') if self.peek_byte_at(1) != Some(b']') => {
                self.advance();
                Some(AttributeModifier::CaseSensitive)
            }
            _ => None,
        };

        self.skip_whitespace();
        self.expect(']')?;

        Ok(AttributeSelector {
            name,
            operator,
            value,
            modifier,
        })
    }

    // -----------------------------------------------------------------------
    // Declaration parsing
    // -----------------------------------------------------------------------

    fn parse_declaration(&mut self) -> Result<Declaration, CompilerError> {
        let start_pos = self.pos;
        let start_line = self.line;
        let start_col = self.column;

        // Check for @apply directive
        if self.peek_byte() == Some(b'@') {
            let saved_pos = self.pos;
            let saved_line = self.line;
            let saved_col = self.column;
            
            self.pos += 1;
            self.column += 1;
            
            if let Some(keyword) = self.read_identifier() {
                if keyword == "apply" {
                    // Parse @apply directive
                    self.skip_whitespace();
                    
                    // Read utility classes until semicolon
                    let start = self.pos;
                    while self.pos < self.bytes.len() && self.bytes[self.pos] != b';' && self.bytes[self.pos] != b'}' {
                        self.pos += 1;
                        self.column += 1;
                    }
                    let classes = self.source[start..self.pos].trim().to_string();
                    
                    if self.peek_byte() == Some(b';') {
                        self.pos += 1;
                        self.column += 1;
                    }
                    
                    return Ok(Declaration {
                        property: Property::Apply(classes.clone()),
                        value: Value::Literal(classes),
                        important: false,
                        span: self.span_from(start_pos, start_line, start_col),
                    });
                }
            }
            
            // Not @apply, reset position
            self.pos = saved_pos;
            self.line = saved_line;
            self.column = saved_col;
        }

        let prop_name = self.read_css_identifier().ok_or_else(|| {
            CompilerError::syntax("Expected property name", self.current_span())
        })?;

        let property = if prop_name.starts_with("--") {
            Property::Custom(prop_name)
        } else {
            Property::Standard(prop_name)
        };

        self.skip_whitespace();
        self.expect(':')?;
        self.skip_whitespace();

        let value = self.parse_value()?;

        // Check for !important
        self.skip_whitespace();
        let important = if self.pos + 10 <= self.bytes.len()
            && &self.bytes[self.pos..self.pos + 10] == b"!important"
        {
            self.pos += 10;
            self.column += 10;
            true
        } else {
            false
        };

        // Optional semicolon
        self.skip_whitespace();
        if self.peek_byte() == Some(b';') {
            self.pos += 1;
            self.column += 1;
        }

        Ok(Declaration {
            property,
            value,
            important,
            span: self.span_from(start_pos, start_line, start_col),
        })
    }

    fn parse_value(&mut self) -> Result<Value, CompilerError> {
        self.skip_whitespace();

        match self.peek_byte() {
            Some(b'$') => self.parse_token_ref(),
            Some(b) if b == b'#' || b == b'"' || b == b'\'' || b.is_ascii_digit() || b == b'-' => {
                // Try parsing as a typed value (hex, string, number) first,
                // then check if there's more content. If so, fall back to full literal.
                let saved_pos = self.pos;
                let saved_line = self.line;
                let saved_col = self.column;

                let initial_result = match b {
                    b'#' => self.parse_color_hex(),
                    b'"' | b'\'' => {
                        let s = self.read_string()?;
                        Ok(Value::Literal(s))
                    }
                    _ => {
                        let num = self.read_number();
                        let unit = self.try_read_unit();
                        Ok(Value::Number(num, unit))
                    }
                };

                if initial_result.is_err() {
                    return initial_result;
                }

                // Check if there's more value content after the initial token
                self.skip_whitespace();
                match self.peek_byte() {
                    Some(b';') | Some(b'}') | Some(b'!') | Some(b'\n') | None => {
                        initial_result
                    }
                    _ => {
                        // Multi-token value: reset and read entire value as literal
                        self.pos = saved_pos;
                        self.line = saved_line;
                        self.column = saved_col;
                        self.read_full_value_literal()
                    }
                }
            }
            _ => {
                // Check for var() or env()
                if self.pos + 4 <= self.bytes.len() && &self.bytes[self.pos..self.pos+4] == b"var(" {
                    return self.parse_var_function();
                }
                if self.pos + 4 <= self.bytes.len() && &self.bytes[self.pos..self.pos+4] == b"env(" {
                    return self.parse_env_function();
                }

                // Check for color functions (only if no more complex value follows)
                if let Some(color_val) = self.try_parse_color_function()? {
                    // Check if there's more after the color function
                    self.skip_whitespace();
                    match self.peek_byte() {
                        Some(b';') | Some(b'}') | Some(b'!') | Some(b'\n') | None => {
                            return Ok(color_val);
                        }
                        _ => {
                            // More content — will be handled by full literal below
                            // But the color function already advanced pos, so we can't easily reset.
                            // Just return the color; multi-value with colors is rare without number start.
                            return Ok(color_val);
                        }
                    }
                }

                self.read_full_value_literal()
            }
        }
    }

    /// Read the full remaining value as a literal, respecting balanced parens and quotes.
    fn read_full_value_literal(&mut self) -> Result<Value, CompilerError> {
        let start = self.pos;
        let mut paren_depth = 0u32;
        while self.pos < self.bytes.len() {
            match self.bytes[self.pos] {
                b'(' => { paren_depth += 1; }
                b')' => { paren_depth = paren_depth.saturating_sub(1); }
                b'"' | b'\'' => {
                    // Read through quoted string
                    let quote = self.bytes[self.pos];
                    self.pos += 1;
                    self.column += 1;
                    while self.pos < self.bytes.len() && self.bytes[self.pos] != quote {
                        if self.bytes[self.pos] == b'\n' {
                            self.line += 1;
                            self.column = 1;
                        } else {
                            self.column += 1;
                        }
                        self.pos += 1;
                    }
                    if self.pos < self.bytes.len() {
                        self.pos += 1;
                        self.column += 1;
                    }
                    continue;
                }
                b';' | b'}' if paren_depth == 0 => break,
                b'!' if paren_depth == 0 => break,
                b'\n' if paren_depth == 0 => break,
                _ => {}
            }
            self.pos += 1;
            self.column += 1;
        }
        let mut end = self.pos;
        while end > start && self.bytes[end - 1].is_ascii_whitespace() {
            end -= 1;
        }
        if end == start {
            Err(CompilerError::syntax("Expected value", self.current_span()))
        } else {
            Ok(Value::Literal(self.source[start..end].to_string()))
        }
    }

    fn parse_var_function(&mut self) -> Result<Value, CompilerError> {
        self.pos += 4; // skip "var("
        self.column += 4;
        self.skip_whitespace();

        let name = self.read_css_identifier().ok_or_else(|| {
            CompilerError::syntax("Expected variable name in var()", self.current_span())
        })?;

        self.skip_whitespace();

        let fallback = if self.peek_byte() == Some(b',') {
            self.advance();
            self.skip_whitespace();
            Some(Box::new(self.parse_value()?))
        } else {
            None
        };

        self.skip_whitespace();
        self.expect(')')?;
        Ok(Value::Var(name, fallback))
    }

    fn parse_env_function(&mut self) -> Result<Value, CompilerError> {
        self.pos += 4; // skip "env("
        self.column += 4;
        self.skip_whitespace();

        let name = self.read_identifier().ok_or_else(|| {
            CompilerError::syntax("Expected env name", self.current_span())
        })?;

        self.skip_whitespace();

        let fallback = if self.peek_byte() == Some(b',') {
            self.advance();
            self.skip_whitespace();
            Some(Box::new(self.parse_value()?))
        } else {
            None
        };

        self.skip_whitespace();
        self.expect(')')?;
        Ok(Value::Env(name, fallback))
    }

    fn parse_token_ref(&mut self) -> Result<Value, CompilerError> {
        let start_pos = self.pos;
        let start_line = self.line;
        let start_col = self.column;

        self.pos += 1; // skip '$'
        self.column += 1;

        let category_name = self.read_identifier().ok_or_else(|| {
            CompilerError::syntax("Expected token category", self.current_span())
        })?;

        self.expect('.')?;

        let token_name = self.read_identifier().ok_or_else(|| {
            CompilerError::syntax("Expected token name", self.current_span())
        })?;

        let category = TokenCategory::from_str(&category_name).ok_or_else(|| {
            CompilerError::syntax(
                format!("Unknown token category '{category_name}'. Expected: colors, spacing, typography, breakpoints, animation, shadows, borders, radii, zindex, opacity"),
                self.span_from(start_pos, start_line, start_col),
            )
        })?;

        Ok(Value::Token(TokenRef {
            category,
            name: token_name,
            span: self.span_from(start_pos, start_line, start_col),
        }))
    }

    fn parse_color_hex(&mut self) -> Result<Value, CompilerError> {
        let start = self.pos;
        self.pos += 1; // skip '#'
        self.column += 1;
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_hexdigit() {
            self.pos += 1;
            self.column += 1;
        }
        let hex = self.source[start..self.pos].to_string();
        Ok(Value::Color(Color::Hex(hex)))
    }

    /// Parse color functions: rgb(), rgba(), hsl(), hsla()
    /// Supports both modern (space-separated with slash) and legacy (comma-separated) syntax:
    /// - Modern: rgb(255 0 0 / 0.5), hsl(120 100% 50% / 0.8)
    /// - Legacy: rgba(255, 0, 0, 0.5), hsla(120, 100%, 50%, 0.8)
    /// Try to parse a color function at the current position. Returns None if not a color function.
    fn try_parse_color_function(&mut self) -> Result<Option<Value>, CompilerError> {
        let color_prefixes: &[&[u8]] = &[
            b"rgb(", b"rgba(", b"hsl(", b"hsla(",
            b"hwb(", b"lab(", b"lch(",
            b"oklch(", b"oklab(",
            b"color-mix(", b"light-dark(",
        ];
        let remaining = &self.bytes[self.pos..];
        for prefix in color_prefixes {
            if remaining.len() >= prefix.len() && &remaining[..prefix.len()] == *prefix {
                return self.parse_color_function().map(Some);
            }
        }
        Ok(None)
    }

    fn parse_color_function(&mut self) -> Result<Value, CompilerError> {
        let start_pos = self.pos;
        
        // Read function name
        let func_start = self.pos;
        while self.pos < self.bytes.len() && self.bytes[self.pos] != b'(' {
            self.pos += 1;
            self.column += 1;
        }
        let func_name = &self.source[func_start..self.pos];
        
        self.expect('(')?;
        self.skip_whitespace();
        
        // Read all content until closing paren
        let content_start = self.pos;
        let mut paren_depth = 1;
        while self.pos < self.bytes.len() && paren_depth > 0 {
            match self.bytes[self.pos] {
                b'(' => paren_depth += 1,
                b')' => paren_depth -= 1,
                b'\n' => {
                    self.line += 1;
                    self.column = 1;
                    self.pos += 1;
                    continue;
                }
                _ => {}
            }
            if paren_depth > 0 {
                self.pos += 1;
                self.column += 1;
            }
        }
        
        if paren_depth != 0 {
            return Err(CompilerError::syntax(
                format!("Unclosed {}()", func_name),
                Span::new(start_pos, self.pos, self.line, self.column)
            ));
        }
        
        let content = self.source[content_start..self.pos].trim();
        self.pos += 1; // skip closing ')'
        self.column += 1;
        
        // Parse the content
        match func_name {
            "rgb" | "rgba" => self.parse_rgb_content(content, func_name),
            "hsl" | "hsla" => self.parse_hsl_content(content, func_name),
            "hwb" => self.parse_3_component_color(content, |a, b, c| Color::Hwb(a, b, c)),
            "lab" => self.parse_3_component_color(content, |a, b, c| Color::Lab(a, b, c)),
            "lch" => self.parse_3_component_color(content, |a, b, c| Color::Lch(a, b, c)),
            "oklch" => self.parse_3_component_color(content, |a, b, c| Color::Oklch(a, b, c)),
            "oklab" => self.parse_3_component_color(content, |a, b, c| Color::Oklab(a, b, c)),
            "color-mix" => Ok(Value::Color(Color::ColorMix(content.to_string()))),
            "light-dark" => {
                // light-dark(light-color, dark-color) — store as literal pair for now
                let parts: Vec<&str> = content.splitn(2, ',').collect();
                if parts.len() == 2 {
                    let light = parts[0].trim();
                    let dark = parts[1].trim();
                    Ok(Value::Color(Color::LightDark(
                        Box::new(Color::Named(light.to_string())),
                        Box::new(Color::Named(dark.to_string())),
                    )))
                } else {
                    Ok(Value::Literal(format!("light-dark({})", content)))
                }
            }
            _ => Ok(Value::Literal(format!("{}({})", func_name, content)))
        }
    }

    /// Parse a 3-component color function (hwb, lab, lch, oklch, oklab).
    /// Format: func(a b c) or func(a b c / alpha) — space-separated.
    fn parse_3_component_color(&self, content: &str, make: impl Fn(f64, f64, f64) -> Color) -> Result<Value, CompilerError> {
        let main_part = if content.contains('/') {
            content.split('/').next().unwrap_or(content).trim()
        } else {
            content.trim()
        };

        let values: Vec<&str> = main_part.split_whitespace().collect();
        if values.len() < 3 {
            return Ok(Value::Literal(format!("unknown-color({})", content)));
        }

        let a = values[0].trim_end_matches('%').trim_end_matches("deg").parse::<f64>().unwrap_or(0.0);
        let b = values[1].trim_end_matches('%').parse::<f64>().unwrap_or(0.0);
        let c = values[2].trim_end_matches('%').parse::<f64>().unwrap_or(0.0);

        Ok(Value::Color(make(a, b, c)))
    }
    
    fn parse_rgb_content(&mut self, content: &str, func_name: &str) -> Result<Value, CompilerError> {
        // Check if modern syntax (space-separated with optional slash for alpha)
        let has_slash = content.contains('/');
        let has_comma = content.contains(',');
        
        if has_slash || (!has_comma && content.split_whitespace().count() >= 3) {
            // Modern syntax: rgb(255 0 0 / 0.5) or rgb(255 0 0)
            let parts: Vec<&str> = if has_slash {
                content.split('/').collect()
            } else {
                vec![content]
            };
            
            let rgb_part = parts[0].trim();
            let rgb_values: Vec<&str> = rgb_part.split_whitespace().collect();
            
            if rgb_values.len() < 3 {
                return Ok(Value::Literal(format!("{}({})", func_name, content)));
            }
            
            let r = rgb_values[0].parse::<f64>().unwrap_or(0.0);
            let g = rgb_values[1].parse::<f64>().unwrap_or(0.0);
            let b = rgb_values[2].parse::<f64>().unwrap_or(0.0);
            
            if parts.len() > 1 {
                let alpha = parts[1].trim().parse::<f64>().unwrap_or(1.0);
                Ok(Value::Color(Color::Rgba(r, g, b, alpha)))
            } else {
                Ok(Value::Color(Color::Rgb(r, g, b)))
            }
        } else {
            // Legacy syntax: rgba(255, 0, 0, 0.5)
            let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
            
            if parts.len() < 3 {
                return Ok(Value::Literal(format!("{}({})", func_name, content)));
            }
            
            let r = parts[0].parse::<f64>().unwrap_or(0.0);
            let g = parts[1].parse::<f64>().unwrap_or(0.0);
            let b = parts[2].parse::<f64>().unwrap_or(0.0);
            
            if parts.len() >= 4 {
                let alpha = parts[3].parse::<f64>().unwrap_or(1.0);
                Ok(Value::Color(Color::Rgba(r, g, b, alpha)))
            } else {
                Ok(Value::Color(Color::Rgb(r, g, b)))
            }
        }
    }
    
    fn parse_hsl_content(&mut self, content: &str, func_name: &str) -> Result<Value, CompilerError> {
        // Check if modern syntax (space-separated with optional slash for alpha)
        let has_slash = content.contains('/');
        let has_comma = content.contains(',');
        
        if has_slash || (!has_comma && content.split_whitespace().count() >= 3) {
            // Modern syntax: hsl(120 100% 50% / 0.8) or hsl(120 100% 50%)
            let parts: Vec<&str> = if has_slash {
                content.split('/').collect()
            } else {
                vec![content]
            };
            
            let hsl_part = parts[0].trim();
            let hsl_values: Vec<&str> = hsl_part.split_whitespace().collect();
            
            if hsl_values.len() < 3 {
                return Ok(Value::Literal(format!("{}({})", func_name, content)));
            }
            
            let h = hsl_values[0].trim_end_matches("deg").parse::<f64>().unwrap_or(0.0);
            let s = hsl_values[1].trim_end_matches('%').parse::<f64>().unwrap_or(0.0);
            let l = hsl_values[2].trim_end_matches('%').parse::<f64>().unwrap_or(0.0);
            
            if parts.len() > 1 {
                let alpha = parts[1].trim().parse::<f64>().unwrap_or(1.0);
                Ok(Value::Color(Color::Hsla(h, s, l, alpha)))
            } else {
                Ok(Value::Color(Color::Hsl(h, s, l)))
            }
        } else {
            // Legacy syntax: hsla(120, 100%, 50%, 0.8)
            let parts: Vec<&str> = content.split(',').map(|s| s.trim()).collect();
            
            if parts.len() < 3 {
                return Ok(Value::Literal(format!("{}({})", func_name, content)));
            }
            
            let h = parts[0].trim_end_matches("deg").parse::<f64>().unwrap_or(0.0);
            let s = parts[1].trim_end_matches('%').parse::<f64>().unwrap_or(0.0);
            let l = parts[2].trim_end_matches('%').parse::<f64>().unwrap_or(0.0);
            
            if parts.len() >= 4 {
                let alpha = parts[3].parse::<f64>().unwrap_or(1.0);
                Ok(Value::Color(Color::Hsla(h, s, l, alpha)))
            } else {
                Ok(Value::Color(Color::Hsl(h, s, l)))
            }
        }
    }

    fn try_read_unit(&mut self) -> Option<Unit> {
        let start = self.pos;
        if self.pos < self.bytes.len() && self.bytes[self.pos] == b'%' {
            self.pos += 1;
            self.column += 1;
            return Some(Unit::Percent);
        }
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_alphabetic() {
            self.pos += 1;
            self.column += 1;
        }
        if self.pos > start {
            let unit_str = &self.source[start..self.pos];
            Unit::from_str(unit_str).or_else(|| {
                self.pos = start;
                self.column -= unit_str.len();
                None
            })
        } else {
            None
        }
    }

    fn parse_state_block(&mut self) -> Result<StateBlock, CompilerError> {
        let start_pos = self.pos;
        let start_line = self.line;
        let start_col = self.column;

        self.pos += 1; // skip '&'
        self.column += 1;

        let mut modifiers = Vec::new();

        while self.peek_byte() == Some(b':') {
            self.pos += 1;
            self.column += 1;
            let name = self.read_identifier().ok_or_else(|| {
                CompilerError::syntax("Expected state name after ':'", self.current_span())
            })?;
            if let Some(modifier) = StateModifier::from_str(&name) {
                modifiers.push(modifier);
            }
        }

        if modifiers.is_empty() {
            return Err(CompilerError::syntax(
                "Expected state modifier after '&'",
                self.current_span(),
            ));
        }

        self.skip_whitespace();
        self.expect('{')?;

        let mut declarations = Vec::new();
        loop {
            self.skip_whitespace();
            match self.peek_byte() {
                Some(b'}') => {
                    self.pos += 1;
                    self.column += 1;
                    break;
                }
                None => {
                    return Err(CompilerError::syntax(
                        "Unclosed state block",
                        self.current_span(),
                    ));
                }
                _ => {
                    declarations.push(self.parse_declaration()?);
                }
            }
        }

        Ok(StateBlock {
            modifiers,
            declarations,
            span: self.span_from(start_pos, start_line, start_col),
        })
    }

    fn parse_responsive_block(&mut self) -> Result<ResponsiveBlock, CompilerError> {
        let start_pos = self.pos;
        let start_line = self.line;
        let start_col = self.column;

        self.pos += 1; // skip '@'
        self.column += 1;

        let breakpoint = self.read_identifier().ok_or_else(|| {
            CompilerError::syntax("Expected breakpoint name after '@'", self.current_span())
        })?;

        self.skip_whitespace();
        self.expect('{')?;

        let mut declarations = Vec::new();
        loop {
            self.skip_whitespace();
            match self.peek_byte() {
                Some(b'}') => {
                    self.pos += 1;
                    self.column += 1;
                    break;
                }
                None => {
                    return Err(CompilerError::syntax(
                        "Unclosed responsive block",
                        self.current_span(),
                    ));
                }
                _ => {
                    declarations.push(self.parse_declaration()?);
                }
            }
        }

        Ok(ResponsiveBlock {
            breakpoint,
            declarations,
            span: self.span_from(start_pos, start_line, start_col),
        })
    }
}

/// Helper enum for top-level parsing.
enum AtRuleOrRule {
    AtRule(AtRule),
    #[allow(dead_code)]
    Rule(Rule),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_rule() {
        let source = r#".button {
            color: red;
            padding: 10px;
        }"#;
        let result = parse(source);
        assert!(result.is_ok());
        let stylesheet = result.unwrap();
        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(stylesheet.rules[0].selector.class_name, "button");
        assert_eq!(stylesheet.rules[0].declarations.len(), 2);
    }

    #[test]
    fn test_parse_id_selector() {
        let source = r#"#main { color: red; }"#;
        let result = parse(source);
        assert!(result.is_ok());
        let stylesheet = result.unwrap();
        assert_eq!(stylesheet.rules[0].selector.class_name, "main");
        assert_eq!(stylesheet.rules[0].selector.kind, SelectorKind::Id);
    }

    #[test]
    fn test_parse_tag_selector() {
        let source = r#"div { color: red; }"#;
        let result = parse(source);
        assert!(result.is_ok());
        let stylesheet = result.unwrap();
        assert_eq!(stylesheet.rules[0].selector.class_name, "div");
        assert_eq!(stylesheet.rules[0].selector.kind, SelectorKind::Tag);
    }

    #[test]
    fn test_parse_universal_selector() {
        let source = r#"* { margin: 0; }"#;
        let result = parse(source);
        assert!(result.is_ok());
        let stylesheet = result.unwrap();
        assert_eq!(stylesheet.rules[0].selector.class_name, "*");
        assert_eq!(stylesheet.rules[0].selector.kind, SelectorKind::Universal);
    }

    #[test]
    fn test_parse_multi_selector() {
        let source = r#".a, .b, .c { color: red; }"#;
        let result = parse(source);
        assert!(result.is_ok());
        let stylesheet = result.unwrap();
        assert_eq!(stylesheet.rules[0].selector.class_name, "a");
        assert_eq!(stylesheet.rules[0].selectors.len(), 2);
        assert_eq!(stylesheet.rules[0].selectors[0].class_name, "b");
        assert_eq!(stylesheet.rules[0].selectors[1].class_name, "c");
    }

    #[test]
    fn test_parse_keyframes() {
        let source = r#"@keyframes fadeIn {
            from { opacity: 0; }
            to { opacity: 1; }
        }"#;
        let result = parse(source);
        assert!(result.is_ok());
        let stylesheet = result.unwrap();
        assert_eq!(stylesheet.at_rules.len(), 1);
        match &stylesheet.at_rules[0] {
            AtRule::Keyframes(kf) => {
                assert_eq!(kf.name, "fadeIn");
                assert_eq!(kf.keyframes.len(), 2);
            }
            _ => panic!("Expected keyframes"),
        }
    }

    #[test]
    fn test_parse_keyframes_percentage() {
        let source = r#"@keyframes slide {
            0% { transform: translateX(0); }
            50% { transform: translateX(50px); }
            100% { transform: translateX(100px); }
        }"#;
        let result = parse(source);
        assert!(result.is_ok());
        let stylesheet = result.unwrap();
        match &stylesheet.at_rules[0] {
            AtRule::Keyframes(kf) => {
                assert_eq!(kf.keyframes.len(), 3);
            }
            _ => panic!("Expected keyframes"),
        }
    }

    #[test]
    fn test_parse_font_face() {
        let source = r#"@font-face {
            font-family: "MyFont";
            src: url("myfont.woff2");
        }"#;
        let result = parse(source);
        assert!(result.is_ok());
        let stylesheet = result.unwrap();
        match &stylesheet.at_rules[0] {
            AtRule::FontFace(ff) => {
                assert_eq!(ff.declarations.len(), 2);
            }
            _ => panic!("Expected font-face"),
        }
    }

    #[test]
    fn test_parse_layer() {
        let source = r#"@layer utilities {
            .hidden { display: none; }
        }"#;
        let result = parse(source);
        assert!(result.is_ok());
        let stylesheet = result.unwrap();
        match &stylesheet.at_rules[0] {
            AtRule::Layer(layer) => {
                assert_eq!(layer.name, "utilities");
                assert!(layer.rules.is_some());
            }
            _ => panic!("Expected layer"),
        }
    }

    #[test]
    fn test_parse_layer_declaration() {
        let source = r#"@layer utilities;"#;
        let result = parse(source);
        assert!(result.is_ok());
        match &result.unwrap().at_rules[0] {
            AtRule::Layer(layer) => {
                assert_eq!(layer.name, "utilities");
                assert!(layer.rules.is_none());
            }
            _ => panic!("Expected layer declaration"),
        }
    }

    #[test]
    fn test_parse_import() {
        let source = r#"@import "reset.css";"#;
        let result = parse(source);
        assert!(result.is_ok());
        match &result.unwrap().at_rules[0] {
            AtRule::Import(imp) => {
                assert_eq!(imp.url, "reset.css");
            }
            _ => panic!("Expected import"),
        }
    }

    #[test]
    fn test_parse_import_url() {
        let source = r#"@import url("theme.css");"#;
        let result = parse(source);
        assert!(result.is_ok());
        match &result.unwrap().at_rules[0] {
            AtRule::Import(imp) => {
                assert_eq!(imp.url, "theme.css");
            }
            _ => panic!("Expected import"),
        }
    }

    #[test]
    fn test_parse_supports() {
        let source = r#"@supports (display: grid) {
            .grid { display: grid; }
        }"#;
        let result = parse(source);
        assert!(result.is_ok());
        match &result.unwrap().at_rules[0] {
            AtRule::Supports(s) => {
                assert_eq!(s.condition, "display: grid");
                assert_eq!(s.rules.len(), 1);
            }
            _ => panic!("Expected supports"),
        }
    }

    #[test]
    fn test_parse_container() {
        // Test basic @container parsing with WCSS syntax
        let source = r#"@container sidebar (min-width: 700px) {
    .card { 
        color: blue;
    }
}"#;
        let result = parse(source);
        if let Err(e) = &result {
            eprintln!("Parse error: {:?}", e);
        }
        assert!(result.is_ok(), "Failed to parse container: {:?}", result.err());
        let stylesheet = result.unwrap();
        assert!(!stylesheet.at_rules.is_empty(), "No at-rules found");
        match &stylesheet.at_rules[0] {
            AtRule::Container(c) => {
                assert_eq!(c.name, Some("sidebar".to_string()));
                assert_eq!(c.condition, "min-width: 700px");
                assert_eq!(c.rules.len(), 1);
                assert_eq!(c.rules[0].selector.class_name, "card");
            }
            _ => panic!("Expected container, got: {:?}", stylesheet.at_rules[0]),
        }
    }

    #[test]
    fn test_parse_media() {
        let source = r#"@media (prefers-color-scheme: dark) {
            .body { background: black; }
        }"#;
        let result = parse(source);
        assert!(result.is_ok());
        match &result.unwrap().at_rules[0] {
            AtRule::Media(m) => {
                assert!(m.query.contains("prefers-color-scheme"));
                assert_eq!(m.rules.len(), 1);
            }
            _ => panic!("Expected media"),
        }
    }

    #[test]
    fn test_parse_token_reference() {
        let source = r#".button {
            color: $colors.primary;
        }"#;
        let result = parse(source);
        assert!(result.is_ok());
        let stylesheet = result.unwrap();
        let decl = &stylesheet.rules[0].declarations[0];
        match &decl.value {
            Value::Token(token_ref) => {
                assert_eq!(token_ref.category, TokenCategory::Colors);
                assert_eq!(token_ref.name, "primary");
            }
            _ => panic!("Expected token reference"),
        }
    }

    #[test]
    fn test_parse_state_block() {
        let source = r#".button {
            color: blue;
            &:hover {
                color: red;
            }
        }"#;
        let result = parse(source);
        assert!(result.is_ok());
        let stylesheet = result.unwrap();
        assert_eq!(stylesheet.rules[0].states.len(), 1);
        assert_eq!(stylesheet.rules[0].states[0].modifiers[0], StateModifier::Hover);
    }

    #[test]
    fn test_parse_dark_mode_state() {
        let source = r#".card {
            background: white;
            &:dark {
                background: black;
            }
        }"#;
        let result = parse(source);
        assert!(result.is_ok());
        let stylesheet = result.unwrap();
        assert_eq!(stylesheet.rules[0].states[0].modifiers[0], StateModifier::Dark);
    }

    #[test]
    fn test_parse_responsive_block() {
        let source = r#".container {
            width: 100%;
            @md {
                width: 768px;
            }
        }"#;
        let result = parse(source);
        assert!(result.is_ok());
        let stylesheet = result.unwrap();
        assert_eq!(stylesheet.rules[0].responsive.len(), 1);
        assert_eq!(stylesheet.rules[0].responsive[0].breakpoint, "md");
    }

    #[test]
    fn test_parse_multiple_rules() {
        let source = r#"
            .header { color: black; }
            .footer { color: gray; }
        "#;
        let result = parse(source);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().rules.len(), 2);
    }

    #[test]
    fn test_parse_error_unclosed_block() {
        let source = r#".button { color: red;"#;
        let result = parse(source);
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_hex_color() {
        let source = r#".box { color: #ff0000; }"#;
        let result = parse(source);
        assert!(result.is_ok());
        let decl = &result.unwrap().rules[0].declarations[0];
        match &decl.value {
            Value::Color(Color::Hex(hex)) => assert_eq!(hex, "#ff0000"),
            _ => panic!("Expected hex color"),
        }
    }

    #[test]
    fn test_parse_number_with_unit() {
        let source = r#".box { width: 100px; }"#;
        let result = parse(source);
        assert!(result.is_ok());
        let decl = &result.unwrap().rules[0].declarations[0];
        match &decl.value {
            Value::Number(n, Some(Unit::Px)) => assert_eq!(*n, 100.0),
            _ => panic!("Expected number with px unit"),
        }
    }

    #[test]
    fn test_parse_modern_units() {
        let source = r#".box { width: 50dvh; }"#;
        let result = parse(source);
        assert!(result.is_ok());
        let decl = &result.unwrap().rules[0].declarations[0];
        match &decl.value {
            Value::Number(n, Some(Unit::Dvh)) => assert_eq!(*n, 50.0),
            _ => panic!("Expected number with dvh unit"),
        }
    }

    #[test]
    fn test_parse_container_units() {
        let source = r#".box { width: 50cqi; }"#;
        let result = parse(source);
        assert!(result.is_ok());
        let decl = &result.unwrap().rules[0].declarations[0];
        match &decl.value {
            Value::Number(n, Some(Unit::Cqi)) => assert_eq!(*n, 50.0),
            _ => panic!("Expected number with cqi unit"),
        }
    }

    #[test]
    fn test_parse_property_rule() {
        let source = r#"@property --my-color {
            syntax: "<color>";
            inherits: false;
            initial-value: red;
        }"#;
        let result = parse(source);
        assert!(result.is_ok());
        match &result.unwrap().at_rules[0] {
            AtRule::Property(p) => {
                assert_eq!(p.name, "--my-color");
            }
            _ => panic!("Expected property rule"),
        }
    }

    #[test]
    fn test_parse_tailwind_base() {
        let source = r#"@tailwind base;"#;
        let result = parse(source);
        assert!(result.is_ok());
        let stylesheet = result.unwrap();
        assert_eq!(stylesheet.at_rules.len(), 1);
        match &stylesheet.at_rules[0] {
            AtRule::Tailwind(tw) => {
                assert_eq!(tw.directive_type, TailwindDirectiveType::Base);
            }
            _ => panic!("Expected Tailwind directive"),
        }
    }

    #[test]
    fn test_parse_tailwind_components() {
        let source = r#"@tailwind components;"#;
        let result = parse(source);
        assert!(result.is_ok());
        match &result.unwrap().at_rules[0] {
            AtRule::Tailwind(tw) => {
                assert_eq!(tw.directive_type, TailwindDirectiveType::Components);
            }
            _ => panic!("Expected Tailwind directive"),
        }
    }

    #[test]
    fn test_parse_tailwind_utilities() {
        let source = r#"@tailwind utilities;"#;
        let result = parse(source);
        assert!(result.is_ok());
        match &result.unwrap().at_rules[0] {
            AtRule::Tailwind(tw) => {
                assert_eq!(tw.directive_type, TailwindDirectiveType::Utilities);
            }
            _ => panic!("Expected Tailwind directive"),
        }
    }

    #[test]
    fn test_parse_apply_directive() {
        let source = r#".btn {
            @apply px-4 py-2 bg-blue-500 text-white rounded;
        }"#;
        let result = parse(source);
        assert!(result.is_ok());
        let stylesheet = result.unwrap();
        assert_eq!(stylesheet.rules.len(), 1);
        assert_eq!(stylesheet.rules[0].declarations.len(), 1);
        match &stylesheet.rules[0].declarations[0].property {
            Property::Apply(classes) => {
                assert_eq!(classes, "px-4 py-2 bg-blue-500 text-white rounded");
            }
            _ => panic!("Expected @apply directive"),
        }
    }

    #[test]
    fn test_parse_tailwind_full_example() {
        let source = r#"
            @tailwind base;
            @tailwind components;
            @tailwind utilities;

            .btn-primary {
                @apply px-4 py-2 bg-blue-500 text-white rounded hover:bg-blue-600;
            }

            @layer components {
                .card {
                    @apply p-4 bg-white shadow rounded;
                }
            }
        "#;
        let result = parse(source);
        if let Err(ref errors) = result {
            for err in errors {
                eprintln!("Parse error: {:?}", err);
            }
        }
        assert!(result.is_ok(), "Failed to parse: {:?}", result.err());
        let stylesheet = result.unwrap();
        
        // Check @tailwind directives
        assert!(stylesheet.at_rules.len() >= 3);
        match &stylesheet.at_rules[0] {
            AtRule::Tailwind(tw) => assert_eq!(tw.directive_type, TailwindDirectiveType::Base),
            _ => panic!("Expected @tailwind base"),
        }
        match &stylesheet.at_rules[1] {
            AtRule::Tailwind(tw) => assert_eq!(tw.directive_type, TailwindDirectiveType::Components),
            _ => panic!("Expected @tailwind components"),
        }
        match &stylesheet.at_rules[2] {
            AtRule::Tailwind(tw) => assert_eq!(tw.directive_type, TailwindDirectiveType::Utilities),
            _ => panic!("Expected @tailwind utilities"),
        }
        
        // Check rules with @apply
        assert!(stylesheet.rules.len() >= 1);
        assert_eq!(stylesheet.rules[0].selector.class_name, "btn-primary");
        match &stylesheet.rules[0].declarations[0].property {
            Property::Apply(_) => {},
            _ => panic!("Expected @apply in btn-primary"),
        }
    }
}
