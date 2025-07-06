use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, OnceLock},
};

use lsp_types::{TextDocumentPositionParams, Url};

type TxtStore = HashMap<String, String>;

pub struct TextStore(TxtStore);

impl Deref for TextStore {
    type Target = TxtStore;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for TextStore {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub static TEXT_STORE: OnceLock<Arc<Mutex<TextStore>>> = OnceLock::new();

pub fn init_text_store() {
    _ = TEXT_STORE.set(Arc::new(Mutex::new(TextStore(HashMap::new()))));
}

pub fn get_text_document(uri: &Url) -> Option<String> {
    return TEXT_STORE
        .get()
        .expect("text store not initialized")
        .lock()
        .expect("text store mutex poisoned")
        .get(&uri.to_string())
        .cloned();
}

/// Find the start and end indices of a word inside the given line
/// Borrowed from RLS
fn find_word_at_pos(line: &str, col: usize) -> (usize, usize) {
    let line_ = line.to_string();
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
    let line = pos_params.position.line as usize;
    let col = pos_params.position.character as usize;

    match get_text_document(uri) {
        Some(text) => {
            let line_conts = match text.lines().nth(line) {
                Some(conts) => conts,
                None => {
                    return Err(anyhow::anyhow!(
                        "get_word_from_pos_params Failed to get word under cursor"
                    ));
                }
            };
            let (start, end) = find_word_at_pos(line_conts, col);
            Ok(String::from(&line_conts[start..end]))
        }
        None => Err(anyhow::anyhow!(
            "get_word_from_pos_params Failed to get word under cursor"
        )),
    }
}
