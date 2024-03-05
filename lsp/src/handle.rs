use crate::{
    htmx::{hx_completion, hx_hover, HxCompletion},
    text_store::{DocInfo, DOCUMENT_STORE},
    tree_sitter::text_doc_change_to_ts_edit,
};
use log::{debug, error, warn};
use lsp_server::{Message, Notification, Request, RequestId};
use lsp_textdocument::FullTextDocument;
use lsp_types::{
    notification::{DidChangeTextDocument, DidOpenTextDocument},
    CompletionContext, CompletionParams, CompletionTriggerKind, HoverParams,
};

#[derive(Debug)]
pub struct HtmxAttributeCompletion {
    pub items: Vec<HxCompletion>,
    pub id: RequestId,
}

#[derive(Debug)]
pub struct HtmxAttributeHoverResult {
    pub id: RequestId,
    pub value: String,
}

#[derive(Debug)]
pub enum HtmxResult {
    // Diagnostic,
    AttributeCompletion(HtmxAttributeCompletion),

    AttributeHover(HtmxAttributeHoverResult),
}

// ignore snakeCase
#[allow(non_snake_case)]
fn handle_didChange(noti: Notification) -> Option<HtmxResult> {
    match cast_notif::<DidChangeTextDocument>(noti) {
        Ok(params) => {
            match DOCUMENT_STORE
                .get()
                .expect("text store not initialized")
                .lock()
                .expect("text store mutex poisoned")
                .get_mut(params.text_document.uri.as_str())
            {
                Some(entry) => {
                    entry
                        .doc
                        .update(&params.content_changes, params.text_document.version);

                    if let Some(ref mut curr_tree) = entry.tree {
                        for edit in params.content_changes.iter() {
                            match text_doc_change_to_ts_edit(edit, &entry.doc) {
                                Ok(edit) => {
                                    curr_tree.edit(&edit);
                                }
                                Err(e) => {
                                    error!("handle_didChange Bad edit info, failed to edit tree -- Error: {e}");
                                }
                            }
                        }
                    } else {
                        error!(
                            "handle_didChange tree for {} is None",
                            params.text_document.uri.as_str()
                        );
                    }
                }
                None => {
                    error!(
                        "handle_didChange No corresponding doc for supplied edits -- {}",
                        params.text_document.uri.as_str()
                    );
                }
            }
        }
        Err(e) => {
            error!("Failed the deserialize DidChangeTextDocument params -- Error {e}");
        }
    }

    None
}

#[allow(non_snake_case)]
fn handle_didOpen(noti: Notification) -> Option<HtmxResult> {
    debug!("handle_didOpen params {:?}", noti.params);
    let text_doc_open = match cast_notif::<DidOpenTextDocument>(noti) {
        Ok(params) => params,
        Err(err) => {
            error!("handle_didOpen parsing params error : {:?}", err);
            return None;
        }
    };

    let doc = FullTextDocument::new(
        text_doc_open.text_document.language_id,
        text_doc_open.text_document.version,
        text_doc_open.text_document.text,
    );
    let mut parser = ::tree_sitter::Parser::new();
    parser
        .set_language(tree_sitter_html::language())
        .expect("Failed to load HTML grammar");
    let tree = parser.parse(doc.get_content(None), None);

    let doc = DocInfo { doc, parser, tree };

    DOCUMENT_STORE
        .get()
        .expect("text store not initialized")
        .lock()
        .expect("text store mutex poisoned")
        .insert(text_doc_open.text_document.uri.to_string(), doc);

    None
}

#[allow(non_snake_case)]
fn handle_completion(req: Request) -> Option<HtmxResult> {
    let completion: CompletionParams = serde_json::from_value(req.params).ok()?;

    error!("handle_completion: {:?}", completion);

    match completion.context {
        Some(CompletionContext {
            trigger_kind: CompletionTriggerKind::TRIGGER_CHARACTER,
            ..
        })
        | Some(CompletionContext {
            trigger_kind: CompletionTriggerKind::INVOKED,
            ..
        }) => {
            let items = match hx_completion(completion.text_document_position) {
                (Some(items), Some(ext_items)) => {
                    let mut temp = items.to_vec();
                    for ext_item in ext_items {
                        temp.append(&mut ext_item.to_vec());
                    }
                    temp
                }
                (Some(items), None) => items.to_vec(),
                (None, Some(ext_items)) => {
                    let mut temp = Vec::new();
                    for ext_item in ext_items {
                        temp.append(&mut ext_item.to_vec());
                    }
                    temp
                }
                (None, None) => {
                    error!("EMPTY RESULTS OF COMPLETION");
                    return None;
                }
            };

            error!(
                "handled result: {:?}: completion result: {:?}",
                completion.context, items
            );

            Some(HtmxResult::AttributeCompletion(HtmxAttributeCompletion {
                items,
                id: req.id,
            }))
        }
        _ => {
            error!("unhandled completion context: {:?}", completion.context);
            None
        }
    }
}

