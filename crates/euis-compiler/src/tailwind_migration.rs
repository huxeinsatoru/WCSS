//! Tailwind CSS to Euis migration tool.
//!
//! Converts Tailwind CSS configurations and utility classes to Euis format,
//! making it easy to migrate existing Tailwind projects.

use std::collections::HashMap;

// ---------------------------------------------------------------------------
// Data structures
// ---------------------------------------------------------------------------

/// Represents a parsed Tailwind configuration (simplified).
#[derive(Debug)]
pub struct TailwindConfig {
    pub colors: HashMap<String, String>,
    pub spacing: HashMap<String, String>,
    pub font_family: HashMap<String, Vec<String>>,
    pub font_size: HashMap<String, String>,
    pub border_radius: HashMap<String, String>,
    pub breakpoints: HashMap<String, String>,
    pub box_shadow: HashMap<String, String>,
    pub opacity: HashMap<String, String>,
    pub z_index: HashMap<String, String>,
}

impl Default for TailwindConfig {
    fn default() -> Self {
        Self {
            colors: HashMap::new(),
            spacing: HashMap::new(),
            font_family: HashMap::new(),
            font_size: HashMap::new(),
            border_radius: HashMap::new(),
            breakpoints: HashMap::new(),
            box_shadow: HashMap::new(),
            opacity: HashMap::new(),
            z_index: HashMap::new(),
        }
    }
}

/// Migration result containing converted config and any warnings.
pub struct MigrationResult {
    pub euis_config: String,
    pub converted_files: Vec<ConvertedFile>,
    pub warnings: Vec<MigrationWarning>,
    pub stats: MigrationStats,
}

pub struct ConvertedFile {
    pub original_path: String,
    pub content: String,
}

pub struct MigrationWarning {
    pub message: String,
    pub line: Option<usize>,
    pub suggestion: Option<String>,
}

pub struct MigrationStats {
    pub classes_converted: usize,
    pub directives_converted: usize,
    pub manual_review_needed: usize,
}

// ---------------------------------------------------------------------------
// 1. Parse Tailwind config (JSON representation)
// ---------------------------------------------------------------------------

/// Parse a JSON representation of a Tailwind config into [`TailwindConfig`].
///
/// The expected JSON shape mirrors `tailwind.config.js`'s `theme` key:
///
/// ```json
/// {
///   "theme": {
///     "colors": { "primary": "#3b82f6" },
///     "spacing": { "1": "0.25rem" },
///     "fontFamily": { "sans": ["Inter", "sans-serif"] },
///     "fontSize": { "sm": "0.875rem" },
///     "borderRadius": { "lg": "0.5rem" },
///     "screens": { "sm": "640px" },
///     "boxShadow": { "md": "0 4px 6px -1px rgba(0,0,0,.1)" },
///     "opacity": { "50": "0.5" },
///     "zIndex": { "10": "10" }
///   }
/// }
/// ```
pub fn parse_tailwind_config(json_content: &str) -> Result<TailwindConfig, String> {
    let root: serde_json::Value =
        serde_json::from_str(json_content).map_err(|e| format!("Invalid JSON: {e}"))?;

    let theme = root
        .get("theme")
        .ok_or_else(|| "Missing \"theme\" key in config".to_string())?;

    let mut config = TailwindConfig::default();

    // colors
    if let Some(obj) = theme.get("colors").and_then(|v| v.as_object()) {
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                config.colors.insert(k.clone(), s.to_string());
            } else if let Some(nested) = v.as_object() {
                // Tailwind nested color objects like colors.blue.500
                for (shade, sv) in nested {
                    if let Some(s) = sv.as_str() {
                        config
                            .colors
                            .insert(format!("{k}-{shade}"), s.to_string());
                    }
                }
            }
        }
    }

    // spacing
    if let Some(obj) = theme.get("spacing").and_then(|v| v.as_object()) {
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                config.spacing.insert(k.clone(), s.to_string());
            }
        }
    }

    // fontFamily
    if let Some(obj) = theme.get("fontFamily").and_then(|v| v.as_object()) {
        for (k, v) in obj {
            if let Some(arr) = v.as_array() {
                let fonts: Vec<String> = arr
                    .iter()
                    .filter_map(|item| item.as_str().map(|s| s.to_string()))
                    .collect();
                config.font_family.insert(k.clone(), fonts);
            }
        }
    }

    // fontSize
    if let Some(obj) = theme.get("fontSize").and_then(|v| v.as_object()) {
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                config.font_size.insert(k.clone(), s.to_string());
            }
        }
    }

    // borderRadius
    if let Some(obj) = theme.get("borderRadius").and_then(|v| v.as_object()) {
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                config.border_radius.insert(k.clone(), s.to_string());
            }
        }
    }

    // screens → breakpoints
    if let Some(obj) = theme.get("screens").and_then(|v| v.as_object()) {
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                config.breakpoints.insert(k.clone(), s.to_string());
            }
        }
    }

    // boxShadow
    if let Some(obj) = theme.get("boxShadow").and_then(|v| v.as_object()) {
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                config.box_shadow.insert(k.clone(), s.to_string());
            }
        }
    }

    // opacity
    if let Some(obj) = theme.get("opacity").and_then(|v| v.as_object()) {
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                config.opacity.insert(k.clone(), s.to_string());
            }
        }
    }

    // zIndex
    if let Some(obj) = theme.get("zIndex").and_then(|v| v.as_object()) {
        for (k, v) in obj {
            if let Some(s) = v.as_str() {
                config.z_index.insert(k.clone(), s.to_string());
            }
        }
    }

    Ok(config)
}

// ---------------------------------------------------------------------------
// 2. Generate Euis config from TailwindConfig
// ---------------------------------------------------------------------------

/// Generate a Euis design-token configuration string from a [`TailwindConfig`].
///
/// The output uses CSS custom properties grouped inside a `:root` rule, which
/// is the idiomatic Euis way of defining design tokens.
pub fn generate_euis_config(tw_config: &TailwindConfig) -> String {
    let mut out = String::from("/* Euis Design Tokens – migrated from Tailwind */\n\n:root {\n");

    // Helper closure to write a section.
    fn write_section(
        out: &mut String,
        comment: &str,
        prefix: &str,
        map: &HashMap<String, String>,
    ) {
        if map.is_empty() {
            return;
        }
        out.push_str(&format!("  /* {comment} */\n"));
        let mut keys: Vec<&String> = map.keys().collect();
        keys.sort();
        for key in keys {
            let val = &map[key];
            out.push_str(&format!("  --{prefix}-{key}: {val};\n"));
        }
        out.push('\n');
    }

    write_section(&mut out, "Colors", "color", &tw_config.colors);
    write_section(&mut out, "Spacing", "spacing", &tw_config.spacing);
    write_section(&mut out, "Font sizes", "font-size", &tw_config.font_size);
    write_section(
        &mut out,
        "Border radius",
        "radius",
        &tw_config.border_radius,
    );
    write_section(&mut out, "Box shadows", "shadow", &tw_config.box_shadow);
    write_section(&mut out, "Opacity", "opacity", &tw_config.opacity);
    write_section(&mut out, "Z-index", "z", &tw_config.z_index);

    // Font families need special handling (value is Vec<String>).
    if !tw_config.font_family.is_empty() {
        out.push_str("  /* Font families */\n");
        let mut keys: Vec<&String> = tw_config.font_family.keys().collect();
        keys.sort();
        for key in keys {
            let fonts = &tw_config.font_family[key];
            let value = fonts.join(", ");
            out.push_str(&format!("  --font-{key}: {value};\n"));
        }
        out.push('\n');
    }

    // Breakpoints as custom media (outside :root).
    let breakpoints_block = if !tw_config.breakpoints.is_empty() {
        let mut bp = String::new();
        let mut keys: Vec<&String> = tw_config.breakpoints.keys().collect();
        keys.sort();
        for key in keys {
            let val = &tw_config.breakpoints[key];
            bp.push_str(&format!(
                "@custom-media --screen-{key} (min-width: {val});\n"
            ));
        }
        bp
    } else {
        String::new()
    };

    out.push_str("}\n");

    if !breakpoints_block.is_empty() {
        out.push('\n');
        out.push_str("/* Breakpoints */\n");
        out.push_str(&breakpoints_block);
    }

    out
}

