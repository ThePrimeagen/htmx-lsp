mod handle;

use anyhow::Result;
use log::{info, warn};
use lsp_types::{
    InitializeParams, ServerCapabilities, TextDocumentSyncCapability, TextDocumentSyncKind,
};

use lsp_server::{Connection, Message};

use crate::handle::handle_notification;

// TODO: I cannot find this in the rust-analyzer project (owner of lsp_types / server)
// I must be duplicating work
/*
fn send_diagnostics(connection: &Connection, perfs: PerfDiagnostic) -> Result<()> {
    let params = PublishDiagnosticsParams {
        uri: Url::parse(&perfs.uri)?,
        diagnostics: perfs.diagnostics,
        version: None,
    };
    let not = Notification::new("textDocument/publishDiagnostics".to_string(), params);
    connection.sender.send(Message::Notification(not))?;
    Ok(())
}
        */

fn main_loop(connection: Connection, params: serde_json::Value) -> Result<()> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    info!("starting example main loop");
    for msg in &connection.receiver {
        match msg {
            Message::Notification(not) => {
                match handle_notification(not) {
                    Some(perfs) => {
                        //send_diagnostics(&connection, perfs)?;
                    }
                    None => {}
                }
            }
            _ => {}
        }
    }
    Ok(())
}

pub fn start_lsp() -> Result<()> {
    // Note that  we must have our logging only write out to stderr.
    info!("starting generic LSP server");

    // Create the transport. Includes the stdio (stdin and stdout) versions but this could
    // also be implemented to use sockets or HTTP.
    let (connection, io_threads) = Connection::stdio();

    // Run the server and wait for the two threads to end (typically by trigger LSP Exit event).
    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
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
