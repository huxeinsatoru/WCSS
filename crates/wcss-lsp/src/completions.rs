use tower_lsp::lsp_types::*;

/// Provide completion items based on the current line text and cursor position.
pub fn get_completions(line_text: &str, position: Position) -> Vec<CompletionItem> {
    let col = position.character as usize;
    let prefix = if col <= line_text.len() {
        &line_text[..col]
    } else {
        line_text
    };
    let trimmed = prefix.trim();

    // At-rule completions: line starts with @
    if trimmed.starts_with('@') {
        return at_rule_completions(trimmed);
    }

    // Pseudo-class completions: cursor is right after ":"
    // but only when NOT inside a declaration (no property context)
    if trimmed.ends_with(':') && !trimmed.contains('{') && !looks_like_property_context(trimmed) {
        return pseudo_class_completions();
    }

    // Value completions: we are after "property:" inside a block
    if let Some(property) = extract_property_context(trimmed) {
        return value_completions(property);
    }

    // Default: property completions (we are likely inside a rule block)
    property_completions(trimmed)
}

/// Check whether the trimmed prefix looks like a property-value context (e.g. "color:").
fn looks_like_property_context(trimmed: &str) -> bool {
    // A property context is something like "  color:" or "display:"
    // It does NOT contain a selector-like pattern before the colon.
    let before_colon = trimmed.trim_end_matches(':');
    let word = before_colon.trim();
    // If the word before : is a known CSS property, treat it as a property context.
    PROPERTIES.iter().any(|p| p.0 == word)
}

/// Extract the property name if the cursor is in a value context (after "property:").
fn extract_property_context(trimmed: &str) -> Option<&str> {
    // Look for "property:" or "property: value..." pattern
    let colon_pos = trimmed.rfind(':')?;
    let before_colon = trimmed[..colon_pos].trim();
    // Get the last word before the colon (the property name)
    let prop = before_colon.split_whitespace().last()?;
    // Only treat as property context if it looks like a CSS property
    if PROPERTIES.iter().any(|p| p.0 == prop) || prop.starts_with("--") {
        Some(prop)
    } else {
        None
    }
}

// ---------------------------------------------------------------------------
// Property completions
// ---------------------------------------------------------------------------

