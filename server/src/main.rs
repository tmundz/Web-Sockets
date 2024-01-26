/*
 * Main file for the chat this will handle the WebSocket connections
 */

use futures_util::{SinkExt, StreamExt};
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio::sync::broadcast::{channel, Sender};
use tokio_tungstenite::tungstenite::Error as WsError;
use tokio_tungstenite::{accept_async, tungstenite::protocol::Message};

// This function handles individual WebSocket connections
async fn handle_connection(
    addr: SocketAddr,
    mut ws_stream: tokio_tungstenite::WebSocketStream<tokio::net::TcpStream>,
    bcast_tx: Sender<String>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let addr_str = addr.to_string();

    // Send a welcome message to the connected client
    let welcome_msg = format!("Welcome {}! Type a message", addr_str);
    ws_stream.send(Message::Text(welcome_msg)).await?;

    let mut bcast_rx = bcast_tx.subscribe();

    // Concurrently listen for incoming WebSocket messages and broadcast messages
    loop {
        tokio::select! {
            message = ws_stream.next() => match message {
                Some(Ok(msg)) => match msg {
                    Message::Text(text) => {
                        println!("From client {}: {}", addr_str, text);
                        let broadcast_msg = format!("{}: {}", addr_str, text);
                        bcast_tx.send(broadcast_msg)?;
                    },
                    _ => {}
                },
                Some(Err(e)) => match e {
                    WsError::ConnectionClosed | WsError::Protocol(_) | WsError::Utf8 => return Ok(()),
                    e => return Err(e.into()),
                },
                None => return Ok(()),
            },
            // handle messages recieved
            broadcast = bcast_rx.recv() => {
                match broadcast {
                    Ok(msg) => {
                        ws_stream.send(Message::Text(msg)).await?;
                    },
                    Err(e) => {
                        return Err(e.into());
                    }
                }
            }
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    // Create a broadcast channel
    let (bcast_tx, _) = channel(16);

    //  listen for incoming TCP connections
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("Server listening on port 8080");

    // Accept incoming connections in a loop
    while let Ok((stream, addr)) = listener.accept().await {
        println!("New connection from {}", addr);
        let bcast_tx = bcast_tx.clone();

        // Spawn a new task to handle the WebSocket connection
        tokio::spawn(async move {
            match accept_async(stream).await {
                Ok(ws_stream) => {
                    if let Err(e) = handle_connection(addr, ws_stream, bcast_tx).await {
                        eprintln!("Error handling connection: {}", e);
                    }
                }
                Err(e) => eprintln!("Error upgrading to WebSocket: {}", e),
            }
        });
    }

    Ok(())
}
