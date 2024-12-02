pub fn setup_tracing() {
    dotenv::dotenv().ok();
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .with_file(true)
        .with_line_number(true)
        .with_thread_ids(true)
        .with_target(false)
        .with_env_filter("mangalib=debug")
        .init();
}