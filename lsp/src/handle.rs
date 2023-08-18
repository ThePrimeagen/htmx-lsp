use log::{debug, warn, error};
use lsp_server::{Notification, Message, Request, RequestId};
use lsp_types::{CompletionTriggerKind, Position, request::Completion, CompletionParams, CompletionContext};

use crate::{text_store::TEXT_STORE, htmx::{HX_TAGS, HxAttribute}};

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

#[derive(Debug)]
pub struct HtmxAttributeCompletion {
    pub items: Vec<HxAttribute>,
    pub id: RequestId,
}

#[derive(Debug)]
pub enum HtmxResult {
    // Diagnostic,
    AttributeCompletion(HtmxAttributeCompletion),
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
        .texts.insert(uri, text);

    return None
}

#[allow(non_snake_case)]
fn handle_completion(req: Request) -> Option<HtmxResult> {
    let completion: CompletionParams = serde_json::from_value(req.params).ok()?;

    match completion.context {
        Some(CompletionContext {
            trigger_kind: CompletionTriggerKind::TRIGGER_CHARACTER,
            ..
        }) => {
            return None;

                /*
            // TODO: clean up clone here if perf is any issue
            return Some(HtmxResult::AttributeCompletion(HtmxAttributeCompletion {
                items: HX_TAGS.get().expect("constant data should always be present").clone(),
                id: req.id,
            }));
                */

        }
        _ => {
            return None;
        }

    };

}

pub fn handle_request(req: Request) -> Option<HtmxResult> {
    match req.method.as_str() {
        "textDocument/completion" => handle_completion(req),
        _ => {
            warn!("unhandled request: {:?}", req);
            None
        }
    }
}

pub fn handle_notification(noti: Notification) -> Option<HtmxResult> {
    return match noti.method.as_str() {
        "textDocument/didChange" => handle_didChange(noti),
        s => {
            debug!("unhandled notification: {:?}", s);
            None
        }
    };
}

pub fn handle_other(msg: Message) -> Option<HtmxResult> {
    warn!("unhandled message {:?}", msg);
    return None
}