// ---------------------------------------------------------------------------
// 3. Map a single Tailwind utility class to its CSS equivalent
// ---------------------------------------------------------------------------

/// Map a single Tailwind utility class to a CSS `property: value` string.
///
/// Returns `None` when the class is not recognised.
pub fn map_utility_class(tw_class: &str) -> Option<String> {
    // ---- static one-to-one mappings ----
    let static_map: HashMap<&str, &str> = HashMap::from([
        // Layout / display
        ("flex", "display: flex"),
        ("inline-flex", "display: inline-flex"),
        ("grid", "display: grid"),
        ("inline-grid", "display: inline-grid"),
        ("block", "display: block"),
        ("inline-block", "display: inline-block"),
        ("inline", "display: inline"),
        ("hidden", "display: none"),
        ("table", "display: table"),
        ("contents", "display: contents"),
        // Position
        ("absolute", "position: absolute"),
        ("relative", "position: relative"),
        ("fixed", "position: fixed"),
        ("sticky", "position: sticky"),
        ("static", "position: static"),
        // Flexbox utilities
        ("flex-row", "flex-direction: row"),
        ("flex-col", "flex-direction: column"),
        ("flex-row-reverse", "flex-direction: row-reverse"),
        ("flex-col-reverse", "flex-direction: column-reverse"),
        ("flex-wrap", "flex-wrap: wrap"),
        ("flex-nowrap", "flex-wrap: nowrap"),
        ("flex-wrap-reverse", "flex-wrap: wrap-reverse"),
        ("flex-1", "flex: 1 1 0%"),
        ("flex-auto", "flex: 1 1 auto"),
        ("flex-initial", "flex: 0 1 auto"),
        ("flex-none", "flex: none"),
        ("grow", "flex-grow: 1"),
        ("grow-0", "flex-grow: 0"),
        ("shrink", "flex-shrink: 1"),
        ("shrink-0", "flex-shrink: 0"),
        // Justify / align
        ("justify-start", "justify-content: flex-start"),
        ("justify-end", "justify-content: flex-end"),
        ("justify-center", "justify-content: center"),
        ("justify-between", "justify-content: space-between"),
        ("justify-around", "justify-content: space-around"),
        ("justify-evenly", "justify-content: space-evenly"),
        ("items-start", "align-items: flex-start"),
        ("items-end", "align-items: flex-end"),
        ("items-center", "align-items: center"),
        ("items-baseline", "align-items: baseline"),
        ("items-stretch", "align-items: stretch"),
        ("self-auto", "align-self: auto"),
        ("self-start", "align-self: flex-start"),
        ("self-end", "align-self: flex-end"),
        ("self-center", "align-self: center"),
        ("self-stretch", "align-self: stretch"),
        // Overflow
        ("overflow-auto", "overflow: auto"),
        ("overflow-hidden", "overflow: hidden"),
        ("overflow-visible", "overflow: visible"),
        ("overflow-scroll", "overflow: scroll"),
        // Text alignment
        ("text-left", "text-align: left"),
        ("text-center", "text-align: center"),
        ("text-right", "text-align: right"),
        ("text-justify", "text-align: justify"),
        // Font weight
        ("font-thin", "font-weight: 100"),
        ("font-extralight", "font-weight: 200"),
        ("font-light", "font-weight: 300"),
        ("font-normal", "font-weight: 400"),
        ("font-medium", "font-weight: 500"),
        ("font-semibold", "font-weight: 600"),
        ("font-bold", "font-weight: 700"),
        ("font-extrabold", "font-weight: 800"),
        ("font-black", "font-weight: 900"),
        // Font style
        ("italic", "font-style: italic"),
        ("not-italic", "font-style: normal"),
        // Text decoration
        ("underline", "text-decoration-line: underline"),
        ("overline", "text-decoration-line: overline"),
        ("line-through", "text-decoration-line: line-through"),
        ("no-underline", "text-decoration-line: none"),
        // Text transform
        ("uppercase", "text-transform: uppercase"),
        ("lowercase", "text-transform: lowercase"),
        ("capitalize", "text-transform: capitalize"),
        ("normal-case", "text-transform: none"),
        // Whitespace
        ("whitespace-normal", "white-space: normal"),
        ("whitespace-nowrap", "white-space: nowrap"),
        ("whitespace-pre", "white-space: pre"),
        ("truncate", "overflow: hidden; text-overflow: ellipsis; white-space: nowrap"),
        // Border style
        ("border-solid", "border-style: solid"),
        ("border-dashed", "border-style: dashed"),
        ("border-dotted", "border-style: dotted"),
        ("border-double", "border-style: double"),
        ("border-none", "border-style: none"),
        // Visibility
        ("visible", "visibility: visible"),
        ("invisible", "visibility: hidden"),
        // Cursor
        ("cursor-pointer", "cursor: pointer"),
        ("cursor-default", "cursor: default"),
        ("cursor-not-allowed", "cursor: not-allowed"),
        // Pointer events
        ("pointer-events-none", "pointer-events: none"),
        ("pointer-events-auto", "pointer-events: auto"),
        // Object fit
        ("object-contain", "object-fit: contain"),
        ("object-cover", "object-fit: cover"),
        ("object-fill", "object-fit: fill"),
        ("object-none", "object-fit: none"),
        // Misc
        ("container", "width: 100%; margin-left: auto; margin-right: auto"),
        ("sr-only", "position: absolute; width: 1px; height: 1px; padding: 0; margin: -1px; overflow: hidden; clip: rect(0,0,0,0); white-space: nowrap; border-width: 0"),
    ]);

    if let Some(&css) = static_map.get(tw_class) {
        return Some(css.to_string());
    }

    // ---- dynamic / parametric mappings ----
    if let Some(val) = try_spacing_class(tw_class) {
        return Some(val);
    }
    if let Some(val) = try_sizing_class(tw_class) {
        return Some(val);
    }
    if let Some(val) = try_position_inset_class(tw_class) {
        return Some(val);
    }
    if let Some(val) = try_border_radius_class(tw_class) {
        return Some(val);
    }
    if let Some(val) = try_border_width_class(tw_class) {
        return Some(val);
    }
    if let Some(val) = try_color_class(tw_class) {
        return Some(val);
    }
    if let Some(val) = try_typography_class(tw_class) {
        return Some(val);
    }
    if let Some(val) = try_shadow_class(tw_class) {
        return Some(val);
    }
    if let Some(val) = try_opacity_class(tw_class) {
        return Some(val);
    }
    if let Some(val) = try_z_index_class(tw_class) {
        return Some(val);
    }
    if let Some(val) = try_gap_class(tw_class) {
        return Some(val);
    }
    if let Some(val) = try_grid_class(tw_class) {
        return Some(val);
    }

    None
}

// ---------- helper: Tailwind spacing scale ----------

