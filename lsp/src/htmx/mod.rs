use log::error;
use lsp_types::TextDocumentPositionParams;
use serde::{Deserialize, Serialize};
use std::{path::PathBuf, sync::OnceLock, collections::HashMap};
use util::get_text_byte_offset;

use crate::tree_sitter::Position;

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct HxCompletion {
    pub name: String,
    pub desc: String,
}

impl From<&(&str, &str)> for HxCompletion {
    fn from((name, desc): &(&str, &str)) -> Self {
        Self {
            name: name.to_string(),
            desc: desc.to_string(),
        }
    }
}

impl TryFrom<&(PathBuf, String)> for HxCompletion {
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

pub fn hx_completion(text_params: TextDocumentPositionParams) -> Option<Vec<HxCompletion>> {

    let result = crate::tree_sitter::get_position_from_lsp_completion(text_params.clone())?;

    error!("result: {:?} params: {:?}", result, text_params);

    match result {
        Position::AttributeName(name) => {
            if name.starts_with("hx-") {
                return HX_TAGS.get().cloned();
            }
        },

        Position::AttributeValue { name, .. } => {
            let values = HX_ATTRIBUTE_VALUES.get()?.get(&name)?;
            return Some(values.clone());
        },
    };

    return None;
}

pub static HX_TAGS: OnceLock<Vec<HxCompletion>> = OnceLock::new();
pub static HX_ATTRIBUTE_VALUES: OnceLock<HashMap<String, Vec<HxCompletion>>> = OnceLock::new();

fn to_hx_completion(values: Vec<(&str, &str)>) -> Vec<HxCompletion> {
    return values
        .iter()
        .filter_map(|x| x.try_into().ok())
        .collect();
}

pub fn init_hx_tags() {
    _ = HX_ATTRIBUTE_VALUES.set(
        maplit::hashmap!{
            String::from("hx-boost") => to_hx_completion(vec![
                ("true", ""),
                ("false", ""),
            ]),

            String::from("hx-swap") => to_hx_completion(vec![
                ("innerHTML", include_str!("./hx-swap/innerHTML.md")),
                ("outerHTML", include_str!("./hx-swap/outerHTML.md")),
                ("afterbegin", include_str!("./hx-swap/afterbegin.md")),
                ("afterend", include_str!("./hx-swap/afterend.md")),
                ("beforebegin", include_str!("./hx-swap/beforebegin.md")),
                ("beforeend", include_str!("./hx-swap/beforeend.md")),
                ("delete", include_str!("./hx-swap/delete.md")),
                ("none", include_str!("./hx-swap/none.md")),
            ]),

            String::from("hx-target") => to_hx_completion(vec![
                ("closest", include_str!("./hx-target/closest.md")),
                ("find", include_str!("./hx-target/find.md")),
                ("next", include_str!("./hx-target/next.md")),
                ("prev", include_str!("./hx-target/prev.md")),
                ("this", include_str!("./hx-target/this.md")),
            ]),
        });

    _ = HX_TAGS.set(
        to_hx_completion(vec![
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
    ]));
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::text_store::{get_text_document, init_text_store, TEXT_STORE};
    use lsp_types::{Position, TextDocumentIdentifier, TextDocumentPositionParams, Url};
    use std::sync::Once;

    static SETUP: Once = Once::new();
    fn prepare_store(file: &str, content: &str) -> () {
        SETUP.call_once(|| {
            init_hx_tags();
            init_text_store();
        });

        TEXT_STORE
            .get()
            .expect("text store not initialized")
            .lock()
            .expect("text store mutex poisoned")
            .texts
            .insert(file.to_string(), content.to_string());
    }

    #[test]
    fn test_it_presents_htmx_tags_after_hx_dash() {
        let file = "file:///test2.html";
        let content = r#"<div hx- "#;
        let file_url = Url::parse(file).unwrap();

        prepare_store(file, content);
        let completion = hx_completion(TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: file_url },
            position: Position {
                line: 0,
                character: 15,
            },
        });

        let completions = completion.expect("completion must not be none");
        assert_eq!(
            completions
                .into_iter()
                .map(|c| c.name)
                .collect::<Vec<String>>(),
            vec![
                "hx-boost",
                "hx-delete",
                "hx-get",
                "hx-include",
                "hx-patch",
                "hx-post",
                "hx-put",
                "hx-swap",
                "hx-target",
                "hx-trigger",
                "hx-vals",
                "hx-push-url",
                "hx-select"
            ],
        );
    }

    #[test]
    fn test_it_presents_htmx_tags_after_hx_dash_when_eval_in_error() {
        let file = "file:///test3.html";
        let content = r#"<div><div hx- </div>"#;
        let file_url = Url::parse(file).unwrap();

        prepare_store(file, content);

        let completions = hx_completion(TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: file_url },
            position: Position {
                line: 0,
                character: 13,
            },
        })
        .expect("completion must not be none");

        assert_eq!(
            completions
                .into_iter()
                .map(|c| c.name)
                .collect::<Vec<String>>(),
            vec![
                "hx-boost",
                "hx-delete",
                "hx-get",
                "hx-include",
                "hx-patch",
                "hx-post",
                "hx-put",
                "hx-swap",
                "hx-target",
                "hx-trigger",
                "hx-vals",
                "hx-push-url",
                "hx-select"
            ],
        );
    }

    #[test]
    fn test_it_presents_htmx_tags_after_hx_dash_for_normal_tags() {
        let file = "file:///normaltag.html";
        let content = r#"<div hx- </div>"#;
        let file_url = Url::parse(file).unwrap();

        prepare_store(file, content);

        let completions = hx_completion(TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: file_url },
            position: Position {
                line: 0,
                character: 8,
            },
        })
        .expect("completion must not be none");

        assert_eq!(
            completions
                .into_iter()
                .map(|c| c.name)
                .collect::<Vec<String>>(),
            vec![
                "hx-boost",
                "hx-delete",
                "hx-get",
                "hx-include",
                "hx-patch",
                "hx-post",
                "hx-put",
                "hx-swap",
                "hx-target",
                "hx-trigger",
                "hx-vals",
                "hx-push-url",
                "hx-select"
            ],
        );
    }

    #[test]
    fn test_it_presents_htmx_targets_after_first_quote() {
        let file = "file:///test1.html";
        let content = r#"<div hx-target=" "#;
        let file_url = Url::parse(file).unwrap();

        prepare_store(file, content);

        let completions = hx_completion(TextDocumentPositionParams {
            text_document: TextDocumentIdentifier { uri: file_url },
            position: Position {
                line: 0,
                character: 16,
            },
        })
        .expect("completion must not be none");

        assert_eq!(
            completions
                .into_iter()
                .map(|c| c.name)
                .collect::<Vec<String>>(),
            vec!["closest", "find", "next", "prev", "this"],
        );
    }
}
