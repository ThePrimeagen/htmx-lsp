mod handle;
mod text_store;
mod htmx;

use anyhow::Result;
use htmx::HxAttribute;
use log::{error, info, warn};
use lsp_types::{
    CompletionItem, CompletionItemKind, CompletionList, InitializeParams, ServerCapabilities,
    TextDocumentSyncCapability, TextDocumentSyncKind, WorkDoneProgressOptions,
};

use lsp_server::{Connection, Message, Response};

use crate::{
    handle::{handle_notification, handle_other, handle_request, HtmxResult},
    text_store::init_text_store,
    htmx::init_hx_tags,
};

fn to_completion_list(items: Vec<HxAttribute>) -> CompletionList {
    return CompletionList {
        is_incomplete: false,
        items: items
            .iter()
            .map(|x| {
                return CompletionItem {
                    label: x.name.clone(),
                    label_details: None,
                    kind: Some(CompletionItemKind::TEXT),
                    detail: Some(x.desc.clone()),
                    documentation: None,
                    deprecated: Some(false),
                    preselect: None,
                    sort_text: None,
                    filter_text: None,
                    insert_text: None,
                    insert_text_format: None,
                    insert_text_mode: None,
                    text_edit: None,
                    additional_text_edits: None,
                    command: None,
                    commit_characters: None,
                    data: None,
                    tags: None,
                };
            })
            .collect(),
    };
}

fn main_loop(connection: Connection, params: serde_json::Value) -> Result<()> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();

    info!("STARTING EXAMPLE MAIN LOOP");

    for msg in &connection.receiver {
        let result = match msg {
            Message::Notification(not) => handle_notification(not),
            Message::Request(req) => handle_request(req),
            _ => handle_other(msg),
        };

        match match result {
            Some(HtmxResult::AttributeCompletion(c)) => {
                let str = match serde_json::to_value(&to_completion_list(c.items)) {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                error!("sending response {:?}", str);
                // TODO: block requests that have been cancelled
                connection.sender.send(Message::Response(Response {
                    id: c.id,
                    result: Some(str),
                    error: None,
                }))
            }
            Some(HtmxResult::Diagnostic) => todo!(),
            None => continue,
        } {
            Ok(_) => error!("sent response"),
            Err(e) => error!("failed to send response: {:?}", e),
        };
    }

    return Ok(());
}

pub fn start_lsp() -> Result<()> {
    init_text_store();
    init_hx_tags();

    // Note that  we must have our logging only write out to stderr.
    info!("starting generic LSP server");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        completion_provider: Some(lsp_types::CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: Some(vec!["-".to_string()]),
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: None,
            },
            all_commit_characters: None,
            completion_item: None,
        }),
        ..Default::default()
    })
    .unwrap();

    let initialization_params = connection.initialize(server_capabilities)?;
    main_loop(connection, initialization_params)?;
    io_threads.join()?;

    // Shut down gracefully.
    warn!("shutting down server");
    Ok(())
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    #[test]
    fn test_byte_col() -> Result<()> {
        /*
        let source = "oeunth";

        let (line, col) = byte_pos_to_line_col(source.as_str(), msg.position.0);
        assert_eq!(line, 9);
        assert_eq!(col, 9);

        let (line, col) = byte_pos_to_line_col(source.as_str(), msg.position.1);
        assert_eq!(line, 9);
        assert_eq!(col, 21);

        */
        return Ok(());
    }
}
