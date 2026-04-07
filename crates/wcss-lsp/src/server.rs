use dashmap::DashMap;
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::completions;
use crate::diagnostics_provider;

/// The WCSS Language Server.
pub struct WcssLanguageServer {
    client: Client,
    documents: DashMap<String, String>,
}

impl WcssLanguageServer {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
        }
    }

    /// Run diagnostics on a document and publish the results.
    async fn publish_diagnostics(&self, uri: Url, source: &str) {
        let diagnostics = diagnostics_provider::get_diagnostics(uri.as_str(), source);
        self.client
            .publish_diagnostics(uri, diagnostics, None)
            .await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for WcssLanguageServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![
                        ":".to_string(),
                        "@".to_string(),
                        ".".to_string(),
                        "-".to_string(),
                        " ".to_string(),
                    ]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "wcss-lsp".to_string(),
                version: Some(env!("CARGO_PKG_VERSION").to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "WCSS Language Server started")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        let text = params.text_document.text.clone();
        self.documents
            .insert(uri.to_string(), text.clone());
        self.publish_diagnostics(uri, &text).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        if let Some(change) = params.content_changes.into_iter().last() {
            let text = change.text.clone();
            self.documents
                .insert(uri.to_string(), text.clone());
            self.publish_diagnostics(uri, &text).await;
        }
    }

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri.clone();
        if let Some(entry) = self.documents.get(&uri.to_string()) {
            let text = entry.value().clone();
            self.publish_diagnostics(uri, &text).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        self.documents.remove(&uri);
        // Clear diagnostics for the closed document.
        self.client
            .publish_diagnostics(params.text_document.uri, vec![], None)
            .await;
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;

        let items = if let Some(entry) = self.documents.get(&uri) {
            let source = entry.value();
            let lines: Vec<&str> = source.lines().collect();
            let line_text = lines
                .get(position.line as usize)
                .copied()
                .unwrap_or("");
            completions::get_completions(line_text, position)
        } else {
            vec![]
        };

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        let hover = if let Some(entry) = self.documents.get(&uri) {
            let source = entry.value();
            let lines: Vec<&str> = source.lines().collect();
            if let Some(line) = lines.get(position.line as usize) {
                get_hover_info(line, position)
            } else {
                None
            }
        } else {
            None
        };

        Ok(hover)
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri.to_string();

        let edits = if let Some(entry) = self.documents.get(&uri) {
            let source = entry.value();
            match wcss_compiler::format(source) {
                Ok(formatted) => {
                    let line_count = source.lines().count().max(1) as u32;
                    let last_line_len = source.lines().last().map_or(0, |l| l.len()) as u32;
                    Some(vec![TextEdit {
                        range: Range {
                            start: Position::new(0, 0),
                            end: Position::new(line_count, last_line_len),
                        },
                        new_text: formatted,
                    }])
                }
                Err(_) => None,
            }
        } else {
            None
        };

        Ok(edits)
    }
}

// ---------------------------------------------------------------------------
// Hover support
// ---------------------------------------------------------------------------

/// Extract the word under the cursor from a line of text.
fn word_at_position(line: &str, col: u32) -> Option<&str> {
    let col = col as usize;
    if col > line.len() {
        return None;
    }
    let bytes = line.as_bytes();
    let mut start = col;
    let mut end = col;

    while start > 0
        && (bytes[start - 1].is_ascii_alphanumeric() || bytes[start - 1] == b'-' || bytes[start - 1] == b'_')
    {
        start -= 1;
    }
    while end < bytes.len()
        && (bytes[end].is_ascii_alphanumeric() || bytes[end] == b'-' || bytes[end] == b'_')
    {
        end += 1;
    }
    if start == end {
        None
    } else {
        Some(&line[start..end])
    }
}

/// Return hover documentation for a CSS property name.
fn get_hover_info(line: &str, position: Position) -> Option<Hover> {
    let word = word_at_position(line, position.character)?;
    let doc = property_documentation(word)?;

    Some(Hover {
        contents: HoverContents::Markup(MarkupContent {
            kind: MarkupKind::Markdown,
            value: doc.to_string(),
        }),
        range: None,
    })
}

