use std::{collections::HashMap, sync::{OnceLock, Arc, Mutex}};

pub struct TextStore {
    pub texts: HashMap<String, String>,
}

pub static TEXT_STORE: OnceLock<Arc<Mutex<TextStore>>> = OnceLock::new();
pub fn init_text_store() {
    _ = TEXT_STORE.set(Arc::new(Mutex::new(TextStore {
        texts: HashMap::new(),
    })));
}
