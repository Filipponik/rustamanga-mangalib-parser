use once_cell::sync::OnceCell;
use tracing_appender::{
    non_blocking::{NonBlocking, WorkerGuard},
    rolling::{RollingFileAppender, Rotation},
};

pub const DEFAULT_APP_PORT: u16 = 8000;
pub const DEFAULT_CHROME_MAX_COUNT: u16 = 16;
pub const DEFAULT_LOG_DIRECTORY_PATH: &str = "/var/log/rustamanga-mangalib-parser";

static GUARD: OnceCell<WorkerGuard> = OnceCell::new();

pub fn setup_logging() {
    dotenv::dotenv().ok();
    GUARD.get_or_init(|| {
        let (writer, guard) = setup_writer();
        setup_tracing(writer);

        guard
    });
}

pub fn setup_tracing(writer: NonBlocking) {
    tracing_subscriber::fmt()
        .json()
        .with_level(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true)
        .with_thread_ids(true)
        .with_writer(writer)
        .init()
}

pub fn setup_writer() -> (NonBlocking, WorkerGuard) {
    let appender = RollingFileAppender::builder()
        .rotation(Rotation::NEVER)
        .filename_prefix("test")
        .filename_suffix("log")
        .build(
            std::env::var("LOG_DIRECTORY_PATH")
                .unwrap_or_else(|_| DEFAULT_LOG_DIRECTORY_PATH.to_string()),
        )
        .expect("Failed to initialize rolling file appender");

    tracing_appender::non_blocking(appender)
}
