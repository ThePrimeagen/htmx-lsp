use lazy_static::lazy_static;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use lsp_types::Url;

type TextStore = HashMap<String, String>;

lazy_static! {
    pub static ref TEXT_STORE: Arc<Mutex<TextStore>> = Arc::new(Mutex::new(HashMap::new()));
}

pub fn get_text_document(uri: Url) -> Option<String> {
    TEXT_STORE
        .lock()
        .expect("text store mutex poisoned")
        .get(uri.as_str())
        .cloned()
}
