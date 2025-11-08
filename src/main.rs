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
    println!("🖱️  CubeMouse server starting...");

    // 1) Determine local IP (falls back to 127.0.0.1)
    let ip = local_ip().unwrap_or_else(|_| "127.0.0.1".parse().unwrap());
    let ws_url = format!("ws://{}:9000", ip);
    let http_url = format!("http://{}:8080", ip);

    println!("\n🌐 WebSocket: {}\n", ws_url);

    // 2) Generate Unicode QR and print it to terminal (uses qrcode crate)
    println!("📱 Scan this QR to open touchpad:\n");
    let code = QrCode::new(http_url.as_bytes())?;
    // Render using the dense unicode renderer (2 pixels per symbol vertically)
    let qr_string = code
        .render::<unicode::Dense1x2>()
        .quiet_zone(true)
        .build();
    println!("{}", qr_string);
    println!("👉 Or open manually: {}\n", http_url);

    // 3) Prepare static web directory (web/)
    let web_dir = std::env::current_dir()?.join("web");
    if !web_dir.exists() {
        eprintln!("⚠️  'web/' folder not found. Please create 'web/index.html' and any assets.");
    }

    // 4) Route: GET /ws-url -> returns {"ws_url": "<ws_url>"}
    let ws_url_clone = ws_url.clone();
    let ws_info = warp::path("ws-url").map(move || warp::reply::json(&json!({ "ws_url": ws_url_clone })));

    // 5) Route: static files from web/
    let static_files = warp::fs::dir(web_dir);

    // 6) Combine routes: /ws-url OR files from web/
    let routes = ws_info.or(static_files);

    // 7) Launch WebSocket server in background task
    let ws_task = task::spawn(async {
        if let Err(e) = server::transport::run_server("0.0.0.0:9000").await {
            eprintln!("WebSocket server error: {:?}", e);
        }
    });

    // 8) Run warp HTTP server (this blocks until shutdown)
    warp::serve(routes).run(([0, 0, 0, 0], 8080)).await;

    // 9) Wait for ws task to finish if it does
    let _ = ws_task.await;

    Ok(())
}
