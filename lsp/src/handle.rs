use crate::{
    htmx::{hx_completion, hx_hover, HxCompletionValue},
    text_store::TEXT_STORE,
};
use log::{debug, error, warn};
use lsp_server::{Message, Notification, Request, RequestId};
use lsp_types::{CompletionContext, CompletionParams, CompletionTriggerKind, HoverParams};

#[derive(serde::Deserialize, Debug)]
struct Text {
    text: String,
}

#[derive(serde::Deserialize, Debug)]
struct TextDocumentLocation {
    uri: String,
}

#[derive(serde::Deserialize, Debug)]
struct TextDocumentChanges {
    #[serde(rename = "textDocument")]
    text_document: TextDocumentLocation,

    #[serde(rename = "contentChanges")]
    content_changes: Vec<Text>,
}

#[derive(serde::Deserialize, Debug)]
struct TextDocumentOpened {
    uri: String,

    text: String,
}

#[derive(serde::Deserialize, Debug)]
struct TextDocumentOpen {
    #[serde(rename = "textDocument")]
    text_document: TextDocumentOpened,
}

#[derive(Debug)]
pub struct HtmxAttributeCompletion {
    pub items: HxCompletionValue,
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
    let text_document_changes: TextDocumentChanges = serde_json::from_value(noti.params).ok()?;
    let uri = text_document_changes.text_document.uri;
    let text = text_document_changes.content_changes[0].text.to_string();

    if text_document_changes.content_changes.len() > 1 {
        error!("more than one content change, please be wary");
    }

    TEXT_STORE
        .get()
        .expect("text store not initialized")
        .lock()
        .expect("text store mutex poisoned")
        .insert(uri, text);

    None
}

#[allow(non_snake_case)]
fn handle_didOpen(noti: Notification) -> Option<HtmxResult> {
    debug!("handle_didOpen params {:?}", noti.params);
    let text_document_changes = match serde_json::from_value::<TextDocumentOpen>(noti.params) {
        Ok(p) => p.text_document,
        Err(err) => {
            error!("handle_didOpen parsing params error : {:?}", err);
            return None;
        }
    };

    TEXT_STORE
        .get()
        .expect("text store not initialized")
        .lock()
        .expect("text store mutex poisoned")
        .insert(text_document_changes.uri, text_document_changes.text);

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
                Some(items) => items,
                None => {
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
    match noti.method.as_str() {
        "textDocument/didChange" => handle_didChange(noti),
        "textDocument/didOpen" => handle_didOpen(noti),
        s => {
            debug!("unhandled notification: {:?}", s);
            None
        }
    }
}

pub fn handle_other(msg: Message) -> Option<HtmxResult> {
    warn!("unhandled message {:?}", msg);
    None
}

#[cfg(test)]
mod tests {
    use super::{handle_request, HtmxResult, Request};
    use crate::text_store::{init_text_store, TEXT_STORE};
    use std::sync::Once;

    static SETUP: Once = Once::new();
    fn prepare_store(file: &str, content: &str) {
        SETUP.call_once(|| {
            init_text_store();
        });

        TEXT_STORE
            .get()
            .expect("text store not initialized")
            .lock()
            .expect("text store mutex poisoned")
            .insert(file.to_string(), content.to_string());
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
                panic!("unexpected result: {result:?}");
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
                panic!("unexpected result: {result:?}");
            }
        }
    }
}