fn tw_spacing_value(key: &str) -> Option<String> {
    // Tailwind default spacing scale (subset that covers the most common
    // values). Arbitrary values like `[20px]` are also handled.
    let table: HashMap<&str, &str> = HashMap::from([
        ("0", "0px"),
        ("px", "1px"),
        ("0.5", "0.125rem"),
        ("1", "0.25rem"),
        ("1.5", "0.375rem"),
        ("2", "0.5rem"),
        ("2.5", "0.625rem"),
        ("3", "0.75rem"),
        ("3.5", "0.875rem"),
        ("4", "1rem"),
        ("5", "1.25rem"),
        ("6", "1.5rem"),
        ("7", "1.75rem"),
        ("8", "2rem"),
        ("9", "2.25rem"),
        ("10", "2.5rem"),
        ("11", "2.75rem"),
        ("12", "3rem"),
        ("14", "3.5rem"),
        ("16", "4rem"),
        ("20", "5rem"),
        ("24", "6rem"),
        ("28", "7rem"),
        ("32", "8rem"),
        ("36", "9rem"),
        ("40", "10rem"),
        ("44", "11rem"),
        ("48", "12rem"),
        ("52", "13rem"),
        ("56", "14rem"),
        ("60", "15rem"),
        ("64", "16rem"),
        ("72", "18rem"),
        ("80", "20rem"),
        ("96", "24rem"),
        ("auto", "auto"),
        ("full", "100%"),
    ]);

    if let Some(&v) = table.get(key) {
        return Some(v.to_string());
    }

    // Arbitrary value: `[12px]` → `12px`
    if key.starts_with('[') && key.ends_with(']') {
        return Some(key[1..key.len() - 1].to_string());
    }

    // Fractions like 1/2, 1/3, etc.
    if key.contains('/') {
        let parts: Vec<&str> = key.split('/').collect();
        if parts.len() == 2 {
            if let (Ok(num), Ok(den)) = (parts[0].parse::<f64>(), parts[1].parse::<f64>()) {
                if den != 0.0 {
                    let pct = (num / den) * 100.0;
                    let s = format!("{pct:.6}");
                    let s = s.trim_end_matches('0').trim_end_matches('.');
                    return Some(format!("{s}%"));
                }
            }
        }
    }

    None
}

#[cfg(test)]
fn format_pct(num: f64, den: f64) -> String {
    if den == 0.0 {
        return "0%".to_string();
    }
    let pct = (num / den) * 100.0;
    let s = format!("{pct:.6}");
    let s = s.trim_end_matches('0').trim_end_matches('.');
    format!("{s}%")
}

fn try_spacing_class(cls: &str) -> Option<String> {
    // p-*, m-*, px-*, py-*, pt-*, pr-*, pb-*, pl-*
    // mx-*, my-*, mt-*, mr-*, mb-*, ml-*
    let mapping: &[(&str, &[&str])] = &[
        ("p-", &["padding"]),
        ("px-", &["padding-left", "padding-right"]),
        ("py-", &["padding-top", "padding-bottom"]),
        ("pt-", &["padding-top"]),
        ("pr-", &["padding-right"]),
        ("pb-", &["padding-bottom"]),
        ("pl-", &["padding-left"]),
        ("m-", &["margin"]),
        ("mx-", &["margin-left", "margin-right"]),
        ("my-", &["margin-top", "margin-bottom"]),
        ("mt-", &["margin-top"]),
        ("mr-", &["margin-right"]),
        ("mb-", &["margin-bottom"]),
        ("ml-", &["margin-left"]),
    ];

    // Negative prefix: -m-4 → margin: -1rem
    let (prefix_sign, effective_cls) = if cls.starts_with('-') {
        ("-", &cls[1..])
    } else {
        ("", cls)
    };

    for &(prefix, properties) in mapping {
        if let Some(val_key) = effective_cls.strip_prefix(prefix) {
            let value = tw_spacing_value(val_key)?;
            let value = if prefix_sign == "-" && value != "0px" && value != "auto" {
                format!("-{value}")
            } else {
                value
            };
            let decls: Vec<String> = properties
                .iter()
                .map(|p| format!("{p}: {value}"))
                .collect();
            return Some(decls.join("; "));
        }
    }

    None
}

fn try_sizing_class(cls: &str) -> Option<String> {
    let mapping: &[(&str, &str)] = &[
        ("w-", "width"),
        ("h-", "height"),
        ("min-w-", "min-width"),
        ("min-h-", "min-height"),
        ("max-w-", "max-width"),
        ("max-h-", "max-height"),
    ];

    for &(prefix, property) in mapping {
        if let Some(val_key) = cls.strip_prefix(prefix) {
            // Special named values
            let value = match val_key {
                "screen" => "100vw".to_string(),
                "full" => "100%".to_string(),
                "auto" => "auto".to_string(),
                "min" => "min-content".to_string(),
                "max" => "max-content".to_string(),
                "fit" => "fit-content".to_string(),
                _ => tw_spacing_value(val_key)?,
            };
            return Some(format!("{property}: {value}"));
        }
    }

    None
}

fn try_position_inset_class(cls: &str) -> Option<String> {
    let mapping: &[(&str, &[&str])] = &[
        ("inset-", &["top", "right", "bottom", "left"]),
        ("inset-x-", &["left", "right"]),
        ("inset-y-", &["top", "bottom"]),
        ("top-", &["top"]),
        ("right-", &["right"]),
        ("bottom-", &["bottom"]),
        ("left-", &["left"]),
    ];

    let (prefix_sign, effective_cls) = if cls.starts_with('-') {
        ("-", &cls[1..])
    } else {
        ("", cls)
    };

    for &(prefix, properties) in mapping {
        if let Some(val_key) = effective_cls.strip_prefix(prefix) {
            let value = tw_spacing_value(val_key)?;
            let value = if prefix_sign == "-" && value != "0px" && value != "auto" {
                format!("-{value}")
            } else {
                value
            };
            let decls: Vec<String> = properties
                .iter()
                .map(|p| format!("{p}: {value}"))
                .collect();
            return Some(decls.join("; "));
        }
    }

    None
}

fn try_border_radius_class(cls: &str) -> Option<String> {
    let scale: HashMap<&str, &str> = HashMap::from([
        ("none", "0px"),
        ("sm", "0.125rem"),
        ("", "0.25rem"),
        ("md", "0.375rem"),
        ("lg", "0.5rem"),
        ("xl", "0.75rem"),
        ("2xl", "1rem"),
        ("3xl", "1.5rem"),
        ("full", "9999px"),
    ]);

    // `rounded` (bare) or `rounded-*`
    if cls == "rounded" {
        return Some(format!("border-radius: {}", scale[""]));
    }
    if let Some(val_key) = cls.strip_prefix("rounded-") {
        // Directional variants
        let dir_map: &[(&str, &[&str])] = &[
            ("t-", &["border-top-left-radius", "border-top-right-radius"]),
            ("r-", &["border-top-right-radius", "border-bottom-right-radius"]),
            ("b-", &["border-bottom-right-radius", "border-bottom-left-radius"]),
            ("l-", &["border-top-left-radius", "border-bottom-left-radius"]),
            ("tl-", &["border-top-left-radius"]),
            ("tr-", &["border-top-right-radius"]),
            ("br-", &["border-bottom-right-radius"]),
            ("bl-", &["border-bottom-left-radius"]),
        ];

        for &(dir_prefix, properties) in dir_map {
            if let Some(size_key) = val_key.strip_prefix(dir_prefix) {
                if let Some(&v) = scale.get(size_key) {
                    let decls: Vec<String> =
                        properties.iter().map(|p| format!("{p}: {v}")).collect();
                    return Some(decls.join("; "));
                }
            }
        }

        if let Some(&v) = scale.get(val_key) {
            return Some(format!("border-radius: {v}"));
        }
    }

    None
}

