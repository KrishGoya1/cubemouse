use std::net::SocketAddr;
use std::sync::Arc;

use tokio::net::TcpListener;
use tokio::sync::Mutex;

use tokio_tungstenite::accept_async;
use tokio_tungstenite::tungstenite::Message;

use futures_util::{StreamExt};

use crate::server::protocol::{Packet, parse_packet};
use crate::server::input::{handle_move,handle_left_click,handle_right_click,handle_scroll};

pub async fn run_server(addr: &str) -> anyhow::Result<()> {
    let listener = TcpListener::bind(addr).await?;
    println!("WebSocket server listening on {}", addr);

    let active_client: Arc<Mutex<Option<SocketAddr>>> = Arc::new(Mutex::new(None));

    loop {
        let (tcp_stream, client_addr) = listener.accept().await?;
        let active_client = active_client.clone();

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

        let active_client_task = active_client.clone();
        tokio::spawn(async move {
            let ws_stream = match accept_async(tcp_stream).await {
                Ok(ws) => ws,
                Err(e) => {
                    eprintln!("WebSocket handshake error from {}: {:?}", client_addr, e);
                    let mut guard = active_client_task.lock().await;
                    if guard.as_ref().map(|a| a == &client_addr).unwrap_or(false) {
                        *guard = None;
                    }
                    return;
                }
            };

            println!("WebSocket established: {}", client_addr);

            let (mut _write, mut read) = ws_stream.split();

            while let Some(msg_result) = read.next().await {
                match msg_result {
                    Ok(Message::Binary(data)) => {
                        if let Some(packet) = parse_packet(&data) {
                            match packet {
                                    Packet::Move { dx, dy } => handle_move(dx, dy),
                                    Packet::LeftClick => handle_left_click(),
                                    Packet::RightClick => handle_right_click(),
                                    Packet::Scroll { dy } => handle_scroll(dy),

                            }
                        }
                    }
                    Ok(Message::Close(_)) => {
                        println!("Client {} requested close", client_addr);
                        break;
                    }
                    Ok(_) => {
                    }
                    Err(e) => {
                        eprintln!("WebSocket read error from {}: {:?}", client_addr, e);
                        break;
                    }
                }
            }

            println!("Client disconnected: {}", client_addr);

            let mut guard = active_client_task.lock().await;
            if guard.as_ref().map(|a| a == &client_addr).unwrap_or(false) {
                *guard = None;
            }
        });
    }
}
