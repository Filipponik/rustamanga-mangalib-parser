pub const DEFAULT_APP_PORT: u16 = 8000;
pub const DEFAULT_SEMAPHORE_PERMITS: usize = 16;

pub fn config_tracing() {
    tracing_subscriber::fmt()
        .with_level(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true)
        .with_thread_ids(true)
        .with_env_filter("rustamanga_mangalib_parser=debug")
        .init();
}