/// Top 50+ CSS properties with short descriptions.
const PROPERTIES: &[(&str, &str)] = &[
    ("display", "Sets the display type of an element"),
    ("margin", "Sets the margin on all four sides"),
    ("margin-top", "Sets the top margin"),
    ("margin-right", "Sets the right margin"),
    ("margin-bottom", "Sets the bottom margin"),
    ("margin-left", "Sets the left margin"),
    ("padding", "Sets the padding on all four sides"),
    ("padding-top", "Sets the top padding"),
    ("padding-right", "Sets the right padding"),
    ("padding-bottom", "Sets the bottom padding"),
    ("padding-left", "Sets the left padding"),
    ("color", "Sets the text color"),
    ("background", "Shorthand for all background properties"),
    ("background-color", "Sets the background color"),
    ("background-image", "Sets a background image"),
    ("background-position", "Sets the background position"),
    ("background-size", "Sets the background size"),
    ("background-repeat", "Sets how background images repeat"),
    ("width", "Sets the element width"),
    ("height", "Sets the element height"),
    ("min-width", "Sets the minimum width"),
    ("max-width", "Sets the maximum width"),
    ("min-height", "Sets the minimum height"),
    ("max-height", "Sets the maximum height"),
    ("font-size", "Sets the font size"),
    ("font-weight", "Sets the font weight (boldness)"),
    ("font-family", "Sets the font family"),
    ("font-style", "Sets the font style (italic, normal)"),
    ("line-height", "Sets the line height"),
    ("letter-spacing", "Sets spacing between characters"),
    ("text-align", "Sets horizontal text alignment"),
    ("text-decoration", "Sets text decoration (underline, etc.)"),
    ("text-transform", "Sets text capitalization"),
    ("border", "Shorthand for border width, style, color"),
    ("border-radius", "Rounds the corners of an element"),
    ("border-color", "Sets the border color"),
    ("border-width", "Sets the border width"),
    ("border-style", "Sets the border style"),
    ("position", "Sets positioning method"),
    ("top", "Sets top offset for positioned elements"),
    ("right", "Sets right offset for positioned elements"),
    ("bottom", "Sets bottom offset for positioned elements"),
    ("left", "Sets left offset for positioned elements"),
    ("z-index", "Sets the stack order"),
    ("overflow", "Sets overflow behavior"),
    ("overflow-x", "Sets horizontal overflow"),
    ("overflow-y", "Sets vertical overflow"),
    ("flex", "Shorthand for flex-grow, shrink, basis"),
    ("flex-direction", "Sets the flex container direction"),
    ("flex-wrap", "Sets whether flex items wrap"),
    ("flex-grow", "Sets the flex grow factor"),
    ("flex-shrink", "Sets the flex shrink factor"),
    ("flex-basis", "Sets the initial main size"),
    ("justify-content", "Aligns items along the main axis"),
    ("align-items", "Aligns items along the cross axis"),
    ("align-self", "Aligns a single item on the cross axis"),
    ("align-content", "Aligns flex lines"),
    ("gap", "Sets gap between grid/flex items"),
    ("row-gap", "Sets gap between rows"),
    ("column-gap", "Sets gap between columns"),
    ("grid-template-columns", "Defines grid column tracks"),
    ("grid-template-rows", "Defines grid row tracks"),
    ("grid-column", "Sets grid column placement"),
    ("grid-row", "Sets grid row placement"),
    ("grid-area", "Sets grid area placement"),
    ("grid-template-areas", "Defines named grid areas"),
    ("transition", "Shorthand for transition properties"),
    ("transition-property", "Sets the property to transition"),
    ("transition-duration", "Sets the transition duration"),
    ("transition-timing-function", "Sets the transition easing"),
    ("transition-delay", "Sets the transition delay"),
    ("transform", "Applies a 2D/3D transformation"),
    ("opacity", "Sets the opacity level"),
    ("visibility", "Sets element visibility"),
    ("box-shadow", "Adds shadow effects"),
    ("cursor", "Sets the mouse cursor type"),
    ("animation", "Shorthand for animation properties"),
    ("animation-name", "Sets the animation keyframes name"),
    ("animation-duration", "Sets the animation duration"),
    ("white-space", "Sets how white space is handled"),
    ("word-break", "Sets word breaking rules"),
    ("box-sizing", "Sets the box model calculation"),
    ("outline", "Shorthand for outline properties"),
    ("content", "Sets generated content"),
    ("pointer-events", "Sets whether element receives pointer events"),
    ("user-select", "Controls text selection"),
    ("container-type", "Establishes a query container"),
    ("container-name", "Names a query container"),
    ("aspect-ratio", "Sets a preferred aspect ratio"),
    ("object-fit", "Sets how replaced content is fitted"),
    ("object-position", "Sets position of replaced content"),
    ("filter", "Applies graphical filters"),
    ("backdrop-filter", "Applies filters behind an element"),
    ("mix-blend-mode", "Sets how content blends with parent"),
    ("clip-path", "Clips an element to a shape"),
    ("place-items", "Shorthand for align-items and justify-items"),
    ("place-content", "Shorthand for align-content and justify-content"),
    ("inset", "Shorthand for top, right, bottom, left"),
    ("accent-color", "Sets the accent color for form controls"),
    ("scroll-behavior", "Sets smooth or instant scrolling"),
];

