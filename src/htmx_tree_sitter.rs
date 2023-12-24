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
    Diagnostic, DiagnosticSeverity, GotoDefinitionParams, GotoDefinitionResponse, Location,
    Position, Range, ReferenceParams, Url,
};
use tree_sitter::{InputEdit, Parser, Point, Query, Tree};

use crate::{
    config::HtmxConfig,
    htmx_tags::{in_tags, Tag},
    init_hx::LangType,
    position::{query_position, Position as PositionType, PositionDefinition, QueryType},
    queries::{HX_JS_TAGS, HX_RUST_TAGS},
    query_helper::{
        find_hx_lsp, query_htmx_lsp, query_tag, HTMLQueries as HTMLQueries2, HTMLQuery, HtmxQuery,
        Queries,
    },
    server::{LocalWriter, ServerTextDocumentItem},
};

#[derive(Debug)]
pub struct BackendTreeSitter {
    pub tree: Tree,
}

#[derive(Clone)]
pub struct LspFiles {
    current: RefCell<usize>,
    indexes: DashMap<String, usize>,
    trees: DashMap<usize, (Tree, LangType)>,
    pub parsers: Arc<Mutex<Parsers>>,
    pub tags: DashMap<String, Tag>,
}

impl Default for LspFiles {
    fn default() -> Self {
        Self {
            current: RefCell::new(0),
            indexes: DashMap::new(),
            trees: DashMap::new(),
            parsers: Arc::new(Mutex::new(Parsers::default())),
            tags: DashMap::new(),
        }
    }
}

impl LspFiles {
    pub fn reset(&self) {
        self.indexes.clear();
        self.trees.clear();
        self.tags.clear();
    }

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

    pub fn add_tag(&self, tag: Tag) -> Result<(), Tag> {
        if self.tags.contains_key(&tag.name) {
            Err(tag)
        } else {
            self.tags.insert(String::from(&tag.name), tag);
            Ok(())
        }
    }

