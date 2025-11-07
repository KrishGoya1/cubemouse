// server/transport.rs
use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::Mutex;

use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

use futures_util::{StreamExt};

use crate::server::protocol::{Packet, parse_packet};
use crate::server::input::handle_move;

pub async fn run_server(addr: &str) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on {}", addr);

    // Keep track of a single active client (if you want multiple clients, change this)
    let active_client: Arc<Mutex<Option<SocketAddr>>> = Arc::new(Mutex::new(None));

    loop {
        let (tcp_stream, client_addr) = listener.accept().await?;
        let active_client = active_client.clone();

        // Try to mark this client as the active one (reject if one exists)
        {
            let mut guard = active_client.lock().await;
            if guard.is_some() {
                println!("Rejected new client: {} (already connected)", client_addr);
                continue;
            } else {
                println!("Client connected (handshake pending): {}", client_addr);
                *guard = Some(client_addr);
            }
        }

        // Spawn handling task
        let active_client_task = active_client.clone();
        tokio::spawn(async move {
            // Perform WebSocket handshake
            let ws_stream = match accept_async(tcp_stream).await {
                Ok(ws) => ws,
                Err(e) => {
                    eprintln!("WebSocket handshake error from {}: {:?}", client_addr, e);
                    // Make sure to clear active_client if we set it earlier
                    let mut guard = active_client_task.lock().await;
                    if guard.as_ref().map(|a| a == &client_addr).unwrap_or(false) {
                        *guard = None;
                    }
                    return;
                }
            };

            println!("WebSocket established: {}", client_addr);

            // split into (sink, stream) -> (write, read)
            let (mut _write, mut read) = ws_stream.split();

            // Read loop — we only read binary frames (your protocol)
            while let Some(msg_result) = read.next().await {
                match msg_result {
                    Ok(Message::Binary(data)) => {
                        if let Some(packet) = parse_packet(&data) {
                            match packet {
                                Packet::Move { dx, dy } => {
                                    handle_move(dx, dy);
                                }
                                // Add other packet handling here.
                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        println!("Client {} requested close", client_addr);
                        break;
                    }
                    Ok(_) => {
                        // Ignore text/pings/pongs if not needed
                    }
                    Err(e) => {
                        eprintln!("WebSocket read error from {}: {:?}", client_addr, e);
                        break;
                    }
                }
            }

            println!("Client disconnected: {}", client_addr);

            // Clear active client only if it is this client
            let mut guard = active_client_task.lock().await;
            if guard.as_ref().map(|a| a == &client_addr).unwrap_or(false) {
                *guard = None;
            }
        });
    }
}