fn try_border_width_class(cls: &str) -> Option<String> {
    let width_scale: HashMap<&str, &str> = HashMap::from([
        ("", "1px"),
        ("0", "0px"),
        ("2", "2px"),
        ("4", "4px"),
        ("8", "8px"),
    ]);

    if cls == "border" {
        return Some("border-width: 1px".to_string());
    }

    if let Some(val_key) = cls.strip_prefix("border-") {
        // border-t, border-r, etc.
        let dir_map: &[(&str, &str)] = &[
            ("t-", "border-top-width"),
            ("r-", "border-right-width"),
            ("b-", "border-bottom-width"),
            ("l-", "border-left-width"),
            ("x-", "border-left-width; border-right-width"),
            ("y-", "border-top-width; border-bottom-width"),
        ];

        // Check bare directional: border-t, border-r, ...
        let bare_dir: HashMap<&str, &str> = HashMap::from([
            ("t", "border-top-width: 1px"),
            ("r", "border-right-width: 1px"),
            ("b", "border-bottom-width: 1px"),
            ("l", "border-left-width: 1px"),
            ("x", "border-left-width: 1px; border-right-width: 1px"),
            ("y", "border-top-width: 1px; border-bottom-width: 1px"),
        ]);

        if let Some(&css) = bare_dir.get(val_key) {
            return Some(css.to_string());
        }

        for &(dir_prefix, prop_template) in dir_map {
            if let Some(w_key) = val_key.strip_prefix(dir_prefix) {
                if let Some(&w) = width_scale.get(w_key) {
                    let decls: Vec<String> = prop_template
                        .split("; ")
                        .map(|p| format!("{p}: {w}"))
                        .collect();
                    return Some(decls.join("; "));
                }
            }
        }

        // border-0, border-2, etc.
        if let Some(&w) = width_scale.get(val_key) {
            return Some(format!("border-width: {w}"));
        }
    }

    None
}

fn try_color_class(cls: &str) -> Option<String> {
    // bg-*, text-* (color), border-* (color)
    // We match patterns like bg-red-500, text-blue-700, border-gray-300,
    // as well as named colors: bg-white, bg-black, bg-transparent, bg-current.

    let named_colors: HashMap<&str, &str> = HashMap::from([
        ("black", "#000000"),
        ("white", "#ffffff"),
        ("transparent", "transparent"),
        ("current", "currentColor"),
        ("inherit", "inherit"),
    ]);

    let color_prefixes: &[(&str, &str)] = &[
        ("bg-", "background-color"),
        ("text-", "color"),
        ("border-", "border-color"),
        ("ring-", "outline-color"),
        ("accent-", "accent-color"),
        ("fill-", "fill"),
        ("stroke-", "stroke"),
    ];

    // Avoid false positives: text-left, text-center, etc. are NOT color classes.
    let text_non_color = [
        "left", "center", "right", "justify", "xs", "sm", "base", "lg", "xl",
        "2xl", "3xl", "4xl", "5xl", "6xl", "7xl", "8xl", "9xl",
    ];

    for &(prefix, property) in color_prefixes {
        if let Some(color_key) = cls.strip_prefix(prefix) {
            // Skip text-alignment and text-size classes
            if prefix == "text-" && text_non_color.contains(&color_key) {
                return None;
            }
            // Skip border-width and border-style classes
            if prefix == "border-" {
                let border_non_color = [
                    "0", "2", "4", "8", "t", "r", "b", "l", "x", "y", "solid",
                    "dashed", "dotted", "double", "none",
                ];
                if border_non_color.contains(&color_key) {
                    return None;
                }
                // Also skip if it starts with a directional prefix like t-, r-, etc.
                if color_key.starts_with("t-")
                    || color_key.starts_with("r-")
                    || color_key.starts_with("b-")
                    || color_key.starts_with("l-")
                    || color_key.starts_with("x-")
                    || color_key.starts_with("y-")
                {
                    return None;
                }
            }

            if let Some(&v) = named_colors.get(color_key) {
                return Some(format!("{property}: {v}"));
            }
            // Arbitrary: bg-[#ff0000]
            if color_key.starts_with('[') && color_key.ends_with(']') {
                let v = &color_key[1..color_key.len() - 1];
                return Some(format!("{property}: {v}"));
            }
            // Named with shade: bg-red-500 → use var(--color-red-500)
            if color_key.contains('-') || color_key.chars().all(|c| c.is_alphabetic()) {
                return Some(format!("{property}: var(--color-{color_key})"));
            }
        }
    }

    None
}

fn try_typography_class(cls: &str) -> Option<String> {
    // text-xs .. text-9xl (sizes)
    let text_sizes: HashMap<&str, (&str, &str)> = HashMap::from([
        ("text-xs", ("0.75rem", "1rem")),
        ("text-sm", ("0.875rem", "1.25rem")),
        ("text-base", ("1rem", "1.5rem")),
        ("text-lg", ("1.125rem", "1.75rem")),
        ("text-xl", ("1.25rem", "1.75rem")),
        ("text-2xl", ("1.5rem", "2rem")),
        ("text-3xl", ("1.875rem", "2.25rem")),
        ("text-4xl", ("2.25rem", "2.5rem")),
        ("text-5xl", ("3rem", "1")),
        ("text-6xl", ("3.75rem", "1")),
        ("text-7xl", ("4.5rem", "1")),
        ("text-8xl", ("6rem", "1")),
        ("text-9xl", ("8rem", "1")),
    ]);

    if let Some(&(size, lh)) = text_sizes.get(cls) {
        return Some(format!("font-size: {size}; line-height: {lh}"));
    }

    // leading-* (line-height)
    if let Some(val_key) = cls.strip_prefix("leading-") {
        let leading: HashMap<&str, &str> = HashMap::from([
            ("none", "1"),
            ("tight", "1.25"),
            ("snug", "1.375"),
            ("normal", "1.5"),
            ("relaxed", "1.625"),
            ("loose", "2"),
            ("3", "0.75rem"),
            ("4", "1rem"),
            ("5", "1.25rem"),
            ("6", "1.5rem"),
            ("7", "1.75rem"),
            ("8", "2rem"),
            ("9", "2.25rem"),
            ("10", "2.5rem"),
        ]);
        if let Some(&v) = leading.get(val_key) {
            return Some(format!("line-height: {v}"));
        }
    }

    // tracking-* (letter-spacing)
    if let Some(val_key) = cls.strip_prefix("tracking-") {
        let tracking: HashMap<&str, &str> = HashMap::from([
            ("tighter", "-0.05em"),
            ("tight", "-0.025em"),
            ("normal", "0em"),
            ("wide", "0.025em"),
            ("wider", "0.05em"),
            ("widest", "0.1em"),
        ]);
        if let Some(&v) = tracking.get(val_key) {
            return Some(format!("letter-spacing: {v}"));
        }
    }

    // font-sans, font-serif, font-mono
    if let Some(val_key) = cls.strip_prefix("font-") {
        let font_families: HashMap<&str, &str> = HashMap::from([
            (
                "sans",
                "font-family: ui-sans-serif, system-ui, sans-serif",
            ),
            ("serif", "font-family: ui-serif, Georgia, serif"),
            ("mono", "font-family: ui-monospace, monospace"),
        ]);
        if let Some(&v) = font_families.get(val_key) {
            return Some(v.to_string());
        }
    }

    None
}

fn try_shadow_class(cls: &str) -> Option<String> {
    let shadows: HashMap<&str, &str> = HashMap::from([
        ("shadow-sm", "0 1px 2px 0 rgb(0 0 0 / 0.05)"),
        ("shadow", "0 1px 3px 0 rgb(0 0 0 / 0.1), 0 1px 2px -1px rgb(0 0 0 / 0.1)"),
        ("shadow-md", "0 4px 6px -1px rgb(0 0 0 / 0.1), 0 2px 4px -2px rgb(0 0 0 / 0.1)"),
        ("shadow-lg", "0 10px 15px -3px rgb(0 0 0 / 0.1), 0 4px 6px -4px rgb(0 0 0 / 0.1)"),
        ("shadow-xl", "0 20px 25px -5px rgb(0 0 0 / 0.1), 0 8px 10px -6px rgb(0 0 0 / 0.1)"),
        ("shadow-2xl", "0 25px 50px -12px rgb(0 0 0 / 0.25)"),
        ("shadow-inner", "inset 0 2px 4px 0 rgb(0 0 0 / 0.05)"),
        ("shadow-none", "none"),
    ]);

    if let Some(&v) = shadows.get(cls) {
        return Some(format!("box-shadow: {v}"));
    }

    None
}