/// Returns Markdown documentation for known CSS properties.
fn property_documentation(property: &str) -> Option<&'static str> {
    match property {
        "display" => Some("**display**\n\nSets the display type of an element.\n\nValues: `none | block | inline | inline-block | flex | inline-flex | grid | inline-grid | contents | flow-root | table | ...`"),
        "margin" => Some("**margin**\n\nSets the margin area on all four sides of an element.\n\nShorthand for `margin-top`, `margin-right`, `margin-bottom`, `margin-left`."),
        "margin-top" | "margin-right" | "margin-bottom" | "margin-left" => Some("**margin-***\n\nSets the margin on a specific side.\n\nValues: `<length> | <percentage> | auto`"),
        "padding" => Some("**padding**\n\nSets the padding area on all four sides of an element.\n\nShorthand for `padding-top`, `padding-right`, `padding-bottom`, `padding-left`."),
        "padding-top" | "padding-right" | "padding-bottom" | "padding-left" => Some("**padding-***\n\nSets the padding on a specific side.\n\nValues: `<length> | <percentage>`"),
        "color" => Some("**color**\n\nSets the foreground color of an element's text content.\n\nValues: `<color>`"),
        "background" => Some("**background**\n\nShorthand for setting all background properties.\n\nSub-properties: `background-color`, `background-image`, `background-position`, `background-size`, `background-repeat`, `background-attachment`, `background-origin`, `background-clip`."),
        "background-color" => Some("**background-color**\n\nSets the background color of an element.\n\nValues: `<color>`"),
        "width" => Some("**width**\n\nSets the width of an element.\n\nValues: `auto | <length> | <percentage> | min-content | max-content | fit-content(<length-percentage>)`"),
        "height" => Some("**height**\n\nSets the height of an element.\n\nValues: `auto | <length> | <percentage> | min-content | max-content | fit-content(<length-percentage>)`"),
        "font-size" => Some("**font-size**\n\nSets the size of the font.\n\nValues: `<absolute-size> | <relative-size> | <length-percentage>`"),
        "font-weight" => Some("**font-weight**\n\nSets the weight (boldness) of the font.\n\nValues: `normal | bold | bolder | lighter | 100-900`"),
        "font-family" => Some("**font-family**\n\nSpecifies the font for an element.\n\nValues: `<family-name> | <generic-family>`"),
        "border" => Some("**border**\n\nShorthand for `border-width`, `border-style`, `border-color`.\n\nExample: `border: 1px solid black;`"),
        "border-radius" => Some("**border-radius**\n\nRounds the corners of an element's outer border edge.\n\nValues: `<length> | <percentage>`"),
        "position" => Some("**position**\n\nSets how an element is positioned in a document.\n\nValues: `static | relative | absolute | fixed | sticky`"),
        "top" | "right" | "bottom" | "left" => Some("**top / right / bottom / left**\n\nSets the position offset for positioned elements.\n\nValues: `auto | <length> | <percentage>`"),
        "z-index" => Some("**z-index**\n\nSets the z-order of a positioned element.\n\nValues: `auto | <integer>`"),
        "overflow" => Some("**overflow**\n\nSets what to do when content overflows.\n\nValues: `visible | hidden | clip | scroll | auto`"),
        "flex" => Some("**flex**\n\nShorthand for `flex-grow`, `flex-shrink`, `flex-basis`.\n\nExample: `flex: 1 1 auto;`"),
        "flex-direction" => Some("**flex-direction**\n\nSets the direction of flex items.\n\nValues: `row | row-reverse | column | column-reverse`"),
        "justify-content" => Some("**justify-content**\n\nAligns flex/grid items along the main axis.\n\nValues: `flex-start | flex-end | center | space-between | space-around | space-evenly`"),
        "align-items" => Some("**align-items**\n\nAligns flex/grid items along the cross axis.\n\nValues: `stretch | flex-start | flex-end | center | baseline`"),
        "gap" => Some("**gap**\n\nSets gaps (gutters) between rows and columns in flex/grid layouts.\n\nShorthand for `row-gap` and `column-gap`."),
        "grid-template-columns" => Some("**grid-template-columns**\n\nDefines the columns of a grid.\n\nValues: `none | <track-list> | <auto-track-list>`"),
        "grid-template-rows" => Some("**grid-template-rows**\n\nDefines the rows of a grid.\n\nValues: `none | <track-list> | <auto-track-list>`"),
        "transition" => Some("**transition**\n\nShorthand for `transition-property`, `transition-duration`, `transition-timing-function`, `transition-delay`."),
        "transform" => Some("**transform**\n\nApplies a 2D or 3D transformation to an element.\n\nValues: `none | <transform-function>+`"),
        "opacity" => Some("**opacity**\n\nSets the opacity level.\n\nValues: `<number>` (0.0 to 1.0)"),
        "box-shadow" => Some("**box-shadow**\n\nAdds shadow effects around an element.\n\nValues: `none | <shadow>#`\n\nExample: `box-shadow: 0 4px 6px rgba(0,0,0,0.1);`"),
        "text-align" => Some("**text-align**\n\nSets the horizontal alignment of inline content.\n\nValues: `start | end | left | right | center | justify`"),
        "text-decoration" => Some("**text-decoration**\n\nShorthand for text decoration lines, style, color, and thickness."),
        "line-height" => Some("**line-height**\n\nSets the height of a line box.\n\nValues: `normal | <number> | <length> | <percentage>`"),
        "letter-spacing" => Some("**letter-spacing**\n\nSets spacing between text characters.\n\nValues: `normal | <length>`"),
        "cursor" => Some("**cursor**\n\nSets the mouse cursor.\n\nValues: `auto | default | pointer | move | text | wait | help | not-allowed | ...`"),
        "animation" => Some("**animation**\n\nShorthand for all animation properties.\n\nSub-properties: `animation-name`, `animation-duration`, `animation-timing-function`, `animation-delay`, `animation-iteration-count`, `animation-direction`, `animation-fill-mode`, `animation-play-state`."),
        "container-type" => Some("**container-type**\n\nEstablishes the element as a query container.\n\nValues: `normal | size | inline-size`"),
        "aspect-ratio" => Some("**aspect-ratio**\n\nSets a preferred aspect ratio for the box.\n\nValues: `auto | <ratio>`"),
        "object-fit" => Some("**object-fit**\n\nSets how replaced element content should be fitted.\n\nValues: `fill | contain | cover | none | scale-down`"),
        _ => None,
    }
}
