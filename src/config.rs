use dashmap::DashMap;
use ropey::Rope;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs::read_to_string,
    path::Path,
    sync::{Arc, Mutex, MutexGuard, RwLock},
};

use crate::{htmx_tags::Tag, htmx_tree_sitter::LspFiles, init_hx::LangType, query_helper::Queries};

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct HtmxConfig {
    pub lang: String,
    pub template_ext: String,
    pub templates: Vec<String>,
    pub js_tags: Vec<String>,
    pub backend_tags: Vec<String>,
    #[serde(skip)]
    pub is_valid: bool,
}

impl HtmxConfig {
    pub fn is_backend(&self, ext: &str) -> bool {
        match self.lang.as_str() {
            "rust" => ext == "rs",
            "python" => ext == "py",
            "go" => ext == "go",
            _ => false,
        }
    }

    pub fn file_ext(&self, path: &Path) -> Option<LangType> {
        match path.extension()?.to_str() {
            Some(e) => match e {
                "js" | "ts" => Some(LangType::JavaScript),
                backend if self.is_backend(backend) => Some(LangType::Backend),
                template if template == self.template_ext => Some(LangType::Template),
                _ => None,
            },
            None => None,
        }
    }

    pub fn is_supported_backend(&self) -> bool {
        matches!(self.lang.as_str(), "python" | "rust" | "go")
    }
}

pub fn validate_config(config: Option<Value>) -> Option<HtmxConfig> {
    if let Some(config) = config {
        if let Ok(mut config) = serde_json::from_value::<HtmxConfig>(config) {
            config.is_valid = true;
            return Some(config);
        }
    }
    None
}

pub fn read_config(
    config: &RwLock<HtmxConfig>,
    lsp_files: &Arc<Mutex<LspFiles>>,
    queries: &Arc<Mutex<Queries>>,
    document_map: &DashMap<String, Rope>,
) -> anyhow::Result<Vec<Tag>> {
    if let Ok(config) = config.read() {
        if config.template_ext.is_empty() || config.template_ext.contains(' ') {
            return Err(anyhow::Error::msg("Template extension not found."));
        } else if !config.is_supported_backend() {
            return Err(anyhow::Error::msg(format!(
                "Language {} is not supported.",
                config.lang
            )));
        }
        walkdir(&config, lsp_files, queries, document_map)
    } else {
        Err(anyhow::Error::msg("Config is not found"))
    }
}

fn walkdir(
    config: &HtmxConfig,
    lsp_files: &Arc<Mutex<LspFiles>>,
    queries: &Arc<Mutex<Queries>>,
    document_map: &DashMap<String, Rope>,
) -> anyhow::Result<Vec<Tag>> {
    let lsp_files = lsp_files.lock().unwrap();
    let mut diagnostics = vec![];
    lsp_files.reset();
    let directories = [&config.templates, &config.js_tags, &config.backend_tags];
    queries
        .lock()
        .ok()
        .and_then(|mut queries| queries.change_backend(&config.lang));
    for (index, dir) in directories.iter().enumerate() {
        let lang_type = LangType::from(index);
        lsp_files
            .parsers
            .lock()
            .ok()
            .and_then(|mut parsers| parsers.change_backend(&config.lang, lang_type));
        for file in dir.iter() {
            for entry in walkdir::WalkDir::new(file) {
                let entry = entry?;
                let metadata = entry.metadata()?;
                if metadata.is_file() {
                    let path = &entry.path();
                    let ext = config.file_ext(path);
                    if !ext.is_some_and(|ext| ext == lang_type) {
                        continue;
                    }
                    if queries
                        .lock()
                        .ok()
                        .and_then(|queries| {
                            add_file(
                                path,
                                &lsp_files,
                                lang_type,
                                &queries,
                                &mut diagnostics,
                                false,
                                document_map,
                            )
                        })
                        .is_none()
                    {
                        return Err(anyhow::Error::msg(format!(
                            "Template path: {} does not exist",
                            file
                        )));
                    };
                }
            }
        }
    }
    Ok(diagnostics)
}

fn add_file(
    path: &&Path,
    lsp_files: &MutexGuard<LspFiles>,
    lang_type: LangType,
    queries: &Queries,
    diags: &mut Vec<Tag>,
    _skip: bool,
    document_map: &DashMap<String, Rope>,
) -> Option<bool> {
    if let Ok(name) = std::fs::canonicalize(path) {
        let name = name.to_str()?;
        let file = lsp_files.add_file(format!("file://{}", name))?;
        return read_to_string(name).ok().map(|content| {
            let rope = ropey::Rope::from_str(&content);
            document_map.insert(format!("file://{}", name).to_string(), rope);
            lsp_files.add_tree(file, Some(lang_type), &content, None);
            let _ = lsp_files.add_tags_from_file(file, lang_type, &content, false, queries, diags);
            true
        });
    }
    None
}