fn try_opacity_class(cls: &str) -> Option<String> {
    if let Some(val_key) = cls.strip_prefix("opacity-") {
        if let Ok(n) = val_key.parse::<u32>() {
            if n <= 100 {
                let v = n as f64 / 100.0;
                // Avoid trailing zeros
                if n == 0 {
                    return Some("opacity: 0".to_string());
                } else if n == 100 {
                    return Some("opacity: 1".to_string());
                } else {
                    return Some(format!("opacity: {v}"));
                }
            }
        }
    }
    None
}

fn try_z_index_class(cls: &str) -> Option<String> {
    if let Some(val_key) = cls.strip_prefix("z-") {
        let z_values: HashMap<&str, &str> = HashMap::from([
            ("0", "0"),
            ("10", "10"),
            ("20", "20"),
            ("30", "30"),
            ("40", "40"),
            ("50", "50"),
            ("auto", "auto"),
        ]);
        if let Some(&v) = z_values.get(val_key) {
            return Some(format!("z-index: {v}"));
        }
        // Arbitrary
        if val_key.starts_with('[') && val_key.ends_with(']') {
            let v = &val_key[1..val_key.len() - 1];
            return Some(format!("z-index: {v}"));
        }
    }
    None
}

fn try_gap_class(cls: &str) -> Option<String> {
    let mapping: &[(&str, &str)] = &[
        ("gap-x-", "column-gap"),
        ("gap-y-", "row-gap"),
        ("gap-", "gap"),
    ];

    for &(prefix, property) in mapping {
        if let Some(val_key) = cls.strip_prefix(prefix) {
            let value = tw_spacing_value(val_key)?;
            return Some(format!("{property}: {value}"));
        }
    }

    None
}

fn try_grid_class(cls: &str) -> Option<String> {
    // grid-cols-N
    if let Some(val_key) = cls.strip_prefix("grid-cols-") {
        if val_key == "none" {
            return Some("grid-template-columns: none".to_string());
        }
        if let Ok(n) = val_key.parse::<u32>() {
            return Some(format!(
                "grid-template-columns: repeat({n}, minmax(0, 1fr))"
            ));
        }
    }
    // grid-rows-N
    if let Some(val_key) = cls.strip_prefix("grid-rows-") {
        if val_key == "none" {
            return Some("grid-template-rows: none".to_string());
        }
        if let Ok(n) = val_key.parse::<u32>() {
            return Some(format!(
                "grid-template-rows: repeat({n}, minmax(0, 1fr))"
            ));
        }
    }
    // col-span-N
    if let Some(val_key) = cls.strip_prefix("col-span-") {
        if val_key == "full" {
            return Some("grid-column: 1 / -1".to_string());
        }
        if let Ok(n) = val_key.parse::<u32>() {
            return Some(format!("grid-column: span {n} / span {n}"));
        }
    }
    // row-span-N
    if let Some(val_key) = cls.strip_prefix("row-span-") {
        if val_key == "full" {
            return Some("grid-row: 1 / -1".to_string());
        }
        if let Ok(n) = val_key.parse::<u32>() {
            return Some(format!("grid-row: span {n} / span {n}"));
        }
    }

    None
}

// ---------------------------------------------------------------------------
// 4. Convert Tailwind CSS directives to Euis
// ---------------------------------------------------------------------------

/// Convert a CSS file that uses Tailwind directives (`@tailwind`, `@apply`,
/// `@layer`) to equivalent Euis output.
pub fn convert_tailwind_css(css_content: &str) -> MigrationResult {
    let mut output_lines: Vec<String> = Vec::new();
    let mut warnings: Vec<MigrationWarning> = Vec::new();
    let mut classes_converted: usize = 0;
    let mut directives_converted: usize = 0;
    let mut manual_review_needed: usize = 0;

    for (line_idx, line) in css_content.lines().enumerate() {
        let trimmed = line.trim();
        let line_no = line_idx + 1;

        // @tailwind directives → comments / Euis equivalents
        if trimmed.starts_with("@tailwind") {
            directives_converted += 1;
            let directive_arg = trimmed
                .strip_prefix("@tailwind")
                .unwrap_or("")
                .trim()
                .trim_end_matches(';')
                .trim();

            match directive_arg {
                "base" => {
                    output_lines.push(
                        "/* @tailwind base → include your base/reset styles here */".to_string(),
                    );
                }
                "components" => {
                    output_lines.push(
                        "/* @tailwind components → component styles are written directly in Euis */"
                            .to_string(),
                    );
                }
                "utilities" => {
                    output_lines.push(
                        "/* @tailwind utilities → utility classes are written directly in Euis */"
                            .to_string(),
                    );
                }
                other => {
                    output_lines.push(format!("/* @tailwind {other} → unsupported, review manually */"));
                    warnings.push(MigrationWarning {
                        message: format!("Unknown @tailwind directive: {other}"),
                        line: Some(line_no),
                        suggestion: Some("Remove or replace with equivalent Euis.".to_string()),
                    });
                    manual_review_needed += 1;
                }
            }
            continue;
        }

        // @apply → expand inline
        if trimmed.starts_with("@apply") {
            directives_converted += 1;
            let classes_str = trimmed
                .strip_prefix("@apply")
                .unwrap_or("")
                .trim()
                .trim_end_matches(';')
                .trim();

            let indent = &line[..line.len() - line.trim_start().len()];

            for cls in classes_str.split_whitespace() {
                if let Some(css) = map_utility_class(cls) {
                    // A mapping may contain multiple declarations separated by "; "
                    for decl in css.split("; ") {
                        output_lines.push(format!("{indent}{decl};"));
                    }
                    classes_converted += 1;
                } else {
                    output_lines.push(format!("{indent}/* TODO: manually convert '{cls}' */"));
                    warnings.push(MigrationWarning {
                        message: format!("Could not convert utility class: {cls}"),
                        line: Some(line_no),
                        suggestion: Some(format!("Look up the CSS for '{cls}' and add it manually.")),
                    });
                    manual_review_needed += 1;
                }
            }
            continue;
        }

        // @layer → preserved as-is (CSS @layer is standard and Euis supports it)
        if trimmed.starts_with("@layer") {
            directives_converted += 1;
            output_lines.push(line.to_string());
            continue;
        }

        // @screen → @media with breakpoint
        if trimmed.starts_with("@screen") {
            directives_converted += 1;
            let bp = trimmed
                .strip_prefix("@screen")
                .unwrap_or("")
                .trim()
                .trim_end_matches('{')
                .trim();
            output_lines.push(format!("@media (min-width: var(--screen-{bp})) {{"));
            warnings.push(MigrationWarning {
                message: format!("@screen {bp} converted to @media; ensure --screen-{bp} is defined."),
                line: Some(line_no),
                suggestion: Some(format!("Define --screen-{bp} in your :root tokens or replace with a concrete value.")),
            });
            manual_review_needed += 1;
            continue;
        }

        // @responsive → @media wrapper hint
        if trimmed.starts_with("@responsive") {
            directives_converted += 1;
            output_lines.push("/* @responsive → use @media queries directly in Euis */".to_string());
            warnings.push(MigrationWarning {
                message: "@responsive is not needed in Euis.".to_string(),
                line: Some(line_no),
                suggestion: Some("Wrap styles in standard @media queries instead.".to_string()),
            });
            manual_review_needed += 1;
            continue;
        }

        // Pass through everything else unchanged
        output_lines.push(line.to_string());
    }

    let content = output_lines.join("\n");

    MigrationResult {
        euis_config: String::new(),
        converted_files: vec![ConvertedFile {
            original_path: "input.css".to_string(),
            content,
        }],
        warnings,
        stats: MigrationStats {
            classes_converted,
            directives_converted,
            manual_review_needed,
        },
    }
}

