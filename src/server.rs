use crate::config::{read_config, validate_config, HtmxConfig};
use crate::htmx_tags::Tag;
use crate::query_helper::Queries;
use crate::to_input_edit::ToInputEdit;
use std::collections::HashMap;

use std::path::Path;
use std::sync::{Arc, Mutex, RwLock};

use dashmap::mapref::one::RefMut;
use dashmap::DashMap;
use ropey::Rope;

use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::request::{GotoImplementationParams, GotoImplementationResponse};
use tower_lsp::lsp_types::{
    CodeActionParams, CodeActionProviderCapability, CodeActionResponse, CompletionContext,
    CompletionItem, CompletionItemKind, CompletionOptions, CompletionParams, CompletionResponse,
    CompletionTriggerKind, Diagnostic, DidChangeTextDocumentParams, DidCloseTextDocumentParams,
    DidOpenTextDocumentParams, DidSaveTextDocumentParams, Documentation, GotoDefinitionParams,
    GotoDefinitionResponse, Hover, HoverContents, HoverParams, HoverProviderCapability,
    ImplementationProviderCapability, InitializedParams, Location, MarkupContent, MarkupKind,
    MessageType, OneOf, Range, ReferenceParams, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, TextDocumentSyncOptions, TextDocumentSyncSaveOptions, Url,
};
use tower_lsp::lsp_types::{InitializeParams, ServerInfo};
use tower_lsp::{lsp_types::InitializeResult, Client, LanguageServer};

use crate::htmx_tree_sitter::LspFiles;
use crate::init_hx::{init_hx_tags, init_hx_values, HxCompletion};
use crate::position::{get_position_from_lsp_completion, Position, QueryType};

pub struct BackendHtmx {
    pub client: Client,
    pub document_map: DashMap<String, Rope>,
    pub hx_tags: Vec<HxCompletion>,
    pub hx_attribute_values: HashMap<String, Vec<HxCompletion>>,
    pub can_complete: RwLock<bool>,
    pub htmx_config: RwLock<Option<HtmxConfig>>,
    pub lsp_files: Arc<Mutex<LspFiles>>,
    pub queries: Arc<Mutex<Queries>>,
}

impl BackendHtmx {
    pub fn new(client: Client) -> Self {
        Self {
            client,
            document_map: DashMap::new(),
            hx_tags: init_hx_tags(),
            hx_attribute_values: init_hx_values(),
            can_complete: RwLock::new(false),
            htmx_config: RwLock::new(None),
            lsp_files: Arc::new(Mutex::new(LspFiles::default())),
            queries: Arc::new(Mutex::new(Queries::default())),
        }
    }

    fn on_change(&self, params: ServerTextDocumentItem) {
        let rope = ropey::Rope::from_str(&params.text);
        self.document_map
            .insert(params.uri.to_string(), rope.clone());
        let _ = self.lsp_files.lock().is_ok_and(|lsp_files| {
            lsp_files.on_change(params);
            true
        });
    }

    fn on_remove(&self, range: &Range, rope: &mut RefMut<'_, String, Rope>) -> Option<()> {
        let (start, end) = range.to_byte(rope);
        rope.remove(start..end);
        None
    }

    fn on_insert(
        &self,
        range: &Range,
        text: &str,
        rope: &mut RefMut<'_, String, Rope>,
    ) -> Option<()> {
        let (start, _) = range.to_byte(rope);
        rope.insert(start, text);
        None
    }

    async fn publish_tag_diagnostics(&self, diagnostics: Vec<Tag>, file: Option<String>) {
        let mut hm: HashMap<String, Vec<Diagnostic>> = HashMap::new();
        let len = diagnostics.len();
        let _ = self.lsp_files.lock().is_ok_and(|lsp_files| {
            lsp_files.publish_tag_diagnostics(diagnostics, &mut hm);
            true
        });
        for (url, diagnostics) in hm {
            if let Ok(uri) = Url::parse(&url) {
                self.client
                    .publish_diagnostics(uri, diagnostics, None)
                    .await;
            }
        }
        if let Some(uri) = file {
            if len == 0 {
                let uri = Url::parse(&uri).unwrap();
                self.client.publish_diagnostics(uri, vec![], None).await;
            }
        }
    }

