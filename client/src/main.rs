/*
 * Basic client to connect send recieve
 */

use futures_util::{stream::StreamExt, SinkExt};
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let url = "ws://127.0.0.1:8080";

    // Connect to the WebSocket server
    let (mut ws_stream, _) = connect_async(url).await?;

    let stdin = tokio::io::stdin();
    let mut stdin = BufReader::new(stdin).lines();

    loop {
        tokio::select! {
            // Read a message from stdin and send it
            line = stdin.next_line() => {
                match line {
                    Ok(Some(msg)) => {
                        ws_stream.send(Message::Text(msg)).await?;
                    }
                    Ok(None) => break,
                    Err(e) => return Err(e.into()),
                }
            },
            // Receive messages from the server
            msg = ws_stream.next() => {
                match msg {
                    Some(Ok(msg)) => {
                        if let Message::Text(text) = msg {
                            println!("Received: {}", text);
                        }
                    },
                    Some(Err(e)) => return Err(e.into()),
                    None => break,
                }
            },
        }
    }

    Ok(())
}
