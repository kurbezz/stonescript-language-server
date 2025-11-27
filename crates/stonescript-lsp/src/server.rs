//! StoneScript LSP Server Backend

use dashmap::DashMap;
use ropey::Rope;
use stonescript_parser::{ast::Program, parse_source};
use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer};

use crate::providers::*;
use crate::utils::ScopeAnalyzer;

/// Document information
struct Document {
    rope: Rope,
    ast: Program,
    scope: ScopeAnalyzer,
    version: i32,
}

pub struct Backend {
    client: Client,
    documents: DashMap<String, Document>,

    // Providers
    completion: CompletionProvider,
    hover: HoverProvider,
    diagnostics: DiagnosticsProvider,
    signature_help: SignatureHelpProvider,
    definition: DefinitionProvider,
    symbols: SymbolsProvider,
    formatting: FormattingProvider,
    semantic_tokens: SemanticTokensProvider,
}

impl Backend {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            documents: DashMap::new(),
            completion: CompletionProvider::new(),
            hover: HoverProvider::new(),
            diagnostics: DiagnosticsProvider::new(),
            signature_help: SignatureHelpProvider::new(),
            definition: DefinitionProvider::new(),
            symbols: SymbolsProvider::new(),
            formatting: FormattingProvider::new(),
            semantic_tokens: SemanticTokensProvider::new(),
        }
    }

    fn analyze_document(&self, uri: &str, text: &str, version: i32) {
        let rope = Rope::from_str(text);

        // Parse with nom-based parser
        let ast = match parse_source(text) {
            Ok(program) => program,
            Err(e) => {
                // Log parse error and publish diagnostic
                tracing::error!("Parse error for {}: {}", uri, e);

                // Create a diagnostic for the parse error
                let diagnostic = Diagnostic {
                    range: Range {
                        start: Position {
                            line: 0,
                            character: 0,
                        },
                        end: Position {
                            line: 0,
                            character: 0,
                        },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    message: format!("Parse error: {}", e),
                    ..Default::default()
                };

                let uri_parsed = Url::parse(uri).unwrap();
                let client = self.client.clone();
                tokio::spawn(async move {
                    client
                        .publish_diagnostics(uri_parsed, vec![diagnostic], None)
                        .await;
                });

                return;
            }
        };

        // Analyze scope
        let mut scope = ScopeAnalyzer::new();
        scope.analyze_ast(&ast);

        // Store document
        self.documents.insert(
            uri.to_string(),
            Document {
                rope,
                ast: ast.clone(),
                scope,
                version,
            },
        );

        // Publish diagnostics
        if let Some(doc) = self.documents.get(uri) {
            let diagnostics = self
                .diagnostics
                .provide_diagnostics(&doc.ast, text, &doc.scope);

            let uri_parsed = Url::parse(uri).unwrap();
            let client = self.client.clone();
            tokio::spawn(async move {
                client
                    .publish_diagnostics(uri_parsed, diagnostics, None)
                    .await;
            });
        }
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::INCREMENTAL,
                )),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string(), " ".to_string()]),
                    ..Default::default()
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                signature_help_provider: Some(SignatureHelpOptions {
                    trigger_characters: Some(vec!["(".to_string(), ",".to_string()]),
                    retrigger_characters: None,
                    work_done_progress_options: WorkDoneProgressOptions::default(),
                }),
                definition_provider: Some(OneOf::Left(true)),
                document_symbol_provider: Some(OneOf::Left(true)),
                document_formatting_provider: Some(OneOf::Left(true)),
                semantic_tokens_provider: Some(
                    SemanticTokensServerCapabilities::SemanticTokensOptions(
                        SemanticTokensOptions {
                            legend: self.semantic_tokens.legend(),
                            full: Some(SemanticTokensFullOptions::Bool(true)),
                            ..Default::default()
                        },
                    ),
                ),
                ..Default::default()
            },
            ..Default::default()
        })
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let text = params.text_document.text;
        let version = params.text_document.version;

        self.analyze_document(&uri, &text, version);
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri.to_string();

        if let Some(mut doc) = self.documents.get_mut(&uri) {
            for change in params.content_changes {
                if let Some(range) = change.range {
                    // Incremental update
                    let start_idx = doc.rope.line_to_char(range.start.line as usize)
                        + range.start.character as usize;
                    let end_idx = doc.rope.line_to_char(range.end.line as usize)
                        + range.end.character as usize;

                    doc.rope.remove(start_idx..end_idx);
                    doc.rope.insert(start_idx, &change.text);
                } else {
                    // Full update
                    doc.rope = Rope::from_str(&change.text);
                }
            }

            let text = doc.rope.to_string();
            drop(doc); // Release mutable borrow

            self.analyze_document(&uri, &text, params.text_document.version);
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        self.documents.remove(&params.text_document.uri.to_string());
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri.to_string();
        let position = params.text_document_position.position;

        if let Some(doc) = self.documents.get(&uri) {
            let text = doc.rope.to_string();
            let items = self
                .completion
                .provide_completion(&doc.ast, position, &text, &doc.scope);
            Ok(Some(CompletionResponse::Array(items)))
        } else {
            Ok(None)
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        if let Some(doc) = self.documents.get(&uri) {
            let text = doc.rope.to_string();
            Ok(self
                .hover
                .provide_hover(&doc.ast, position, &text, &doc.scope))
        } else {
            Ok(None)
        }
    }

    async fn signature_help(&self, params: SignatureHelpParams) -> Result<Option<SignatureHelp>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let position = params.text_document_position_params.position;

        if let Some(doc) = self.documents.get(&uri) {
            let text = doc.rope.to_string();
            Ok(self
                .signature_help
                .provide_signature_help(&doc.ast, position, &text, &doc.scope))
        } else {
            Ok(None)
        }
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .clone();
        let position = params.text_document_position_params.position;

        if let Some(doc) = self.documents.get(&uri.to_string()) {
            let text = doc.rope.to_string();
            Ok(self
                .definition
                .provide_definition(&doc.ast, position, &text, &doc.scope, &uri))
        } else {
            Ok(None)
        }
    }

    async fn document_symbol(
        &self,
        params: DocumentSymbolParams,
    ) -> Result<Option<DocumentSymbolResponse>> {
        let uri = params.text_document.uri.to_string();

        if let Some(doc) = self.documents.get(&uri) {
            let text = doc.rope.to_string();
            let symbols = self.symbols.provide_symbols(&doc.ast, &doc.scope, &text);
            Ok(Some(DocumentSymbolResponse::Nested(symbols)))
        } else {
            Ok(None)
        }
    }

    async fn formatting(&self, params: DocumentFormattingParams) -> Result<Option<Vec<TextEdit>>> {
        let uri = params.text_document.uri.to_string();

        if let Some(doc) = self.documents.get(&uri) {
            let text = doc.rope.to_string();
            let edits = self.formatting.provide_formatting(&text);
            Ok(Some(edits))
        } else {
            Ok(None)
        }
    }

    async fn semantic_tokens_full(
        &self,
        params: SemanticTokensParams,
    ) -> Result<Option<SemanticTokensResult>> {
        let uri = params.text_document.uri.to_string();

        if let Some(doc) = self.documents.get(&uri) {
            let text = doc.rope.to_string();
            let tokens = self
                .semantic_tokens
                .provide_semantic_tokens(&doc.ast, &text);
            Ok(Some(SemanticTokensResult::Tokens(tokens)))
        } else {
            Ok(None)
        }
    }
}
