use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
    sync::{Arc, Mutex, OnceLock},
};

use lsp_types::Url;

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

pub fn get_text_document(uri: Url) -> Option<String> {
    TEXT_STORE
        .get()
        .expect("text store not initialized")
        .lock()
        .expect("text store mutex poisoned")
        .get(&uri.to_string())
        .cloned()
}
