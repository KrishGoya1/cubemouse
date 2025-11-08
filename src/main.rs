mod server;

use anyhow::Result;
use local_ip_address::local_ip;
use qrcode::QrCode;
use qrcode::render::unicode;
use serde_json::json;
use tokio::task;
use warp::Filter;

#[tokio::main]
async fn main() -> Result<()> {
    println!("CubeMouse server starting...");

    let ip = local_ip().unwrap_or_else(|_| "127.0.0.1".parse().unwrap());
    let ws_url = format!("ws://{}:9000", ip);
    let http_url = format!("http://{}:8080", ip);

    println!("\nWebSocket: {}\n", ws_url);
    println!("device ip : {ip}");

    println!("Scan this QR to open touchpad:\n");
    let code = QrCode::new(http_url.as_bytes())?;

    let qr_string = code
        .render::<unicode::Dense1x2>()
        .quiet_zone(true)
        .build();
    println!("{}", qr_string);
    println!("Or open manually: {}\n", http_url);

    let web_dir = std::env::current_dir()?.join("web");
    if !web_dir.exists() {
        eprintln!("'web/' folder not found. Please create 'web/index.html' and any assets.");
    }

    let ws_url_clone = ws_url.clone();
    let ws_info = warp::path("ws-url").map(move || warp::reply::json(&json!({ "ws_url": ws_url_clone })));

    let static_files = warp::fs::dir(web_dir);

    let routes = ws_info.or(static_files);

    let ws_task = task::spawn(async {
        if let Err(e) = server::transport::run_server("0.0.0.0:9000").await {
            eprintln!("WebSocket server error: {:?}", e);
        }
    });

    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;

    let _ = ws_task.await;

    Ok(())
}