// ---------------------------------------------------------------------------
// 5. Full migration entry point
// ---------------------------------------------------------------------------

/// Full migration entry point: converts CSS content and optionally a Tailwind
/// config JSON into a complete Euis project output.
pub fn migrate_project(css_content: &str, config_json: Option<&str>) -> MigrationResult {
    let mut result = convert_tailwind_css(css_content);

    if let Some(json) = config_json {
        match parse_tailwind_config(json) {
            Ok(tw_config) => {
                result.euis_config = generate_euis_config(&tw_config);
            }
            Err(err) => {
                result.warnings.push(MigrationWarning {
                    message: format!("Failed to parse Tailwind config: {err}"),
                    line: None,
                    suggestion: Some(
                        "Ensure the config JSON has a top-level \"theme\" key.".to_string(),
                    ),
                });
                result.stats.manual_review_needed += 1;
            }
        }
    }

    result
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // -- parse_tailwind_config -------------------------------------------------

    #[test]
    fn test_parse_config_basic() {
        let json = r##"{
            "theme": {
                "colors": {
                    "primary": "#3b82f6",
                    "gray": { "100": "#f3f4f6", "900": "#111827" }
                },
                "spacing": { "1": "0.25rem", "4": "1rem" },
                "fontFamily": { "sans": ["Inter", "sans-serif"] },
                "fontSize": { "sm": "0.875rem" },
                "borderRadius": { "lg": "0.5rem" },
                "screens": { "sm": "640px", "md": "768px" },
                "boxShadow": { "md": "0 4px 6px rgba(0,0,0,.1)" },
                "opacity": { "50": "0.5" },
                "zIndex": { "10": "10" }
            }
        }"##;

        let config = parse_tailwind_config(json).unwrap();
        assert_eq!(config.colors.get("primary").unwrap(), "#3b82f6");
        assert_eq!(config.colors.get("gray-100").unwrap(), "#f3f4f6");
        assert_eq!(config.colors.get("gray-900").unwrap(), "#111827");
        assert_eq!(config.spacing.get("4").unwrap(), "1rem");
        assert_eq!(config.font_family.get("sans").unwrap(), &vec!["Inter".to_string(), "sans-serif".to_string()]);
        assert_eq!(config.font_size.get("sm").unwrap(), "0.875rem");
        assert_eq!(config.border_radius.get("lg").unwrap(), "0.5rem");
        assert_eq!(config.breakpoints.get("sm").unwrap(), "640px");
        assert_eq!(config.box_shadow.get("md").unwrap(), "0 4px 6px rgba(0,0,0,.1)");
        assert_eq!(config.opacity.get("50").unwrap(), "0.5");
        assert_eq!(config.z_index.get("10").unwrap(), "10");
    }

    #[test]
    fn test_parse_config_missing_theme() {
        let json = r#"{ "plugins": [] }"#;
        let err = parse_tailwind_config(json).unwrap_err();
        assert!(err.contains("theme"));
    }

    #[test]
    fn test_parse_config_invalid_json() {
        let err = parse_tailwind_config("not json").unwrap_err();
        assert!(err.contains("Invalid JSON"));
    }

    #[test]
    fn test_parse_config_empty_theme() {
        let json = r#"{ "theme": {} }"#;
        let config = parse_tailwind_config(json).unwrap();
        assert!(config.colors.is_empty());
        assert!(config.spacing.is_empty());
    }

    // -- generate_euis_config --------------------------------------------------

    #[test]
    fn test_generate_euis_config_colors() {
        let mut config = TailwindConfig::default();
        config.colors.insert("primary".to_string(), "#3b82f6".to_string());
        config.colors.insert("danger".to_string(), "#ef4444".to_string());

        let output = generate_euis_config(&config);
        assert!(output.contains("--color-danger: #ef4444;"));
        assert!(output.contains("--color-primary: #3b82f6;"));
    }

    #[test]
    fn test_generate_euis_config_breakpoints() {
        let mut config = TailwindConfig::default();
        config.breakpoints.insert("sm".to_string(), "640px".to_string());

        let output = generate_euis_config(&config);
        assert!(output.contains("@custom-media --screen-sm (min-width: 640px);"));
    }

    #[test]
    fn test_generate_euis_config_font_families() {
        let mut config = TailwindConfig::default();
        config.font_family.insert(
            "sans".to_string(),
            vec!["Inter".to_string(), "sans-serif".to_string()],
        );

        let output = generate_euis_config(&config);
        assert!(output.contains("--font-sans: Inter, sans-serif;"));
    }

    #[test]
    fn test_generate_euis_config_empty() {
        let config = TailwindConfig::default();
        let output = generate_euis_config(&config);
        assert!(output.contains(":root {"));
        assert!(output.contains("}"));
    }

    // -- map_utility_class: layout & display -----------------------------------

    #[test]
    fn test_map_display_classes() {
        assert_eq!(map_utility_class("flex").unwrap(), "display: flex");
        assert_eq!(map_utility_class("grid").unwrap(), "display: grid");
        assert_eq!(map_utility_class("block").unwrap(), "display: block");
        assert_eq!(map_utility_class("inline").unwrap(), "display: inline");
        assert_eq!(map_utility_class("hidden").unwrap(), "display: none");
        assert_eq!(map_utility_class("inline-flex").unwrap(), "display: inline-flex");
        assert_eq!(map_utility_class("inline-block").unwrap(), "display: inline-block");
        assert_eq!(map_utility_class("contents").unwrap(), "display: contents");
    }

    #[test]
    fn test_map_position_classes() {
        assert_eq!(map_utility_class("absolute").unwrap(), "position: absolute");
        assert_eq!(map_utility_class("relative").unwrap(), "position: relative");
        assert_eq!(map_utility_class("fixed").unwrap(), "position: fixed");
        assert_eq!(map_utility_class("sticky").unwrap(), "position: sticky");
        assert_eq!(map_utility_class("static").unwrap(), "position: static");
    }

    // -- map_utility_class: spacing --------------------------------------------

    #[test]
    fn test_map_padding() {
        assert_eq!(map_utility_class("p-4").unwrap(), "padding: 1rem");
        assert_eq!(map_utility_class("p-0").unwrap(), "padding: 0px");
        assert_eq!(map_utility_class("p-px").unwrap(), "padding: 1px");
        assert_eq!(
            map_utility_class("px-4").unwrap(),
            "padding-left: 1rem; padding-right: 1rem"
        );
        assert_eq!(
            map_utility_class("py-2").unwrap(),
            "padding-top: 0.5rem; padding-bottom: 0.5rem"
        );
        assert_eq!(map_utility_class("pt-8").unwrap(), "padding-top: 2rem");
        assert_eq!(map_utility_class("pr-6").unwrap(), "padding-right: 1.5rem");
        assert_eq!(map_utility_class("pb-3").unwrap(), "padding-bottom: 0.75rem");
        assert_eq!(map_utility_class("pl-1").unwrap(), "padding-left: 0.25rem");
    }

    #[test]
    fn test_map_margin() {
        assert_eq!(map_utility_class("m-4").unwrap(), "margin: 1rem");
        assert_eq!(map_utility_class("m-auto").unwrap(), "margin: auto");
        assert_eq!(
            map_utility_class("mx-auto").unwrap(),
            "margin-left: auto; margin-right: auto"
        );
        assert_eq!(
            map_utility_class("my-8").unwrap(),
            "margin-top: 2rem; margin-bottom: 2rem"
        );
        assert_eq!(map_utility_class("mt-4").unwrap(), "margin-top: 1rem");
        assert_eq!(map_utility_class("mr-2").unwrap(), "margin-right: 0.5rem");
        assert_eq!(map_utility_class("mb-6").unwrap(), "margin-bottom: 1.5rem");
        assert_eq!(map_utility_class("ml-3").unwrap(), "margin-left: 0.75rem");
    }

    #[test]
    fn test_map_negative_margin() {
        assert_eq!(map_utility_class("-m-4").unwrap(), "margin: -1rem");
        assert_eq!(map_utility_class("-mt-2").unwrap(), "margin-top: -0.5rem");
    }

    #[test]
    fn test_map_arbitrary_spacing() {
        assert_eq!(map_utility_class("p-[20px]").unwrap(), "padding: 20px");
        assert_eq!(map_utility_class("m-[2em]").unwrap(), "margin: 2em");
    }

    // -- map_utility_class: sizing ---------------------------------------------

    #[test]
    fn test_map_width() {
        assert_eq!(map_utility_class("w-full").unwrap(), "width: 100%");
        assert_eq!(map_utility_class("w-screen").unwrap(), "width: 100vw");
        assert_eq!(map_utility_class("w-auto").unwrap(), "width: auto");
        assert_eq!(map_utility_class("w-4").unwrap(), "width: 1rem");
        assert_eq!(map_utility_class("w-1/2").unwrap(), "width: 50%");
    }

    #[test]
    fn test_map_height() {
        assert_eq!(map_utility_class("h-full").unwrap(), "height: 100%");
        assert_eq!(map_utility_class("h-screen").unwrap(), "height: 100vw");
        assert_eq!(map_utility_class("h-16").unwrap(), "height: 4rem");
    }

    #[test]
    fn test_map_min_max_sizing() {
        assert_eq!(map_utility_class("min-w-0").unwrap(), "min-width: 0px");
        assert_eq!(map_utility_class("min-h-full").unwrap(), "min-height: 100%");
        assert_eq!(map_utility_class("max-w-full").unwrap(), "max-width: 100%");
        assert_eq!(map_utility_class("max-h-screen").unwrap(), "max-height: 100vw");
    }

    // -- map_utility_class: typography -----------------------------------------

    #[test]
    fn test_map_text_size() {
        let css = map_utility_class("text-sm").unwrap();
        assert!(css.contains("font-size: 0.875rem"));
        assert!(css.contains("line-height: 1.25rem"));

        let css = map_utility_class("text-xl").unwrap();
        assert!(css.contains("font-size: 1.25rem"));
    }

    #[test]
    fn test_map_font_weight() {
        assert_eq!(map_utility_class("font-bold").unwrap(), "font-weight: 700");
        assert_eq!(map_utility_class("font-normal").unwrap(), "font-weight: 400");
        assert_eq!(map_utility_class("font-semibold").unwrap(), "font-weight: 600");
    }

    #[test]
    fn test_map_font_family() {
        assert!(map_utility_class("font-sans").unwrap().contains("font-family:"));
        assert!(map_utility_class("font-mono").unwrap().contains("monospace"));
    }

    #[test]
    fn test_map_leading() {
        assert_eq!(map_utility_class("leading-tight").unwrap(), "line-height: 1.25");
        assert_eq!(map_utility_class("leading-normal").unwrap(), "line-height: 1.5");
    }

    #[test]
    fn test_map_tracking() {
        assert_eq!(map_utility_class("tracking-tight").unwrap(), "letter-spacing: -0.025em");
        assert_eq!(map_utility_class("tracking-widest").unwrap(), "letter-spacing: 0.1em");
    }

    #[test]
    fn test_map_text_align() {
        assert_eq!(map_utility_class("text-left").unwrap(), "text-align: left");
        assert_eq!(map_utility_class("text-center").unwrap(), "text-align: center");
    }

    #[test]
    fn test_map_text_transform() {
        assert_eq!(map_utility_class("uppercase").unwrap(), "text-transform: uppercase");
        assert_eq!(map_utility_class("lowercase").unwrap(), "text-transform: lowercase");
        assert_eq!(map_utility_class("capitalize").unwrap(), "text-transform: capitalize");
    }

    // -- map_utility_class: colors ---------------------------------------------

    #[test]
    fn test_map_bg_colors() {
        assert_eq!(map_utility_class("bg-white").unwrap(), "background-color: #ffffff");
        assert_eq!(map_utility_class("bg-black").unwrap(), "background-color: #000000");
        assert_eq!(
            map_utility_class("bg-transparent").unwrap(),
            "background-color: transparent"
        );
        assert_eq!(
            map_utility_class("bg-red-500").unwrap(),
            "background-color: var(--color-red-500)"
        );
        assert_eq!(
            map_utility_class("bg-[#ff0000]").unwrap(),
            "background-color: #ff0000"
        );
    }

    #[test]
    fn test_map_text_colors() {
        assert_eq!(map_utility_class("text-white").unwrap(), "color: #ffffff");
        assert_eq!(
            map_utility_class("text-gray-700").unwrap(),
            "color: var(--color-gray-700)"
        );
    }

    #[test]
    fn test_map_border_colors() {
        assert_eq!(
            map_utility_class("border-gray-300").unwrap(),
            "border-color: var(--color-gray-300)"
        );
    }

    // -- map_utility_class: borders -------------------------------------------

    #[test]
    fn test_map_border_width() {
        assert_eq!(map_utility_class("border").unwrap(), "border-width: 1px");
        assert_eq!(map_utility_class("border-0").unwrap(), "border-width: 0px");
        assert_eq!(map_utility_class("border-2").unwrap(), "border-width: 2px");
        assert_eq!(map_utility_class("border-t").unwrap(), "border-top-width: 1px");
    }

    #[test]
    fn test_map_border_radius() {
        assert_eq!(map_utility_class("rounded").unwrap(), "border-radius: 0.25rem");
        assert_eq!(map_utility_class("rounded-lg").unwrap(), "border-radius: 0.5rem");
        assert_eq!(map_utility_class("rounded-full").unwrap(), "border-radius: 9999px");
        assert_eq!(map_utility_class("rounded-none").unwrap(), "border-radius: 0px");
    }

    #[test]
    fn test_map_border_style() {
        assert_eq!(map_utility_class("border-solid").unwrap(), "border-style: solid");
        assert_eq!(map_utility_class("border-dashed").unwrap(), "border-style: dashed");
    }

    // -- map_utility_class: effects -------------------------------------------

    #[test]
    fn test_map_shadow() {
        assert!(map_utility_class("shadow").unwrap().starts_with("box-shadow:"));
        assert!(map_utility_class("shadow-lg").unwrap().contains("box-shadow:"));
        assert_eq!(map_utility_class("shadow-none").unwrap(), "box-shadow: none");
    }

    #[test]
    fn test_map_opacity() {
        assert_eq!(map_utility_class("opacity-0").unwrap(), "opacity: 0");
        assert_eq!(map_utility_class("opacity-50").unwrap(), "opacity: 0.5");
        assert_eq!(map_utility_class("opacity-100").unwrap(), "opacity: 1");
        assert_eq!(map_utility_class("opacity-75").unwrap(), "opacity: 0.75");
    }

    // -- map_utility_class: flexbox -------------------------------------------

    #[test]
    fn test_map_flex_direction() {
        assert_eq!(map_utility_class("flex-row").unwrap(), "flex-direction: row");
        assert_eq!(map_utility_class("flex-col").unwrap(), "flex-direction: column");
    }

    #[test]
    fn test_map_justify_items() {
        assert_eq!(
            map_utility_class("justify-center").unwrap(),
            "justify-content: center"
        );
        assert_eq!(
            map_utility_class("justify-between").unwrap(),
            "justify-content: space-between"
        );
        assert_eq!(
            map_utility_class("items-center").unwrap(),
            "align-items: center"
        );
        assert_eq!(
            map_utility_class("items-stretch").unwrap(),
            "align-items: stretch"
        );
    }

    #[test]
    fn test_map_flex_shortcuts() {
        assert_eq!(map_utility_class("flex-1").unwrap(), "flex: 1 1 0%");
        assert_eq!(map_utility_class("flex-none").unwrap(), "flex: none");
    }

    // -- map_utility_class: position insets -----------------------------------

    #[test]
    fn test_map_position_insets() {
        assert_eq!(map_utility_class("top-0").unwrap(), "top: 0px");
        assert_eq!(map_utility_class("right-4").unwrap(), "right: 1rem");
        assert_eq!(map_utility_class("bottom-auto").unwrap(), "bottom: auto");
        assert_eq!(map_utility_class("left-1/2").unwrap(), "left: 50%");
        assert!(map_utility_class("inset-0").unwrap().contains("top: 0px"));
        assert!(map_utility_class("inset-0").unwrap().contains("left: 0px"));
    }

    #[test]
    fn test_map_negative_insets() {
        assert_eq!(map_utility_class("-top-4").unwrap(), "top: -1rem");
    }

    // -- map_utility_class: z-index -------------------------------------------

    #[test]
    fn test_map_z_index() {
        assert_eq!(map_utility_class("z-10").unwrap(), "z-index: 10");
        assert_eq!(map_utility_class("z-50").unwrap(), "z-index: 50");
        assert_eq!(map_utility_class("z-auto").unwrap(), "z-index: auto");
        assert_eq!(map_utility_class("z-[999]").unwrap(), "z-index: 999");
    }

    // -- map_utility_class: gap -----------------------------------------------

    #[test]
    fn test_map_gap() {
        assert_eq!(map_utility_class("gap-4").unwrap(), "gap: 1rem");
        assert_eq!(map_utility_class("gap-x-2").unwrap(), "column-gap: 0.5rem");
        assert_eq!(map_utility_class("gap-y-8").unwrap(), "row-gap: 2rem");
    }

    // -- map_utility_class: grid ----------------------------------------------

    #[test]
    fn test_map_grid_cols() {
        assert_eq!(
            map_utility_class("grid-cols-3").unwrap(),
            "grid-template-columns: repeat(3, minmax(0, 1fr))"
        );
        assert_eq!(
            map_utility_class("grid-cols-none").unwrap(),
            "grid-template-columns: none"
        );
    }

    #[test]
    fn test_map_col_span() {
        assert_eq!(
            map_utility_class("col-span-2").unwrap(),
            "grid-column: span 2 / span 2"
        );
        assert_eq!(
            map_utility_class("col-span-full").unwrap(),
            "grid-column: 1 / -1"
        );
    }

    // -- map_utility_class: unknown -------------------------------------------

    #[test]
    fn test_map_unknown_class() {
        assert!(map_utility_class("nonexistent-thing").is_none());
    }

    // -- convert_tailwind_css (directives) ------------------------------------

    #[test]
    fn test_convert_tailwind_directives() {
        let input = "@tailwind base;\n@tailwind components;\n@tailwind utilities;\n";
        let result = convert_tailwind_css(input);
        assert_eq!(result.stats.directives_converted, 3);
        let content = &result.converted_files[0].content;
        assert!(content.contains("base/reset"));
        assert!(content.contains("component styles"));
        assert!(content.contains("utility classes"));
    }

    #[test]
    fn test_convert_apply() {
        let input = ".btn {\n  @apply flex items-center px-4 py-2;\n}\n";
        let result = convert_tailwind_css(input);
        let content = &result.converted_files[0].content;
        assert!(content.contains("display: flex;"));
        assert!(content.contains("align-items: center;"));
        assert!(content.contains("padding-left: 1rem;"));
        assert!(content.contains("padding-right: 1rem;"));
        assert!(content.contains("padding-top: 0.5rem;"));
        assert!(content.contains("padding-bottom: 0.5rem;"));
        assert!(result.stats.classes_converted >= 3);
    }

    #[test]
    fn test_convert_apply_unknown_class() {
        let input = ".x {\n  @apply some-plugin-class;\n}";
        let result = convert_tailwind_css(input);
        let content = &result.converted_files[0].content;
        assert!(content.contains("TODO"));
        assert_eq!(result.stats.manual_review_needed, 1);
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_convert_layer_passthrough() {
        let input = "@layer components {\n  .btn { color: red; }\n}\n";
        let result = convert_tailwind_css(input);
        let content = &result.converted_files[0].content;
        assert!(content.contains("@layer components"));
        assert_eq!(result.stats.directives_converted, 1);
    }

    #[test]
    fn test_convert_screen_directive() {
        let input = "@screen md {\n  .container { max-width: 768px; }\n}\n";
        let result = convert_tailwind_css(input);
        let content = &result.converted_files[0].content;
        assert!(content.contains("@media (min-width: var(--screen-md))"));
        assert!(result.stats.manual_review_needed >= 1);
    }

    #[test]
    fn test_convert_preserves_regular_css() {
        let input = ".card {\n  color: red;\n  background: blue;\n}\n";
        let result = convert_tailwind_css(input);
        let content = &result.converted_files[0].content;
        assert!(content.contains("color: red;"));
        assert!(content.contains("background: blue;"));
        assert_eq!(result.stats.directives_converted, 0);
    }

    // -- migrate_project -------------------------------------------------------

    #[test]
    fn test_migrate_project_css_only() {
        let css = "@tailwind base;\n.btn {\n  @apply flex p-4;\n}\n";
        let result = migrate_project(css, None);
        assert!(result.euis_config.is_empty());
        assert!(!result.converted_files.is_empty());
        assert!(result.stats.classes_converted >= 2);
    }

    #[test]
    fn test_migrate_project_with_config() {
        let css = "@tailwind base;\n";
        let config = r##"{ "theme": { "colors": { "brand": "#ff6600" } } }"##;
        let result = migrate_project(css, Some(config));
        assert!(result.euis_config.contains("--color-brand: #ff6600;"));
        assert!(result.stats.directives_converted >= 1);
    }

    #[test]
    fn test_migrate_project_bad_config() {
        let css = ".x { color: red; }";
        let result = migrate_project(css, Some("bad json"));
        assert!(result.euis_config.is_empty());
        assert!(result.warnings.iter().any(|w| w.message.contains("Failed to parse")));
    }

    // -- edge cases -----------------------------------------------------------

    #[test]
    fn test_container_class() {
        let css = map_utility_class("container").unwrap();
        assert!(css.contains("width: 100%"));
        assert!(css.contains("margin-left: auto"));
        assert!(css.contains("margin-right: auto"));
    }

    #[test]
    fn test_truncate_class() {
        let css = map_utility_class("truncate").unwrap();
        assert!(css.contains("overflow: hidden"));
        assert!(css.contains("text-overflow: ellipsis"));
        assert!(css.contains("white-space: nowrap"));
    }

    #[test]
    fn test_fraction_width() {
        let css = map_utility_class("w-1/3").unwrap();
        assert!(css.contains("33."));
        assert!(css.contains("%"));
    }

    #[test]
    fn test_format_pct_helper() {
        assert_eq!(format_pct(1.0, 2.0), "50%");
        assert_eq!(format_pct(1.0, 3.0), "33.333333%");
        assert_eq!(format_pct(0.0, 1.0), "0%");
    }
}
