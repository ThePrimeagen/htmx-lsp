use dashmap::DashMap;
use ropey::Rope;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::{
    fs::read_to_string,
    io::Error,
    path::Path,
    sync::{Arc, Mutex, MutexGuard, RwLock},
};

use crate::{htmx_tags::Tag, htmx_tree_sitter::LspFiles, init_hx::LangType, query_helper::Queries};
use thiserror::Error;

#[derive(Serialize, Deserialize, Debug)]
pub struct HtmxConfig {
    pub lang: String,
    pub template_ext: String,
    pub templates: Vec<String>,
    pub js_tags: Vec<String>,
    pub backend_tags: Vec<String>,
}

impl HtmxConfig {
    pub fn is_backend(&self, ext: &str) -> bool {
        match self.lang.as_str() {
            "rust" => ext == "rs",
            "python" => ext == "py",
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
}

pub fn validate_config(config: Option<Value>) -> Option<HtmxConfig> {
    if let Some(config) = config {
        if let Ok(config) = serde_json::from_value::<HtmxConfig>(config) {
            return Some(config);
        }
    }
    None
}

pub fn read_config(
    config: &RwLock<Option<HtmxConfig>>,
    lsp_files: &Arc<Mutex<LspFiles>>,
    queries: &Queries,
    document_map: &DashMap<String, Rope>,
) -> Result<Vec<Tag>, ConfigError> {
    if let Ok(config) = config.read() {
        if let Some(config) = config.as_ref().filter(|_| true) {
            if config.template_ext.is_empty() || config.template_ext.contains(' ') {
                return Err(ConfigError::TemplateExtension);
            } else if config.lang != "rust" {
                return Err(ConfigError::LanguageSupport(String::from(&config.lang)));
            }
            walkdir(config, lsp_files, queries, document_map)
        } else {
            Err(ConfigError::ConfigNotFound)
        }
    } else {
        Err(ConfigError::ConfigNotFound)
    }
}

fn walkdir(
    config: &HtmxConfig,
    lsp_files: &Arc<Mutex<LspFiles>>,
    queries: &Queries,
    document_map: &DashMap<String, Rope>,
) -> Result<Vec<Tag>, ConfigError> {
    let lsp_files = lsp_files.lock().unwrap();
    let mut diagnostics = vec![];
    lsp_files.reset();
    let directories = [&config.templates, &config.js_tags, &config.backend_tags];
    for (index, dir) in directories.iter().enumerate() {
        let lang_type = LangType::from(index);
        for file in dir.iter() {
            for entry in walkdir::WalkDir::new(file) {
                if let Ok(entry) = &entry {
                    if let Ok(metadata) = &entry.metadata() {
                        if metadata.is_file() {
                            let path = &entry.path();
                            let ext = config.file_ext(path);
                            if !ext.is_some_and(|t| t == lang_type) {
                                continue;
                            }
                            add_file(
                                path,
                                &lsp_files,
                                lang_type,
                                queries,
                                &mut diagnostics,
                                false,
                                document_map,
                            );
                        }
                    }
                } else {
                    return Err(ConfigError::TemplatePath(String::from(file)));
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
) -> Option<()> {
    if let Ok(name) = std::fs::canonicalize(path) {
        let name = name.to_str()?;
        let file = lsp_files.add_file(format!("file://{}", name))?;
        let _ = read_to_string(name).is_ok_and(|content| {
            let rope = ropey::Rope::from_str(&content);
            document_map.insert(format!("file://{}", name).to_string(), rope);
            lsp_files.add_tree(file, Some(lang_type), &content, None);
            let _ = lsp_files.add_tags_from_file(file, lang_type, &content, false, queries, diags);
            true
        });
    }
    None
}

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Template path: {0} does not exist")]
    TemplatePath(String),
    #[error("Language {0} is not supported")]
    LanguageSupport(String),
    #[error("Template extension is empty")]
    TemplateExtension,
    #[error("Config is not found")]
    ConfigNotFound,
}

impl From<Error> for ConfigError {
    fn from(_value: Error) -> Self {
        todo!()
    }
}