fn handle_hover(req: Request) -> Option<HtmxResult> {
    let hover: HoverParams = serde_json::from_value(req.params).ok()?;
    debug!("handle_hover: {:?}", hover);

    let text_params = hover.text_document_position_params;

    debug!("handle_hover text_position_params: {:?}", text_params);

    let attribute = hx_hover(text_params)?;

    debug!("handle_request attribute: {:?}", attribute);

    Some(HtmxResult::AttributeHover(HtmxAttributeHoverResult {
        id: req.id,
        value: attribute.desc.to_string(),
    }))
}

pub fn handle_request(req: Request) -> Option<HtmxResult> {
    error!("handle_request");
    match req.method.as_str() {
        "textDocument/completion" => handle_completion(req),
        "textDocument/hover" => handle_hover(req),
        _ => {
            warn!("unhandled request: {:?}", req);
            None
        }
    }
}

pub fn handle_notification(noti: Notification) -> Option<HtmxResult> {
    return match noti.method.as_str() {
        "textDocument/didChange" => handle_didChange(noti),
        "textDocument/didOpen" => handle_didOpen(noti),
        s => {
            debug!("unhandled notification: {:?}", s);
            None
        }
    };
}

pub fn handle_other(msg: Message) -> Option<HtmxResult> {
    warn!("unhandled message {:?}", msg);
    None
}

fn cast_notif<R>(notif: Notification) -> anyhow::Result<R::Params>
where
    R: lsp_types::notification::Notification,
    R::Params: serde::de::DeserializeOwned,
{
    match notif.extract(R::METHOD) {
        Ok(value) => Ok(value),
        Err(e) => Err(anyhow::anyhow!(
            "cast_notif Failed to extract params -- Error: {e}"
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::{handle_request, HtmxResult, Request};
    use crate::text_store::{init_text_store, DocInfo, DOCUMENT_STORE};
    use std::sync::Once;

    static SETUP: Once = Once::new();
    fn prepare_store(file: &str, content: &str) {
        SETUP.call_once(|| {
            init_text_store();
        });

        let doc =
            lsp_textdocument::FullTextDocument::new("html".to_string(), 0, content.to_string());
        let mut parser = ::tree_sitter::Parser::new();
        parser
            .set_language(tree_sitter_html::language())
            .expect("Failed to load HTML grammar");
        let tree = parser.parse(doc.get_content(None), None);
        let doc_info = DocInfo { doc, parser, tree };

        DOCUMENT_STORE
            .get()
            .expect("text store not initialized")
            .lock()
            .expect("text store mutex poisoned")
            .insert(file.to_string(), doc_info);
    }

    #[test]
    fn handle_hover_it_presents_details_when_tag_value_is_under_cursor() {
        let file = "file:///detailstag.html";
        let content = r#"<div hx-target="next"></div>"#;

        prepare_store(file, content);

        let req = Request {
            id: 1.into(),
            method: "textDocument/hover".to_string(),
            params: serde_json::json!({
                "textDocument": {
                    "uri": file,
                },
                "position": {
                    "line": 0,
                    "character": 13
                }
            }),
        };

        let result = handle_request(req);

        assert!(result.is_some());
        match result {
            Some(HtmxResult::AttributeHover(h)) => {
                assert_eq!(h.id, 1.into());
                assert!(h.value.starts_with("hx-target"));
            }
            _ => {
                panic!("unexpected result: {:?}", result);
            }
        }
    }

    #[test]
    fn handle_hover_it_presents_details_of_the_tag_name_when_is_under_cursor() {
        let file = "file:///detailstag.html";
        let content = r#"<div hx-target="next"></div>"#;

        prepare_store(file, content);

        let req = Request {
            id: 1.into(),
            method: "textDocument/hover".to_string(),
            params: serde_json::json!({
                "textDocument": {
                    "uri": file,
                },
                "position": {
                    "line": 0,
                    "character": 14
                }
            }),
        };

        let result = handle_request(req);

        assert!(result.is_some());
        match result {
            Some(HtmxResult::AttributeHover(h)) => {
                assert_eq!(h.id, 1.into());
                assert!(h.value.starts_with("hx-target"));
            }
            _ => {
                panic!("unexpected result: {:?}", result);
            }
        }
    }
}
