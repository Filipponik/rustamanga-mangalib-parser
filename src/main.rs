mod config;
mod mangalib;
mod server;

#[tokio::main]
async fn main() {
    config::setup_tracing();
    server::serve().await;
}
