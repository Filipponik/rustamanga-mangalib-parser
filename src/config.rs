#![allow(dead_code)]
use once_cell::sync::OnceCell;
use tracing_appender::{
    non_blocking::{NonBlocking, WorkerGuard},
    rolling::{RollingFileAppender, Rotation},
};
use tracing_subscriber::{
    EnvFilter,
    fmt::{SubscriberBuilder, format::DefaultFields},
};

pub const DEFAULT_APP_PORT: u16 = 8000;
pub const DEFAULT_CHROME_MAX_COUNT: u16 = 16;
pub const DEFAULT_LOG_DIRECTORY_PATH: &str = "/var/log/rustamanga-mangalib-parser";

static GUARD: OnceCell<WorkerGuard> = OnceCell::new();

pub struct Logger;

#[allow(clippy::unused_self)]
impl Logger {
    pub fn new() -> Self {
        dotenv::dotenv().ok();
        Self
    }

    pub fn setup_json_file(&self) {
        GUARD.get_or_init(|| {
            let (writer, guard) = setup_file_non_blocking_writer();
            base_tracing().json().with_writer(writer).init();

            guard
        });
    }

    pub fn setup_console_text(&self) {
        base_tracing().init();
    }
}

fn base_tracing()
-> SubscriberBuilder<DefaultFields, tracing_subscriber::fmt::format::Format, EnvFilter> {
    tracing_subscriber::fmt()
        .with_level(true)
        .with_file(true)
        .with_line_number(true)
        .with_thread_names(true)
        .with_thread_ids(true)
        .with_env_filter("rustamanga_mangalib_parser=debug")
}

fn setup_file_non_blocking_writer() -> (NonBlocking, WorkerGuard) {
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
