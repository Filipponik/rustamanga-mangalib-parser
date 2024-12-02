mod mangalib;
mod server;
mod config;

#[tokio::main]
async fn main() {
    config::setup_tracing();
    server::serve().await;
}
