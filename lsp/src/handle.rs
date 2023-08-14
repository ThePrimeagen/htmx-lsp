use log::{debug, warn};
use lsp_server::Notification;

#[derive(serde::Deserialize, Debug)]
struct Text {
    text: String,
}

#[derive(serde::Deserialize, Debug)]
struct TextDocumentLocation {
    uri: String,
}

#[derive(serde::Deserialize, Debug)]
struct TextDocument {
    #[serde(rename = "textDocument")]
    text_document: TextDocumentLocation,
}

#[derive(serde::Deserialize, Debug)]
struct TextDocumentChanges {
    #[serde(rename = "textDocument")]
    text_document: TextDocumentLocation,

    #[serde(rename = "contentChanges")]
    content_changes: Vec<Text>,
}

/*
fn perf_msg_to_diagnostic(perf: &PerfMessage, source: &str) -> Diagnostic {
    return match perf {
        PerfMessage::Diagnostic(d) => {
            Diagnostic::new(
                Range {
                    start: byte_pos_to_line_col(&source, d.position.0),
                    end: byte_pos_to_line_col(&source, d.position.1),
                },
                Some(DiagnosticSeverity::HINT),
                None,
                None,
                d.msg.clone(),
                None,
                None)
        }
    }
}
*/

// ignore snakeCase
#[allow(non_snake_case)]
fn handle_didChange(notification: Notification) -> Option<()> {
    debug!("handling didChange {:?}", notification);
    return None;
}

pub fn handle_notification(not: Notification) -> Option<()> {
    debug!("got notification: {:?}", not);

    match not.method.as_str() {
        "textDocument/didChange" => {
            return handle_didChange(not);
        }
        "textDocument/didSave" => {
            warn!("textDocument/didSave was called");
        }
        s => {
            debug!("unhandled notification: {:?}", s);
        }
    };

    return None;
}
