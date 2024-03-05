use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, OnceLock},
};

use lsp_textdocument::FullTextDocument;
use lsp_types::{Position, Range, TextDocumentPositionParams, Url};
use tree_sitter::{Parser, Tree};

pub struct DocInfo {
    pub doc: FullTextDocument,
    pub parser: Parser,
    pub tree: Option<Tree>,
}

type DocStore = HashMap<String, DocInfo>;

#[derive(Default)]
pub struct DocumentStore(DocStore);

impl Deref for DocumentStore {
    type Target = DocStore;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for DocumentStore {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub static DOCUMENT_STORE: OnceLock<Arc<Mutex<DocumentStore>>> = OnceLock::new();

pub fn init_text_store() {
    _ = DOCUMENT_STORE.set(Arc::new(Mutex::new(DocumentStore::default())));
}

pub fn get_text_document(uri: &Url, range: Option<Range>) -> Option<String> {
    return DOCUMENT_STORE
        .get()
        .expect("text store not initialized")
        .lock()
        .expect("text store mutex poisoned")
        .get(&uri.to_string())
        .map(|doc| doc.doc.get_content(range).to_string());
}

/// Find the start and end indices of a word inside the given line
/// Borrowed from RLS
fn find_word_at_pos(line: &str, col: usize) -> (usize, usize) {
    let line_ = format!("{} ", line);
    let is_ident_char = |c: char| c.is_alphanumeric() || c == '_' || c == '-';

    let start = line_
        .chars()
        .enumerate()
        .take(col)
        .filter(|&(_, c)| !is_ident_char(c))
        .last()
        .map(|(i, _)| i + 1)
        .unwrap_or(0);

    #[allow(clippy::filter_next)]
    let mut end = line_
        .chars()
        .enumerate()
        .skip(col)
        .filter(|&(_, c)| !is_ident_char(c));

    let end = end.next();
    (start, end.map(|(i, _)| i).unwrap_or(col))
}

pub fn get_word_from_pos_params(pos_params: &TextDocumentPositionParams) -> anyhow::Result<String> {
    let uri = &pos_params.text_document.uri;
    let line = pos_params.position.line;
    let col = pos_params.position.character as usize;

    let range = Range {
        start: Position { line, character: 0 },
        end: Position {
            line,
            character: u32::MAX,
        },
    };

    match get_text_document(uri, Some(range)) {
        Some(line_conts) => {
            let (start, end) = find_word_at_pos(&line_conts, col);
            Ok(String::from(&line_conts[start..end]))
        }
        None => Err(anyhow::anyhow!(
            "get_word_from_pos_params Failed to get word under cursor"
        )),
    }
}
