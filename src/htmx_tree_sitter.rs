use std::{
    cell::RefCell,
    collections::HashMap,
    path::Path,
    sync::{Arc, Mutex, RwLock},
};

use dashmap::{
    mapref::one::{Ref, RefMut},
    DashMap,
};
use ropey::Rope;
use tower_lsp::lsp_types::{
    request::{GotoImplementationParams, GotoImplementationResponse},
    CodeActionParams, Diagnostic, DiagnosticSeverity, GotoDefinitionParams, GotoDefinitionResponse,
    Location, Position, Range, ReferenceParams, Url,
};
use tree_sitter::{InputEdit, Parser, Point, Query, Tree};

use crate::{
    config::HtmxConfig,
    htmx_tags::{in_tags, Tag},
    init_hx::{LangType, LangTypes},
    position::{query_position, Position as PositionType, PositionDefinition, QueryType},
    queries::{HX_JS_TAGS, HX_RUST_TAGS},
    query_helper::{
        find_hx_lsp, query_htmx_lsp, query_tag, HTMLQueries, HTMLQuery, HtmxQuery, Queries,
    },
    server::{FileWriter, ServerTextDocumentItem},
    to_input_edit::to_position,
};

type FileName = usize;

/// LspFiles
///
/// This struct contains:
///
///   * TreeSitter and Parsers for
///      * html
///      * javascript
///      * backend
///   * file indexes(faster when comparing different tags because it's smaller than String)
///   * backend/frontend tags
///
/// It handles all language server requests.
#[derive(Clone)]
pub struct LspFiles {
    current: RefCell<usize>,
    indexes: DashMap<String, FileName>,
    template: DashMap<FileName, Tree>,
    javascript: DashMap<FileName, Tree>,
    backend: DashMap<FileName, Tree>,
    pub parsers: Arc<Mutex<Parsers>>,
    pub tags: DashMap<String, Tag>,
}

impl Default for LspFiles {
    fn default() -> Self {
        Self {
            current: RefCell::new(0),
            indexes: DashMap::new(),
            parsers: Arc::new(Mutex::new(Parsers::default())),
            tags: DashMap::new(),
            template: DashMap::new(),
            javascript: DashMap::new(),
            backend: DashMap::new(),
        }
    }
}

impl LspFiles {
    /// Reset indexes, trees and tags.
    pub fn reset(&self) {
        self.indexes.clear();
        self.template.clear();
        self.javascript.clear();
        self.backend.clear();
        self.tags.clear();
    }

    /// After each save for backend/javascript, tags are deleted for that file.
    pub fn delete_tags_by_index(&self, index: usize) {
        let mut tags = vec![];
        for i in &self.tags {
            let file = i.value().file;
            if file == index {
                tags.push(String::from(i.key()));
            }
        }
        for i in tags {
            self.tags.remove(&i);
        }
    }

    /// Errors if tag already exist.
    pub fn add_tag(&self, tag: Tag) -> Result<(), Tag> {
        if self.tags.contains_key(&tag.name) {
            Err(tag)
        } else {
            self.tags.insert(String::from(&tag.name), tag);
            Ok(())
        }
    }

