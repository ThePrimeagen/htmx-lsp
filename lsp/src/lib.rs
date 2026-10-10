mod handle;
mod htmx;
mod text_store;
mod tree_sitter;
mod tree_sitter_querier;

use anyhow::Result;
use htmx::HxCompletionValue;
use log::{debug, error, info};
use lsp_types::{
    Command, CompletionItem, CompletionItemKind, CompletionList, HoverContents, InitializeParams,
    InsertTextFormat, MarkupContent, ServerCapabilities, TextDocumentSyncCapability,
    TextDocumentSyncKind, WorkDoneProgressOptions,
};

use lsp_server::{Connection, Message, Response};

use crate::{
    handle::{handle_notification, handle_other, handle_request, HtmxResult},
    text_store::init_text_store,
};

fn to_completion_list(items: HxCompletionValue) -> CompletionList {
    match items {
        HxCompletionValue::AttributeName(items) => CompletionList {
            is_incomplete: true,
            items: items
                .to_vec()
                .iter()
                .map(|x| CompletionItem {
                    label: x.name.to_string(),
                    kind: Some(CompletionItemKind::VALUE),
                    detail: Some(x.desc.to_string()),
                    // TODO: Figure out if we can use edit_text instead of insert_text here
                    insert_text: Some(x.name.to_string() + "=\"$1\""),
                    insert_text_format: Some(InsertTextFormat::SNIPPET),
                    command: Some(Command {
                        title: String::from("Suggest"),
                        command: "editor.action.triggerSuggest".to_string(),
                        arguments: None,
                    }),
                    ..Default::default()
                })
                .collect(),
        },
        HxCompletionValue::AttributeValue(items) => CompletionList {
            is_incomplete: true,
            items: items
                .to_vec()
                .iter()
                .map(|x| CompletionItem {
                    label: x.name.to_string(),
                    kind: Some(CompletionItemKind::PROPERTY),
                    detail: Some(x.desc.to_string()),
                    ..Default::default()
                })
                .collect(),
        },
    }
}

fn main_loop(connection: Connection, params: serde_json::Value) -> Result<()> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();

    info!("STARTING EXAMPLE MAIN LOOP");

    for msg in &connection.receiver {
        error!("connection received message: {:?}", msg);
        let req_id = match &msg {
            Message::Request(ref req) => Some(req.id.clone()),
            _ => None,
        };
        let result = match msg {
            Message::Notification(not) => handle_notification(not),
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                handle_request(req)
            }
            _ => handle_other(msg),
        };

        match match result {
            Some(HtmxResult::AttributeCompletion(c)) => {
                let str = match serde_json::to_value(to_completion_list(c.items)) {
                    Ok(s) => s,
                    Err(_) => continue,
                };

                // TODO: block requests that have been cancelled
                connection.sender.send(Message::Response(Response {
                    id: c.id,
                    result: Some(str),
                    error: None,
                }))
            }

            Some(HtmxResult::AttributeHover(hover_resp)) => {
                debug!("main_loop - hover response: {:?}", hover_resp);
                let hover_response = lsp_types::Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: lsp_types::MarkupKind::Markdown,
                        value: hover_resp.value.to_string(),
                    }),
                    range: None,
                };

                let str = match serde_json::to_value(&hover_response) {
                    Ok(s) => s,
                    Err(err) => {
                        error!("Fail to parse hover_response: {:?}", err);
                        return Err(anyhow::anyhow!("Fail to parse hover_response"));
                    }
                };

                connection.sender.send(Message::Response(Response {
                    id: hover_resp.id,
                    result: Some(str),
                    error: None,
                }))
            }
            None => {
                // If we receive a request, we *must* send a response. Otherwise, the Neovim
                // client will not work with multiple servers
                if let Some(id) = req_id {
                    connection.sender.send(Message::Response(Response {
                        id,
                        result: None,
                        error: Some(lsp_server::ResponseError {
                            code: lsp_server::ErrorCode::RequestFailed as i32,
                            message: "No information available".to_string(),
                            data: None,
                        }),
                    }))
                } else {
                    Ok(())
                }
            }
        } {
            Ok(_) => {}
            Err(e) => error!("failed to send response: {:?}", e),
        };
    }

    Ok(())
}

pub fn start_lsp() -> Result<()> {
    init_text_store();

    // Note that  we must have our logging only write out to stderr.
    info!("starting generic LSP server");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let server_capabilities = serde_json::to_value(ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        completion_provider: Some(lsp_types::CompletionOptions {
            resolve_provider: Some(false),
            trigger_characters: Some(vec!["-".to_string(), "\"".to_string(), " ".to_string()]),
            work_done_progress_options: WorkDoneProgressOptions {
                work_done_progress: None,
            },
            all_commit_characters: None,
            completion_item: None,
        }),

        hover_provider: Some(lsp_types::HoverProviderCapability::Simple(true)),

        ..Default::default()
    })
    .unwrap();

    let initialization_params = connection.initialize(server_capabilities)?;
    main_loop(connection, initialization_params)?;
    io_threads.join()?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use std::{thread, time::Duration};

    use anyhow::Result;
    use lsp_server::{Connection, Message, Notification, Request, RequestId};
    use serde_json::Value;

    use super::main_loop;

    #[test]
    fn shutdown_request_exits_gracefully() -> Result<()> {
        let (server, client) = Connection::memory();
        let server_thread =
            thread::spawn(move || main_loop(server, serde_json::json!({ "capabilities": {} })));
        let request_id = RequestId::from(1);

        client.sender.send(Message::Request(Request {
            id: request_id.clone(),
            method: "shutdown".to_string(),
            params: Value::Null,
        }))?;

        let response = match client.receiver.recv_timeout(Duration::from_secs(1))? {
            Message::Response(response) => response,
            message => panic!("expected shutdown response, got {message:?}"),
        };
        assert_eq!(response.id, request_id);
        assert_eq!(response.result, Some(Value::Null));
        assert!(response.error.is_none());

        client.sender.send(Message::Notification(Notification {
            method: "exit".to_string(),
            params: Value::Null,
        }))?;
        drop(client);

        server_thread.join().expect("server thread panicked")?;
        Ok(())
    }
}
