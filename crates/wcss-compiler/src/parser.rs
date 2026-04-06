use crate::ast::*;
use crate::error::CompilerError;

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
                                // Line comment — skip to end of line
                                self.pos += 2;
                                self.column += 2;
                                while self.pos < self.bytes.len() && self.bytes[self.pos] != b'\n' {
                                    self.pos += 1;
                                }
                            }
                            b'*' => {
                                // Block comment — skip to */
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
        // Fast-path: parse integer part directly
        let mut int_val: i64 = 0;
        while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
            int_val = int_val * 10 + (self.bytes[self.pos] - b'0') as i64;
            self.pos += 1;
            self.column += 1;
        }
        if self.pos < self.bytes.len() && self.bytes[self.pos] == b'.' {
            self.pos += 1;
            self.column += 1;
            // Has decimal part — fall back to f64 parse
            while self.pos < self.bytes.len() && self.bytes[self.pos].is_ascii_digit() {
                self.pos += 1;
                self.column += 1;
            }
            self.source[start..self.pos].parse().unwrap_or(0.0)
        } else {
            // Pure integer — no f64 parsing needed
            if negative { -(int_val as f64) } else { int_val as f64 }
        }
    }

    fn parse_stylesheet(&mut self) -> Result<StyleSheet, Vec<CompilerError>> {
        let start_pos = self.pos;
        let start_line = self.line;
        let start_col = self.column;
        // Pre-allocate based on rough estimate: ~80 bytes per rule in input
        let estimated_rules = self.bytes.len() / 80;
        let mut rules = Vec::with_capacity(estimated_rules);

        loop {
            self.skip_whitespace();
            if self.pos >= self.bytes.len() {
                break;
            }

            match self.parse_rule() {
                Ok(rule) => rules.push(rule),
                Err(err) => {
                    self.errors.push(err);
                    self.recover_to_next_rule();
                }
            }
        }

        if !self.errors.is_empty() {
            return Err(std::mem::take(&mut self.errors));
        }

        Ok(StyleSheet {
            rules,
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

    fn parse_rule(&mut self) -> Result<Rule, CompilerError> {
        let start_pos = self.pos;
        let start_line = self.line;
        let start_col = self.column;

        let selector = self.parse_selector()?;

        self.skip_whitespace();
        self.expect('{')?;

        let mut declarations = Vec::new();
        let mut states = Vec::new();
        let mut responsive = Vec::new();

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
                _ => {
                    declarations.push(self.parse_declaration()?);
                }
            }
        }

        Ok(Rule {
            selector,
            declarations,
            states,
            responsive,
            span: self.span_from(start_pos, start_line, start_col),
        })
    }

    fn parse_selector(&mut self) -> Result<Selector, CompilerError> {
        self.skip_whitespace();
        let start_pos = self.pos;
        let start_line = self.line;
        let start_col = self.column;

        if self.peek_byte() == Some(b'.') {
            self.pos += 1;
            self.column += 1;
        }

        let class_name = self.read_identifier().ok_or_else(|| {
            CompilerError::syntax("Expected class name", self.current_span())
        })?;

        let mut combinators = Vec::new();
        let mut pseudo_elements = Vec::new();

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
                Some(b':') if self.pos + 1 < self.bytes.len() && self.bytes[self.pos + 1] == b':' => {
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
                        _ => PseudoElement::Custom(name),
                    };
                    pseudo_elements.push(pe);
                }
                _ => break,
            }
        }

        Ok(Selector {
            class_name,
            combinators,
            pseudo_elements,
            span: self.span_from(start_pos, start_line, start_col),
        })
    }

    fn parse_declaration(&mut self) -> Result<Declaration, CompilerError> {
        let start_pos = self.pos;
        let start_line = self.line;
        let start_col = self.column;

        let prop_name = self.read_identifier().ok_or_else(|| {
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
                // Trim trailing whitespace efficiently
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
                format!("Unknown token category '{category_name}'. Expected: colors, spacing, typography, breakpoints"),
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
}