fn property_completions(prefix: &str) -> Vec<CompletionItem> {
    let filter = prefix.trim().trim_start_matches(|c: char| !c.is_ascii_alphanumeric() && c != '-');

    PROPERTIES
        .iter()
        .filter(|(name, _)| filter.is_empty() || name.starts_with(filter))
        .enumerate()
        .map(|(i, (name, detail))| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::PROPERTY),
            detail: Some(detail.to_string()),
            insert_text: Some(format!("{name}: ")),
            insert_text_format: Some(InsertTextFormat::PLAIN_TEXT),
            sort_text: Some(format!("{:04}", i)),
            ..Default::default()
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Value completions
// ---------------------------------------------------------------------------

fn value_completions(property: &str) -> Vec<CompletionItem> {
    let values: &[&str] = match property {
        "display" => &[
            "none", "block", "inline", "inline-block", "flex", "inline-flex",
            "grid", "inline-grid", "contents", "flow-root", "table",
            "table-row", "table-cell", "list-item",
        ],
        "position" => &["static", "relative", "absolute", "fixed", "sticky"],
        "overflow" | "overflow-x" | "overflow-y" => {
            &["visible", "hidden", "clip", "scroll", "auto"]
        }
        "flex-direction" => &["row", "row-reverse", "column", "column-reverse"],
        "flex-wrap" => &["nowrap", "wrap", "wrap-reverse"],
        "justify-content" => &[
            "flex-start", "flex-end", "center", "space-between", "space-around",
            "space-evenly", "start", "end",
        ],
        "align-items" | "align-self" => &[
            "stretch", "flex-start", "flex-end", "center", "baseline",
            "start", "end",
        ],
        "align-content" => &[
            "stretch", "flex-start", "flex-end", "center", "space-between",
            "space-around", "space-evenly",
        ],
        "text-align" => &["left", "right", "center", "justify", "start", "end"],
        "text-transform" => &["none", "uppercase", "lowercase", "capitalize", "full-width"],
        "text-decoration" => &["none", "underline", "overline", "line-through"],
        "font-weight" => &[
            "normal", "bold", "bolder", "lighter",
            "100", "200", "300", "400", "500", "600", "700", "800", "900",
        ],
        "font-style" => &["normal", "italic", "oblique"],
        "visibility" => &["visible", "hidden", "collapse"],
        "cursor" => &[
            "auto", "default", "pointer", "move", "text", "wait", "help",
            "not-allowed", "crosshair", "grab", "grabbing", "zoom-in",
            "zoom-out", "col-resize", "row-resize",
        ],
        "white-space" => &["normal", "nowrap", "pre", "pre-wrap", "pre-line", "break-spaces"],
        "word-break" => &["normal", "break-all", "keep-all", "break-word"],
        "box-sizing" => &["content-box", "border-box"],
        "border-style" => &[
            "none", "solid", "dashed", "dotted", "double", "groove",
            "ridge", "inset", "outset",
        ],
        "object-fit" => &["fill", "contain", "cover", "none", "scale-down"],
        "container-type" => &["normal", "size", "inline-size"],
        "scroll-behavior" => &["auto", "smooth"],
        "pointer-events" => &["auto", "none"],
        "user-select" => &["auto", "none", "text", "all"],
        "mix-blend-mode" => &[
            "normal", "multiply", "screen", "overlay", "darken", "lighten",
            "color-dodge", "color-burn", "hard-light", "soft-light",
            "difference", "exclusion", "hue", "saturation", "color", "luminosity",
        ],
        "transition-timing-function" | "animation-timing-function" => &[
            "ease", "linear", "ease-in", "ease-out", "ease-in-out",
            "step-start", "step-end",
        ],
        "background-repeat" => &["repeat", "repeat-x", "repeat-y", "no-repeat", "space", "round"],
        "background-size" => &["auto", "cover", "contain"],
        "background-attachment" => &["scroll", "fixed", "local"],
        "color" | "background-color" | "border-color" | "accent-color" => &[
            "inherit", "initial", "unset", "currentColor", "transparent",
            "red", "green", "blue", "white", "black",
        ],
        _ => &["inherit", "initial", "unset", "revert", "revert-layer"],
    };

    values
        .iter()
        .enumerate()
        .map(|(i, val)| CompletionItem {
            label: val.to_string(),
            kind: Some(CompletionItemKind::VALUE),
            detail: Some(format!("Value for {property}")),
            sort_text: Some(format!("{:04}", i)),
            ..Default::default()
        })
        .collect()
}

// ---------------------------------------------------------------------------
// At-rule completions
// ---------------------------------------------------------------------------

const AT_RULES: &[(&str, &str, &str)] = &[
    ("@media", "Media query", "@media ($1) {\n\t$0\n}"),
    ("@keyframes", "Keyframes animation", "@keyframes $1 {\n\tfrom { $2 }\n\tto { $0 }\n}"),
    ("@layer", "Cascade layer", "@layer $1 {\n\t$0\n}"),
    ("@container", "Container query", "@container $1 ($2) {\n\t$0\n}"),
    ("@supports", "Feature query", "@supports ($1) {\n\t$0\n}"),
    ("@font-face", "Font face declaration", "@font-face {\n\tfont-family: $1;\n\tsrc: url($0);\n}"),
    ("@import", "Import stylesheet", "@import url(\"$0\");"),
    ("@property", "Custom property definition", "@property --$1 {\n\tsyntax: \"$2\";\n\tinherits: false;\n\tinitial-value: $0;\n}"),
    ("@charset", "Character encoding", "@charset \"UTF-8\";"),
    ("@namespace", "Namespace declaration", "@namespace $0;"),
];

fn at_rule_completions(prefix: &str) -> Vec<CompletionItem> {
    AT_RULES
        .iter()
        .filter(|(name, _, _)| name.starts_with(prefix))
        .enumerate()
        .map(|(i, (name, detail, snippet))| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            detail: Some(detail.to_string()),
            insert_text: Some(snippet.to_string()),
            insert_text_format: Some(InsertTextFormat::SNIPPET),
            sort_text: Some(format!("{:04}", i)),
            ..Default::default()
        })
        .collect()
}

