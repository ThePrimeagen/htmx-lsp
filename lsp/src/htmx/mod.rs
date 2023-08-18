mod tokenizer;

use std::{sync::OnceLock, path::PathBuf};
use lsp_types::{CompletionContext, CompletionParams, TextDocumentPositionParams};
use serde::{Serialize, Deserialize};
use util::get_text_byte_offset;

use crate::text_store::{TEXT_STORE, get_text_document};

use self::tokenizer::Tokenizer;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HxAttribute {
    pub name: String,
    pub desc: String,
}

impl From<&(&str, &str)> for HxAttribute {
    fn from((name, desc): &(&str, &str)) -> Self {
        Self {
            name: name.to_string(),
            desc: desc.to_string(),
        }
    }
}

impl TryFrom<&(PathBuf, String)> for HxAttribute {
    type Error = anyhow::Error;

    fn try_from((path, desc): &(PathBuf, String)) -> Result<Self, Self::Error> {
        let name = path.to_str().unwrap_or("").to_string();
        if name == "" {
            return Err(anyhow::anyhow!("Invalid path"));
        }
        return Ok(Self {
            name,
            desc: desc.to_string(),
        });
    }
}


pub enum HxTypeCompletion {
}

pub fn parse_hx_type(text_params: TextDocumentPositionParams) -> Option<HxTypeCompletion> {
    let text = get_text_document(text_params.text_document.uri)?;
    let pos = text_params.position;
    let end = get_text_byte_offset(&text, pos.line as usize, pos.character as usize)?;

    let tokenizer = Tokenizer::new(&text[..end]);

    return None;
}

pub fn completion(param: CompletionParams) {
}

pub static HX_TAGS: OnceLock<Vec<HxAttribute>> = OnceLock::new();

pub fn init_hx_tags() {
    _ = HX_TAGS.set(
        vec![
            ("hx-boost", include_str!("./attributes/hx-boost.md")),
            ("hx-delete", include_str!("./attributes/hx-delete.md")),
            ("hx-get", include_str!("./attributes/hx-get.md")),
            ("hx-include", include_str!("./attributes/hx-include.md")),
            ("hx-patch", include_str!("./attributes/hx-patch.md")),
            ("hx-post", include_str!("./attributes/hx-post.md")),
            ("hx-put", include_str!("./attributes/hx-put.md")),
            ("hx-swap", include_str!("./attributes/hx-swap.md")),
            ("hx-target", include_str!("./attributes/hx-target.md")),
            ("hx-trigger", include_str!("./attributes/hx-trigger.md")),
            ("hx-vals", include_str!("./attributes/hx-vals.md")),
            ("hx-push-url", include_str!("./attributes/hx-push-url.md")),
            ("hx-select", include_str!("./attributes/hx-select.md")),
        ]
            .iter()
            .filter_map(|x| x.try_into().ok())
            .collect()
    );

}