    /// Get reference to tag. Only used in definition request, deadlock can't happen here.
    pub fn get_tag<'a>(&'a self, key: &String) -> Option<Ref<'a, std::string::String, Tag>> {
        self.tags.get(key)
    }

    /// Returns index for file. If file already exists, then old index is returned.
    pub fn add_file(&self, key: String) -> Option<usize> {
        match self.get_index(&key) {
            Some(index) => Some(index),
            None => {
                let old = self.current.replace_with(|&mut old| old + 1);
                self.indexes.insert(key, old);
                Some(old)
            }
        }
    }

    pub fn get_index(&self, file: &String) -> Option<usize> {
        if let Some(d) = self.indexes.get(file) {
            let a = *d;
            return Some(a);
        }
        None
    }

    /// Get file path for this index.
    pub fn get_uri(&self, index: usize) -> Option<String> {
        self.indexes.iter().find_map(|item| {
            if item.value() == &index {
                Some(String::from(item.key()))
            } else {
                None
            }
        })
    }

    pub fn after_open(&self, _params: ServerTextDocumentItem) -> Option<()> {
        // let file = self.get_index(&params.uri.to_string())?;
        // self.add_tree(file, None, &params.text, None);
        None
    }

    /// Notify client about possible tag name conflict.
    pub fn publish_tag_diagnostics(
        &self,
        diagnostics: Vec<Tag>,
        hm: &mut HashMap<String, Vec<Diagnostic>>,
    ) {
        for diag in diagnostics {
            if let Some(uri) = self.get_uri(diag.file) {
                let position = to_position(&diag);
                let diagnostic = Diagnostic {
                    range: Range::new(position.0, position.1),
                    severity: Some(DiagnosticSeverity::WARNING),
                    message: String::from("This tag already exist."),
                    source: Some(String::from("htmx-lsp")),
                    ..Default::default()
                };
                if hm.contains_key(&uri) {
                    let _ = hm.get_mut(&uri).is_some_and(|d| {
                        d.push(diagnostic);
                        false
                    });
                } else {
                    hm.insert(String::from(&uri), vec![diagnostic]);
                }
            }
        }
    }

    /// Returns Position from request, this works only if called from templates.
    pub fn goto_definition(
        &self,
        params: GotoDefinitionParams,
        config: &RwLock<HtmxConfig>,
        document_map: &DashMap<String, Rope>,
        query: &HTMLQueries,
    ) -> Option<PositionType> {
        let response = None;
        let file = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let ext = config.read().is_ok_and(|config| {
            if !config.is_valid {
                return false;
            }
            let ext = config.file_ext(Path::new(&file));
            ext.is_some_and(|lang_types| lang_types.is_lang(LangType::Template))
        });
        if !ext {
            return None;
        }
        let text = {
            let c = document_map.get(&file)?;
            let mut w = FileWriter::default();
            let _ = c.value().write_to(&mut w);
            w
        };
        let index = self.get_index(&file);
        if let Some(index) = index {
            if let Some(tree) = self.get_tree(LangType::Template, index) {
                let root_node = tree.root_node();
                let pos = params.text_document_position_params.position;
                let trigger_point = Point::new(pos.line as usize, pos.character as usize);
                return query_position(
                    root_node,
                    &text.content,
                    trigger_point,
                    QueryType::Definition,
                    query,
                );
            }
        }
        response
    }

    /// Prepare response for goto definition request.
    pub fn goto_definition_response(
        &self,
        definition: Option<PositionDefinition>,
        value: &str,
        def: &mut Option<GotoDefinitionResponse>,
    ) -> Option<()> {
        let tag = in_tags(value, definition?)?;
        let tag = self.get_tag(&tag.name)?;
        let file = self.get_uri(tag.file)?;
        let (start, end) = to_position(&tag);
        let range = Range::new(start, end);
        *def = Some(GotoDefinitionResponse::Scalar(Location {
            uri: Url::parse(&file).unwrap(),
            range,
        }));
        None
    }

    /// Search and insert every tag, collect errors.
    #[allow(clippy::result_unit_err)]
    pub fn add_tags_from_file(
        &self,
        index: usize,
        lang_type: LangType,
        text: &str,
        _overwrite: bool,
        queries: &Queries,
        diags: &mut Vec<Tag>,
    ) -> Result<(), ()> {
        let query = HtmxQuery::try_from(lang_type)?;
        let query = queries.get(query);
        if let Some(old_tree) = self.get_tree(lang_type, index) {
            let tags = query_tag(
                old_tree.root_node(),
                text,
                Point::new(0, 0),
                &QueryType::Completion,
                query,
                true,
            );
            self.delete_tags_by_index(index);
            for mut tag in tags {
                tag.file = index;
                if let Err(tag) = self.add_tag(tag) {
                    diags.push(tag);
                }
            }
            drop(old_tree);
        }
        Ok(())
    }

    /// Called after didSave request. Returns tag errors.
    pub fn saved(
        &self,
        uri: &String,
        diagnostics: &mut Vec<Tag>,
        config: &RwLock<HtmxConfig>,
        document_map: &DashMap<String, Rope>,
        queries: &Arc<Mutex<Queries>>,
    ) -> Option<Vec<Tag>> {
        let path = Path::new(&uri);
        let file = self.get_index(uri)?;
        let mut lang_type = LangType::Template;
        if let Ok(config) = config.read() {
            let lang_types = config.file_ext(path)?;
            lang_type = lang_types.get();
            if lang_type == LangType::Template {
                return None;
            }
        }
        let content = document_map.get(uri)?;
        let content = content.value();
        let mut a = FileWriter::default();
        let _ = content.write_to(&mut a);
        let content = a.content;
        queries.lock().ok().and_then(|queries| {
            self.add_tags_from_file(file, lang_type, &content, false, &queries, diagnostics)
                .ok()
        });
        Some(diagnostics.to_vec())
    }

    /// Can be called from backend/javascript file.
    pub fn references(
        &self,
        params: ReferenceParams,
        queries: &Queries,
        document_map: &DashMap<String, Rope>,
        lang_type: LangType,
    ) -> Option<Vec<Location>> {
        let mut locations = None;
        let uri = String::from(&params.text_document_position.text_document.uri.to_string());
        let point = Point::new(
            params.text_document_position.position.line as usize,
            params.text_document_position.position.character as usize,
        );
        let index = self.get_index(&uri)?;
        let tree = self.get_tree(lang_type, index)?;
        if let Ok(c) = HtmxQuery::try_from(lang_type) {
            let query = queries.get(c);
            let content = document_map.get(&uri)?;
            let mut w = FileWriter::default();
            let _ = content.value().write_to(&mut w);
            drop(content);
            let tags = query_tag(
                tree.root_node(),
                &w.content,
                point,
                &QueryType::Completion,
                query,
                false,
            );
            let tag = tags.first()?;
            let mut references = vec![];
            for tree in self.template.iter() {
                let file = self.get_uri(*tree.key())?;
                let mut w = FileWriter::default();
                let content = document_map.get(&file)?;
                let _ = content.value().write_to(&mut w);
                query_htmx_lsp(
                    tree.root_node(),
                    &w.content,
                    Point::new(0, 0),
                    &QueryType::Hover,
                    queries.html.get(HTMLQuery::Lsp),
                    &tag.name,
                    &mut references,
                    *tree.key(),
                );
            }
            references.sort();
            let mut response = vec![];
            for i in &references {
                let index = self.get_uri(i.file)?;
                let (start, mut end) = to_position(i);
                end.character += 1;
                let range = Range::new(start, end);
                let location = Location::new(Url::parse(&index).unwrap(), range);
                response.push(location);
            }
            locations = Some(response);
        }
        locations
    }

    /// Goto first hx-lsp attribute.
    pub fn goto_implementation(
        &self,
        params: GotoImplementationParams,
        queries: &Queries,
        document_map: &DashMap<String, Rope>,
        lang_type: LangTypes,
    ) -> Option<GotoImplementationResponse> {
        if !lang_type.is_lang(LangType::Template) {
            return None;
        }
        let uri = String::from(
            &params
                .text_document_position_params
                .text_document
                .uri
                .to_string(),
        );
        let point = Point::new(
            params.text_document_position_params.position.line as usize,
            params.text_document_position_params.position.character as usize,
        );
        let index = self.get_index(&uri)?;
        let tree = self.get_tree(LangType::Template, index)?;
        let query = queries.html.get(HTMLQuery::Lsp);
        let content = document_map.get(&uri)?;
        let mut w = FileWriter::default();
        let _ = content.value().write_to(&mut w);
        drop(content);
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .clone();
        let capture = find_hx_lsp(tree.root_node(), w.content, point, query)?;
        let start = Position {
            line: capture.start_position.row as u32,
            character: capture.start_position.column as u32,
        };
        let end = Position {
            line: capture.end_position.row as u32,
            character: capture.end_position.column as u32,
        };
        let range = Range { start, end };
        Some(GotoImplementationResponse::Scalar(Location { uri, range }))
    }

    /// Called from template, hx-lsp attribute.
    pub fn code_action(
        &self,
        params: CodeActionParams,
        config: &RwLock<HtmxConfig>,
        query: &HTMLQueries,
        document_map: &DashMap<String, Rope>,
    ) -> Option<()> {
        let uri = String::from(&params.text_document.uri.to_string());
        let ext = config.read().is_ok_and(|config| {
            if !config.is_valid {
                return false;
            }
            let ext = config.file_ext(Path::new(&uri));
            ext.is_some_and(|lang_types| lang_types.is_lang(LangType::Template))
        });
        if !ext {
            return None;
        }
        let text = {
            let c = document_map.get(&uri)?;
            let mut w = FileWriter::default();
            let _ = c.value().write_to(&mut w);
            w
        };
        let index = self.get_index(&uri)?;
        let tree = self.get_tree(LangType::Template, index)?;
        let root_node = tree.root_node();
        let pos = params.range.start;
        let trigger_point = Point::new(pos.line as usize, pos.character as usize);
        let position = query_position(
            root_node,
            &text.content,
            trigger_point,
            QueryType::Definition,
            query,
        )?;
        match position {
            PositionType::AttributeName(name) => {
                if name == "hx-lsp" {
                    Some(())
                } else {
                    None
                }
            }
            PositionType::AttributeValue { name, .. } => {
                if name == "hx-lsp" {
                    Some(())
                } else {
                    None
                }
            }
        }
    }

    pub fn query_position(
        &self,
        index: usize,
        text: &str,
        query_type: QueryType,
        pos: Position,
        query: &HTMLQueries,
    ) -> Option<PositionType> {
        let tree = self.get_tree(LangType::Template, index)?;
        let root_node = tree.root_node();
        let trigger_point = Point::new(pos.line as usize, pos.character as usize);

        query_position(root_node, text, trigger_point, query_type, query)
    }

    pub fn get_tree(&self, lang_type: LangType, index: usize) -> Option<Ref<'_, usize, Tree>> {
        match lang_type {
            LangType::Template => self.template.get(&index),
            LangType::JavaScript => self.javascript.get(&index),
            LangType::Backend => self.backend.get(&index),
        }
    }

    pub fn get_mut_tree(
        &self,
        lang_type: LangType,
        index: usize,
    ) -> Option<RefMut<'_, usize, Tree>> {
        match lang_type {
            LangType::Template => self.template.get_mut(&index),
            LangType::JavaScript => self.javascript.get_mut(&index),
            LangType::Backend => self.backend.get_mut(&index),
        }
    }

    pub fn add_tree(
        &self,
        index: usize,
        lang_type: LangType,
        text: &str,
        _range: Option<Range>,
    ) -> Option<()> {
        self.parsers
            .lock()
            .ok()
            .and_then(|mut parsers| -> Option<()> {
                if let Some(old_tree) = self.get_mut_tree(lang_type, index) {
                    if let Some(tree) = parsers.parse(lang_type, text, Some(&old_tree)) {
                        drop(old_tree);
                        self.insert_tree(lang_type, index, tree);
                    }
                } else {
                    // tree doesn't exist, first insertion
                    if let Some(tree) = parsers.parse(lang_type, text, None) {
                        self.insert_tree(lang_type, index, tree);
                    }
                }
                None
            })
    }

    pub fn insert_tree(&self, lang_type: LangType, index: usize, tree: Tree) -> Option<Tree> {
        match lang_type {
            LangType::Template => self.template.insert(index, tree),
            LangType::JavaScript => self.javascript.insert(index, tree),
            LangType::Backend => self.backend.insert(index, tree),
        }
    }

    /// TreeSitter incremental parsing.
    pub fn input_edit(
        &self,
        file: &String,
        code: String,
        input_edit: InputEdit,
        lang_type: LangType,
    ) -> Option<()> {
        let file = self.get_index(file)?;
        let mut old_tree = self.get_mut_tree(lang_type, file)?;
        let _ = self.parsers.lock().ok().and_then(|mut parsers| {
            old_tree.edit(&input_edit);
            let tree = parsers.parse(lang_type, &code, Some(&old_tree))?;
            drop(old_tree);
            self.insert_tree(lang_type, file, tree)
        });
        None
    }
}