    fn check_definition(&self, position: Option<Position>) -> Option<GotoDefinitionResponse> {
        let mut def = None;
        let _ = position.is_some_and(|position| {
            if let Position::AttributeValue {
                name,
                value,
                definition,
            } = position
            {
                if &name == "hx-lsp" {
                    let _ = self.lsp_files.lock().is_ok_and(|lsp_files| {
                        lsp_files
                            .goto_definition_response(definition, &value, &mut def)
                            .is_some()
                    });
                }
            }
            true
        });

        def
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for BackendHtmx {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let mut definition_provider = None;
        let mut references_provider = None;
        let mut code_action_provider = None;
        let mut implementation_provider = None;
        if let Some(client_info) = params.client_info {
            if client_info.name == "helix" {
                if let Ok(mut can_complete) = self.can_complete.write() {
                    *can_complete = true;
                }
            }
        }
        match validate_config(params.initialization_options) {
            Some(htmx_config) => {
                let _ = self.htmx_config.try_write().is_ok_and(|mut config| {
                    definition_provider = Some(OneOf::Left(true));
                    references_provider = Some(OneOf::Left(true));
                    code_action_provider = Some(CodeActionProviderCapability::Simple(true));
                    implementation_provider = Some(ImplementationProviderCapability::Simple(true));
                    *config = Some(htmx_config);
                    true
                });
            }
            None => {
                self.client
                    .log_message(MessageType::INFO, "Config not found")
                    .await;
            }
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        change: Some(TextDocumentSyncKind::INCREMENTAL),
                        will_save: Some(true),
                        save: Some(TextDocumentSyncSaveOptions::Supported(true)),
                        ..Default::default()
                    },
                )),
                completion_provider: Some(CompletionOptions {
                    resolve_provider: Some(false),
                    trigger_characters: Some(vec![
                        "-".to_string(),
                        "\"".to_string(),
                        " ".to_string(),
                    ]),
                    all_commit_characters: None,
                    work_done_progress_options: Default::default(),
                    completion_item: None,
                }),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                definition_provider,
                references_provider,
                code_action_provider,
                implementation_provider,
                ..ServerCapabilities::default()
            },
            server_info: Some(ServerInfo {
                name: String::from("htmx-lsp"),
                version: Some(String::from("0.1.2")),
            }),
            offset_encoding: None,
        })
    }

    async fn initialized(&self, _params: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "initialized!")
            .await;

        match read_config(
            &self.htmx_config,
            &self.lsp_files,
            &self.queries,
            &self.document_map,
        ) {
            Ok(diagnostics) => {
                self.publish_tag_diagnostics(diagnostics, None).await;
            }
            Err(err) => {
                let _ = self.htmx_config.write().is_ok_and(|mut config| {
                    *config = None;
                    true
                });
                let msg = err.to_string();
                self.client.log_message(MessageType::INFO, msg).await;
            }
        };
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let _temp_uri = params.text_document.uri.clone();
        self.on_change(ServerTextDocumentItem {
            uri: params.text_document.uri,
            text: params.text_document.text,
        });
    }

    async fn did_close(&self, _: DidCloseTextDocumentParams) {}

    async fn did_save(&self, params: DidSaveTextDocumentParams) {
        let uri = params.text_document.uri.to_string();
        let _path = Path::new(&uri);
        let mut diags = vec![];
        if let Ok(lsp_files) = self.lsp_files.lock() {
            if let Some(diagnostics) = lsp_files.saved(
                &uri,
                &mut diags,
                &self.htmx_config,
                &self.document_map,
                &self.queries,
            ) {
                diags = diagnostics;
            }
        }
        self.publish_tag_diagnostics(diags, Some(uri)).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = &params.text_document.uri.to_string();
        let rope = self.document_map.get_mut(uri);
        if let Some(mut rope) = rope {
            for change in params.content_changes {
                if let Some(range) = &change.range {
                    let input_edit = range.to_input_edit(&rope);
                    if change.text.is_empty() {
                        self.on_remove(range, &mut rope);
                    } else {
                        self.on_insert(range, &change.text, &mut rope);
                    }
                    let mut w = LocalWriter::default();
                    let _ = rope.write_to(&mut w);
                    let _ = self.lsp_files.lock().is_ok_and(|lsp_files| {
                        lsp_files.input_edit(uri, w.content, input_edit);
                        true
                    });
                }
            }
        }
        // if let Some(text) = params.content_changes.first_mut() {
        // self.on_change(ServerTextDocumentItem {
        //     uri: params.text_document.uri,
        //     text: std::mem::take(&mut text.text),
        // })
        // .await
        // }
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let can_complete = {
            matches!(
                params.context,
                Some(CompletionContext {
                    trigger_kind: CompletionTriggerKind::TRIGGER_CHARACTER,
                    ..
                }) | Some(CompletionContext {
                    trigger_kind: CompletionTriggerKind::INVOKED,
                    ..
                })
            )
        };
        // TODO disable for backend and javascript
        if !can_complete {
            let can_complete = self.can_complete.read().is_ok_and(|d| *d);
            if !can_complete {
                return Ok(None);
            }
        }

        let uri = &params.text_document_position.text_document.uri;
        match uri.to_file_path().unwrap().extension().is_some_and(|ext| {
            self.htmx_config.read().is_ok_and(|config| {
                if let Some(config) = config.as_ref() {
                    return ext.to_str().unwrap() != config.template_ext;
                }
                false
            })
        }) {
            true => return Ok(None),
            false => (),
        }
        let mut result = None;
        let _ = self.queries.lock().is_ok_and(|queries| {
            result = get_position_from_lsp_completion(
                &params.text_document_position,
                &self.document_map,
                uri.to_string(),
                QueryType::Completion,
                &self.lsp_files,
                &queries.html,
            );
            true
        });

        if let Some(result) = result {
            match result {
                Position::AttributeName(name) => {
                    if name.starts_with("hx-") {
                        let completions = self.hx_tags.clone();
                        let mut ret = Vec::with_capacity(completions.len());
                        for item in completions {
                            ret.push(CompletionItem {
                                label: item.name.to_string(),
                                kind: Some(CompletionItemKind::TEXT),
                                documentation: Some(Documentation::MarkupContent(MarkupContent {
                                    kind: MarkupKind::Markdown,
                                    value: item.desc.to_string(),
                                })),
                                ..Default::default()
                            });
                        }
                        return Ok(Some(CompletionResponse::Array(ret)));
                    }
                }
                Position::AttributeValue { name, .. } => {
                    if let Some(completions) = self.hx_attribute_values.get(&name) {
                        let mut ret = Vec::with_capacity(completions.len());
                        for item in completions {
                            ret.push(CompletionItem {
                                label: item.name.to_string(),
                                detail: Some(item.desc.to_string()),
                                kind: Some(CompletionItemKind::TEXT),
                                ..Default::default()
                            });
                        }
                        return Ok(Some(CompletionResponse::Array(ret)));
                    }
                    return Ok(None);
                }
            }
        }
        Ok(None)
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = &params.text_document_position_params.text_document.uri;
        let mut result = None;
        let _ = self.queries.lock().is_ok_and(|queries| {
            result = get_position_from_lsp_completion(
                &params.text_document_position_params,
                &self.document_map,
                uri.to_string(),
                QueryType::Hover,
                &self.lsp_files,
                &queries.html,
            );
            true
        });

        if let Some(result) = result {
            match result {
                Position::AttributeName(name) => {
                    if let Some(res) = self
                        .hx_tags
                        .iter()
                        .find(|x| x.name == name.replace("hx-", ""))
                        .cloned()
                    {
                        let markup_content = MarkupContent {
                            kind: MarkupKind::Markdown,
                            value: res.desc,
                        };
                        let hover_contents = HoverContents::Markup(markup_content);
                        let hover = Hover {
                            contents: hover_contents,
                            range: None,
                        };
                        return Ok(Some(hover));
                    }
                }
                Position::AttributeValue { name, value, .. } => {
                    if let Some(res) = self.hx_attribute_values.get(&name) {
                        if let Some(res) = res.iter().find(|x| x.name == value).cloned() {
                            let markup_content = MarkupContent {
                                kind: MarkupKind::Markdown,
                                value: res.desc,
                            };
                            let hover_contents = HoverContents::Markup(markup_content);
                            let hover = Hover {
                                contents: hover_contents,
                                range: None,
                            };
                            return Ok(Some(hover));
                        }
                    }
                }
            }
        }

        Ok(None)
    }

    async fn goto_definition(
        &self,
        params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let mut res: Result<Option<GotoDefinitionResponse>> = Ok(None);
        let _tree = self.lsp_files.lock().is_ok_and(|lsp_files| {
            let _ = self.queries.lock().is_ok_and(|queries| {
                let position = lsp_files.goto_definition(
                    params,
                    &self.htmx_config,
                    &self.document_map,
                    &queries.html,
                );
                drop(queries);
                drop(lsp_files);
                res = Ok(self.check_definition(position));
                true
            });
            true
        });
        res
    }

    async fn references(&self, params: ReferenceParams) -> Result<Option<Vec<Location>>> {
        let mut locations = None;
        if let Ok(config) = self.htmx_config.read() {
            if let Some(_a) = config.as_ref() {
                let _ = self.lsp_files.lock().is_ok_and(|lsp_files| {
                    let _ = self.queries.lock().is_ok_and(|queries| {
                        locations = lsp_files.references(params, &queries, &self.document_map);
                        true
                    });
                    true
                });
            }
        }
        Ok(locations)
    }

    async fn goto_implementation(
        &self,
        params: GotoImplementationParams,
    ) -> Result<Option<GotoImplementationResponse>> {
        let mut res = None;
        if let Ok(config) = self.htmx_config.read() {
            if let Some(_a) = config.as_ref() {
                let _ = self.lsp_files.lock().is_ok_and(|lsp_files| {
                    let _ = self.queries.lock().is_ok_and(|queries| {
                        res = lsp_files.goto_implementation(params, &queries, &self.document_map);
                        true
                    });
                    true
                });
            }
        }
        Ok(res)
    }

    async fn code_action(&self, _params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        Ok(None)
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }
}

pub struct ServerTextDocumentItem {
    pub uri: Url,
    pub text: String,
}

#[derive(Default, Debug)]
pub struct LocalWriter {
    pub content: String,
}

impl std::io::Write for LocalWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        if let Ok(b) = std::str::from_utf8(buf) {
            self.content.push_str(b);
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