// ---------------------------------------------------------------------------
// Pseudo-class completions
// ---------------------------------------------------------------------------

const PSEUDO_CLASSES: &[(&str, &str)] = &[
    (":hover", "Mouse pointer is over the element"),
    (":focus", "Element has focus"),
    (":focus-visible", "Element has focus and focus should be visible"),
    (":focus-within", "Element or its descendant has focus"),
    (":active", "Element is being activated (clicked)"),
    (":visited", "Link has been visited"),
    (":link", "Unvisited link"),
    (":disabled", "Form element is disabled"),
    (":enabled", "Form element is enabled"),
    (":checked", "Checkbox/radio is checked"),
    (":indeterminate", "Element is in indeterminate state"),
    (":required", "Form element is required"),
    (":optional", "Form element is optional"),
    (":valid", "Form element value is valid"),
    (":invalid", "Form element value is invalid"),
    (":read-only", "Element is not editable"),
    (":read-write", "Element is editable"),
    (":placeholder-shown", "Placeholder text is visible"),
    (":default", "Default form element"),
    (":first-child", "First child of its parent"),
    (":last-child", "Last child of its parent"),
    (":only-child", "Only child of its parent"),
    (":first-of-type", "First of its type among siblings"),
    (":last-of-type", "Last of its type among siblings"),
    (":only-of-type", "Only of its type among siblings"),
    (":empty", "Element has no children"),
    (":root", "Document root element"),
    (":nth-child()", "Element at position An+B"),
    (":nth-last-child()", "Element at position An+B from end"),
    (":nth-of-type()", "Element of type at position An+B"),
    (":nth-last-of-type()", "Element of type at position An+B from end"),
    (":not()", "Elements that do not match a selector"),
    (":is()", "Matches any of the given selectors"),
    (":where()", "Like :is() but with zero specificity"),
    (":has()", "Parent selector (relational pseudo-class)"),
    (":lang()", "Element matches a language"),
    (":dir()", "Element matches a text direction"),
    ("::before", "Inserts content before the element"),
    ("::after", "Inserts content after the element"),
    ("::first-line", "First line of a block element"),
    ("::first-letter", "First letter of a block element"),
    ("::placeholder", "Placeholder text of a form element"),
    ("::selection", "Portion selected by the user"),
    ("::marker", "Marker of a list item"),
    ("::backdrop", "Backdrop of a fullscreen element"),
    (":dark", "WCSS dark mode shorthand (prefers-color-scheme: dark)"),
];

fn pseudo_class_completions() -> Vec<CompletionItem> {
    PSEUDO_CLASSES
        .iter()
        .enumerate()
        .map(|(i, (name, detail))| CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(detail.to_string()),
            sort_text: Some(format!("{:04}", i)),
            ..Default::default()
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_property_completions() {
        let items = get_completions("  dis", Position::new(0, 5));
        assert!(items.iter().any(|c| c.label == "display"));
    }

    #[test]
    fn test_value_completions_display() {
        let items = get_completions("  display: ", Position::new(0, 11));
        assert!(items.iter().any(|c| c.label == "flex"));
        assert!(items.iter().any(|c| c.label == "grid"));
    }

    #[test]
    fn test_at_rule_completions() {
        let items = get_completions("@m", Position::new(0, 2));
        assert!(items.iter().any(|c| c.label == "@media"));
    }

    #[test]
    fn test_pseudo_class_completions() {
        let items = get_completions(".btn:", Position::new(0, 5));
        assert!(items.iter().any(|c| c.label == ":hover"));
    }
}