/// Parsers for HTML, JavaScript and backend language(Python, Rust, Go).
pub struct Parsers {
    html: Parser,
    javascript: Parser,
    backend: Parser,
}

impl Parsers {
    /// Get new tree after parsing.
    pub fn parse(
        &mut self,
        lang_type: LangType,
        text: &str,
        _old_tree: Option<&Tree>,
    ) -> Option<Tree> {
        match lang_type {
            LangType::Template => self.html.parse(text, None),
            LangType::JavaScript => self.javascript.parse(text, None),
            LangType::Backend => self.backend.parse(text, None),
        }
    }

    /// Change backend based on `lang_type` and `language`. It's called once, at reading config.
    pub fn change_backend(&mut self, language: &str, lang_type: LangType) -> Option<()> {
        if lang_type != LangType::Backend {
            return None;
        }
        let language = match language {
            "python" => Some(tree_sitter_python::language()),
            "go" => Some(tree_sitter_go::language()),
            _ => None,
        };
        let mut backend = Parser::new();
        let _ = backend.set_language(language?);
        self.backend = backend;
        None
    }
}

impl Default for Parsers {
    fn default() -> Self {
        let mut html = Parser::new();
        let _ = html.set_language(tree_sitter_html::language());
        let mut javascript = Parser::new();
        let _query_javascript = Query::new(tree_sitter_javascript::language(), HX_JS_TAGS).unwrap();
        let _ = javascript.set_language(tree_sitter_javascript::language());
        let mut backend = Parser::new();
        let _query_backend = Query::new(tree_sitter_rust::language(), HX_RUST_TAGS).unwrap();
        let _ = backend.set_language(tree_sitter_rust::language());

        Self {
            html,
            javascript,
            backend,
        }
    }
}

impl Clone for Parsers {
    fn clone(&self) -> Self {
        Self::default()
    }
}
