use lsp_server::{Connection, ExtractError, Message, Request, RequestId, Response};
use lsp_types::{
    notification::DidOpenTextDocument, request::DocumentSymbolRequest, DocumentSymbolResponse,
    InitializeParams, Location, OneOf, Position, Range, ServerCapabilities, SymbolInformation,
    SymbolKind, TextDocumentSyncCapability, TextDocumentSyncKind, Uri,
};
use once_cell::sync::Lazy;
use std::{collections::HashMap, error::Error, fs, str::FromStr};

use crate::{
    ast::Instruction,
    lexer::{self, Lexer},
    parser::RexxParser,
};
static EMPTY: Lazy<String> = Lazy::new(|| String::from("label:"));
pub fn run_lsp() -> Result<(), Box<dyn Error + Sync + Send>> {
    let (connection, io_threads) = Connection::stdio();
    let server_capabilities = serde_json::to_value(&ServerCapabilities {
        text_document_sync: Some(TextDocumentSyncCapability::Kind(TextDocumentSyncKind::FULL)),
        document_symbol_provider: Some(OneOf::Left(true)),
        ..Default::default()
    })
    .unwrap();
    let initialization_params = match connection.initialize(server_capabilities) {
        Ok(it) => it,
        Err(e) => {
            if e.channel_is_disconnected() {
                io_threads.join()?;
            }
            return Err(e.into());
        }
    };
    lsp_loop(connection, initialization_params)?;
    io_threads.join()?;
    // Shut down gracefully.
    eprintln!("Shutting down LSP server");
    Ok(())
}

fn lsp_loop(
    connection: Connection,
    params: serde_json::Value,
) -> Result<(), Box<dyn Error + Sync + Send>> {
    let _params: InitializeParams = serde_json::from_value(params).unwrap();
    eprintln!("starting example main loop");
    for msg in &connection.receiver {
        eprintln!("got msg: {msg:?}");
        match msg {
            Message::Request(req) => {
                if connection.handle_shutdown(&req)? {
                    return Ok(());
                }
                eprintln!("got request: {req:?}");
                match cast::<DocumentSymbolRequest>(req) {
                    Ok((id, params)) => {
                        eprintln!("got DocumentSymbolRequest request #{id}: {params:?}");
                        let src = fs::read_to_string(params.text_document.uri.path().as_str()).unwrap_or(EMPTY.to_string());
                        let mut lexer = Lexer::new(src.as_str());
                        let mut parser = RexxParser::new(&mut lexer);
                        let sym = parser.parse().unwrap();
                        let result = Some(DocumentSymbolResponse::Flat(
                            sym.instructions
                                .iter()
                                .filter(|t| match t {
                                    Instruction::Label(_) => true,
                                    _ => false,
                                })
                                .map(|x| -> SymbolInformation {
                                    match x {
                                        Instruction::Label(x) => SymbolInformation {
                                            name: parser.get_text(x).to_string(),
                                            kind: SymbolKind::FUNCTION,
                                            tags: None,
                                            deprecated: None,
                                            location: Location {
                                                uri: params.text_document.uri.clone(),
                                                range: Range {
                                                    start: Position {
                                                        line: x.range.start.line as u32,
                                                        character: x.range.start.character as u32,
                                                    },
                                                    end: Position {
                                                        line: x.range.end.line as u32,
                                                        character: x.range.end.character as u32,
                                                    },
                                                },
                                            },
                                            container_name: None,
                                        },
                                        _ => SymbolInformation {
                                            name: "Unknown".to_string(),
                                            kind: todo!(),
                                            tags: todo!(),
                                            deprecated: todo!(),
                                            location: todo!(),
                                            container_name: todo!(),
                                        },
                                    }
                                })
                                .collect(),
                        ));
                        let result = serde_json::to_value(&result).unwrap();
                        let resp = Response {
                            id,
                            result: Some(result),
                            error: None,
                        };
                        connection.sender.send(Message::Response(resp))?;
                        continue;
                    }
                    Err(err @ ExtractError::JsonError { .. }) => panic!("{err:?}"),
                    Err(ExtractError::MethodMismatch(req)) => req,
                };
                // ...
            }
            Message::Response(resp) => {
                eprintln!("got response: {resp:?}");
            }

            Message::Notification(not) => {
                eprintln!("got notification: {not:?}");
            }
        }
    }
    Ok(())
}

fn cast<R>(req: Request) -> Result<(RequestId, R::Params), ExtractError<Request>>
where
    R: lsp_types::request::Request,
    R::Params: serde::de::DeserializeOwned,
{
    req.extract(R::METHOD)
}