    pub fn get_tag<'a>(&'a self, key: &String) -> Option<Ref<'a, std::string::String, Tag>> {
        self.tags.get(key)
    }

    pub fn add_file(&self, key: String) -> Option<usize> {
        if self.get_index(&key).is_none() {
            let old = self.current.replace_with(|&mut old| old + 1);
            self.indexes.insert(key, old);
            return Some(old);
        }
        None
    }

    pub fn get_index(&self, key: &String) -> Option<usize> {
        if let Some(d) = self.indexes.get(key) {
            let a = *d;
            return Some(a);
        }
        None
    }

    pub fn get_uri(&self, index: usize) -> Option<String> {
        self.indexes.iter().find_map(|item| {
            if item.value() == &index {
                Some(String::from(item.key()))
            } else {
                None
            }
        })
    }

    pub fn on_change(&self, params: ServerTextDocumentItem) -> Option<()> {
        let file = self.get_index(&params.uri.to_string())?;
        self.add_tree(file, None, &params.text, None);
        None
    }

    pub fn input_edit(&self, file: &String, code: String, input_edit: InputEdit) -> Option<()> {
        let file = self.get_index(file)?;
        let mut old_tree = self.get_mut_tree(file)?;
        let _ = self.parsers.lock().is_ok_and(|mut parsers| {
            old_tree.0.edit(&input_edit);
            let tree = parsers.parse(old_tree.1, &code, Some(&old_tree.0));
            if let Some(tree) = tree {
                let lang = old_tree.1;
                drop(old_tree);
                self.trees.insert(file, (tree, lang));
            }
            true
        });
        //
        None
    }

    pub fn publish_tag_diagnostics(
        &self,
        diagnostics: Vec<Tag>,
        hm: &mut HashMap<String, Vec<Diagnostic>>,
    ) {
        for diag in diagnostics {
            if let Some(uri) = self.get_uri(diag.file) {
                let diagnostic = Diagnostic {
                    range: Range::new(
                        Position::new(diag.line as u32, diag.start as u32),
                        Position::new(diag.line as u32, diag.end as u32),
                    ),
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

    pub fn goto_definition(
        &self,
        params: GotoDefinitionParams,
        config: &RwLock<Option<HtmxConfig>>,
        document_map: &DashMap<String, Rope>,
        query: &HTMLQueries2,
        // res: tower_lsp::jsonrpc::Result<Option<GotoDefinitionResponse>>,
    ) -> Option<PositionType> {
        let response = None;
        let file = params
            .text_document_position_params
            .text_document
            .uri
            .to_string();
        let ext = config.read().is_ok_and(|config| {
            if let Some(config) = config.as_ref() {
                let ext = config.file_ext(Path::new(&file));
                return ext.is_some_and(|lang_type| lang_type == LangType::Template);
            }
            false
        });
        if !ext {
            return None;
        }
        let text = {
            let c = document_map.get(&file)?;
            let mut w = LocalWriter::default();
            let _ = c.value().write_to(&mut w);
            w
        };
        let index = self.get_index(&file);
        if let Some(index) = index {
            if let Some(tree) = self.get_tree(index) {
                let root_node = tree.0.root_node();
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
            // res = Ok(self.check_definition(position));
        }
        response
        // drop(lsp_files);
    }

    pub fn goto_definition_response(
        &self,
        definition: Option<PositionDefinition>,
        value: &str,
        def: &mut Option<GotoDefinitionResponse>,
    ) -> Option<()> {
        let tag = in_tags(value, definition?)?;
        let tag = self.get_tag(&tag.name)?;
        let file = self.get_uri(tag.file)?;
        let start = Position::new(tag.line as u32, tag.start as u32);
        let end = Position::new(tag.line as u32, tag.end as u32);
        let range = Range::new(start, end);
        *def = Some(GotoDefinitionResponse::Scalar(Location {
            uri: Url::parse(&file).unwrap(),
            range,
        }));
        None
    }

    /// LangType is None when it comes from editor.
    pub fn add_tree(
        &self,
        index: usize,
        lang_type: Option<LangType>,
        text: &str,
        _range: Option<Range>,
    ) {
        let _ = self.parsers.lock().is_ok_and(|mut parsers| {
            if let Some(old_tree) = self.trees.get_mut(&index) {
                if let Some(tree) = parsers.parse(old_tree.1, text, Some(&old_tree.0)) {
                    let lang = old_tree.1;
                    drop(old_tree);
                    self.trees.insert(index, (tree, lang));
                }
            } else if let Some(lang_type) = lang_type {
                // tree doesn't exist, first insertion
                if let Some(tree) = parsers.parse(lang_type, text, None) {
                    self.trees.insert(index, (tree, lang_type));
                }
            }
            true
        });
    }

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
        if let Some(old_tree) = self.trees.get(&index) {
            let tags = query_tag(
                old_tree.0.root_node(),
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

    pub fn saved(
        &self,
        uri: &String,
        diagnostics: &mut Vec<Tag>,
        config: &RwLock<Option<HtmxConfig>>,
        document_map: &DashMap<String, Rope>,
        queries: &Arc<Mutex<Queries>>,
    ) -> Option<Vec<Tag>> {
        let path = Path::new(&uri);
        let file = self.get_index(uri)?;
        if let Ok(config) = config.read() {
            let config = config.as_ref()?;
            let lang_type = config.file_ext(path)?;
            if lang_type == LangType::Template {
                return None;
            }
            let content = document_map.get(uri)?;
            let content = content.value();
            let mut a = LocalWriter::default();
            let _ = content.write_to(&mut a);
            let content = a.content;
            let _ = queries.lock().is_ok_and(|queries| {
                let _ = self.add_tags_from_file(
                    file,
                    lang_type,
                    &content,
                    false,
                    &queries,
                    diagnostics,
                );
                true
            });
            return Some(diagnostics.to_vec());
            //
        }
        None
    }

    pub fn references(
        &self,
        params: ReferenceParams,
        queries: &Queries,
        document_map: &DashMap<String, Rope>,
    ) -> Option<Vec<Location>> {
        let mut locations = None;
        let uri = String::from(&params.text_document_position.text_document.uri.to_string());
        let point = Point::new(
            params.text_document_position.position.line as usize,
            params.text_document_position.position.character as usize,
        );
        let index = self.get_index(&uri)?;
        let tree = self.get_tree(index)?;
        if let Ok(c) = HtmxQuery::try_from(tree.1) {
            let query = queries.get(c);
            let content = document_map.get(&uri)?;
            let mut w = LocalWriter::default();
            let _ = content.value().write_to(&mut w);
            drop(content);
            let tags = query_tag(
                tree.0.root_node(),
                &w.content,
                point,
                &QueryType::Completion,
                query,
                false,
            );
            let tag = tags.first()?;
            let mut references = vec![];
            for tree in self.trees.iter() {
                if tree.1 == LangType::Template {
                    let file = self.get_uri(*tree.key())?;
                    let mut w = LocalWriter::default();
                    let content = document_map.get(&file)?;
                    let _ = content.value().write_to(&mut w);
                    query_htmx_lsp(
                        tree.0.root_node(),
                        &w.content,
                        Point::new(0, 0),
                        &QueryType::Hover,
                        queries.html.get(HTMLQuery::Lsp),
                        &tag.name,
                        &mut references,
                        *tree.key(),
                    );
                }
            }
            let mut response = vec![];
            for i in &references {
                let index = self.get_uri(i.file)?;
                let start = Position::new(i.line as u32, i.start as u32);
                let end = Position::new(i.line as u32, i.end as u32 + 1);
                let range = Range::new(start, end);
                let location = Location::new(Url::parse(&index).unwrap(), range);
                response.push(location);
            }
            locations = Some(response);
        }
        locations
    }

    pub fn goto_implementation(
        &self,
        params: GotoImplementationParams,
        queries: &Queries,
        document_map: &DashMap<String, Rope>,
    ) -> Option<GotoImplementationResponse> {
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
        let tree = self.get_tree(index)?;
        if tree.1 != LangType::Template {
            return None;
        }
        let query = queries.html.get(HTMLQuery::Lsp);
        let content = document_map.get(&uri)?;
        let mut w = LocalWriter::default();
        let _ = content.value().write_to(&mut w);
        drop(content);
        let uri = params
            .text_document_position_params
            .text_document
            .uri
            .clone();
        let capture = find_hx_lsp(tree.0.root_node(), w.content, point, query)?;
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

    pub fn query_position(
        &self,
        index: usize,
        text: &str,
        query_type: QueryType,
        pos: Position,
        query: &HTMLQueries2,
    ) -> Option<PositionType> {
        let tree = self.get_tree(index)?;
        let root_node = tree.0.root_node();
        let trigger_point = Point::new(pos.line as usize, pos.character as usize);

        query_position(root_node, text, trigger_point, query_type, query)
    }

    pub fn get_tree(&self, index: usize) -> Option<Ref<'_, usize, (Tree, LangType)>> {
        self.trees.get(&index)
    }

    pub fn get_mut_tree(&self, index: usize) -> Option<RefMut<'_, usize, (Tree, LangType)>> {
        self.trees.get_mut(&index)
    }
}

pub struct Parsers {
    html: Parser,
    javascript: Parser,
    backend: Parser,
}

impl Parsers {
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

// pub struct HTMLQueries {
//     lsp: Query,
//     name: Query,
//     value: Query,
// }

// impl Default for HTMLQueries {
//     fn default() -> Self {
//         let lsp = Query::new(tree_sitter_html::language(), HX_HTML).unwrap();
//         let name = Query::new(tree_sitter_html::language(), HX_NAME).unwrap();
//         let value = Query::new(tree_sitter_html::language(), HX_VALUE).unwrap();
//         Self { lsp, name, value }
//     }
// }

// pub enum HTMLQuery {
//     Lsp,
//     Name,
//     Value,
// }
