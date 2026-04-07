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

    /// Read a CSS identifier that may start with -- (custom properties).
    fn read_css_identifier(&mut self) -> Option<String> {
        let start = self.pos;
        // Allow leading --
        if self.pos + 1 < self.bytes.len()
            && self.bytes[self.pos] == b'-'
            && self.bytes[self.pos + 1] == b'-'
        {
            self.pos += 2;
            self.column += 2;
        }
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

    /// Read everything until a closing brace, respecting nesting.
    fn read_until_closing_brace(&mut self) -> String {
        let start = self.pos;
        let mut depth = 1;
        while self.pos < self.bytes.len() && depth > 0 {
            match self.bytes[self.pos] {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 { break; }
                }
                b'\n' => { self.line += 1; self.column = 0; }
                _ => {}
            }
            self.pos += 1;
            self.column += 1;
        }
        self.source[start..self.pos].trim().to_string()
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
        let name = self.read_identifier().unwrap_or_default();
        self.skip_whitespace();

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
                    responsive.push(self.parse_responsive_block()?);
                }
                Some(b'&') => {
                    states.push(self.parse_state_block()?);
                }
                // CSS nesting: nested rules starting with selectors
                Some(b'.') | Some(b'#') | Some(b'*') => {
                    // Check if this looks like a nested rule (has { after selector)
                    let saved = self.pos;
                    let saved_line = self.line;
                    let saved_col = self.column;

                    // Try to parse as nested rule
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
                    declarations.push(self.parse_declaration()?);
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
            self.skip_whitespace();
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
            Some(b'#') => self.parse_color_hex(),
            Some(b'"') | Some(b'\'') => {
                let s = self.read_string()?;
                Ok(Value::Literal(s))
            }
            Some(b) if b.is_ascii_digit() || b == b'-' => {
                let num = self.read_number();
                let unit = self.try_read_unit();
                Ok(Value::Number(num, unit))
            }
            _ => {
                // Check for var() or env()
                if self.pos + 4 <= self.bytes.len() && &self.bytes[self.pos..self.pos+4] == b"var(" {
                    return self.parse_var_function();
                }
                if self.pos + 4 <= self.bytes.len() && &self.bytes[self.pos..self.pos+4] == b"env(" {
                    return self.parse_env_function();
                }

                // Read until semicolon, closing brace, or newline
                let start = self.pos;
                while self.pos < self.bytes.len() {
                    match self.bytes[self.pos] {
                        b';' | b'}' | b'\n' | b'!' => break,
                        _ => {
                            self.pos += 1;
                            self.column += 1;
                        }
                    }
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
}
