mod server;

use server::transport::run_server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("CubeMouse server starting...");

    run_server("0.0.0.0:9000").await?;

    Ok(())
}