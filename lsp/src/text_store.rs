use std::{
    collections::HashMap,
    sync::{Arc, Mutex, OnceLock},
};

use lsp_types::Url;

pub struct TextStore {
    pub texts: HashMap<String, String>,
}

pub static TEXT_STORE: OnceLock<Arc<Mutex<TextStore>>> = OnceLock::new();
pub fn init_text_store() {
    _ = TEXT_STORE.set(Arc::new(Mutex::new(TextStore {
        texts: HashMap::new(),
    })));
}

pub fn get_text_document(uri: Url) -> Option<String> {
    return TEXT_STORE
        .get()
        .expect("text store not initialized")
        .lock()
        .expect("text store mutex poisoned")
        .texts
        .get(&uri.to_string())
        .cloned();
}
