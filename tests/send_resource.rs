#![allow(clippy::expect_used)]

use std::{
    io::Write,
    net::SocketAddr,
    sync::{
        Arc, Mutex as StdMutex, Once, OnceLock,
        atomic::{AtomicUsize, Ordering},
    },
    time::Duration,
};

use axum::{
    Router,
    extract::{Json, State},
    http::StatusCode,
    routing::post,
};
use reqwest::Client;
use rustamanga_mangalib_parser::{mangalib::MangaPreview, send_resource::send_resource};
use serde::Deserialize;
use tokio::{
    net::TcpListener,
    sync::{Mutex, oneshot},
};
use tracing::{Level, error};

const MANGALIB_STATIC_RESOURCE: &str = include_str!("../resource/json/mangalib_manga_list.json");

struct TestWriter {
    buf: Arc<StdMutex<Vec<u8>>>,
}

impl Write for TestWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.buf
            .lock()
            .expect("log buffer poisoned")
            .extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn init_tracing() -> Arc<StdMutex<Vec<u8>>> {
    static LOG_BUF: OnceLock<Arc<StdMutex<Vec<u8>>>> = OnceLock::new();
    static LOG_INIT: Once = Once::new();

    let buf = LOG_BUF
        .get_or_init(|| Arc::new(StdMutex::new(Vec::new())))
        .clone();

    LOG_INIT.call_once(|| {
        let writer_buf = buf.clone();
        let _ = tracing_subscriber::fmt()
            .with_max_level(Level::ERROR)
            .with_ansi(false)
            .with_writer(move || TestWriter {
                buf: writer_buf.clone(),
            })
            .try_init();
    });

    buf
}

#[derive(Deserialize)]
struct IncomingPreview {
    #[serde(alias = "type", rename = "manga_type")]
    r#type: String,
    name: String,
    url: String,
    slug: String,
    image_url: String,
}

impl From<IncomingPreview> for MangaPreview {
    fn from(value: IncomingPreview) -> Self {
        Self {
            r#type: value.r#type,
            name: value.name,
            url: value.url,
            slug: value.slug,
            image_url: value.image_url,
        }
    }
}

#[derive(Default)]
struct CaptureState {
    counter: AtomicUsize,
    received: Mutex<Vec<MangaPreview>>,
}

async fn handle_post(
    State(state): State<Arc<CaptureState>>,
    Json(manga): Json<IncomingPreview>,
) -> StatusCode {
    state.counter.fetch_add(1, Ordering::Relaxed);
    state.received.lock().await.push(manga.into());

    StatusCode::OK
}

async fn spawn_server(
    state: Arc<CaptureState>,
) -> (SocketAddr, oneshot::Sender<()>, tokio::task::JoinHandle<()>) {
    let app = Router::new()
        .route("/", post(handle_post))
        .with_state(state);

    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("failed to bind test server");

    let addr = listener.local_addr().expect("failed to get addr");
    let (shutdown_tx, shutdown_rx) = oneshot::channel();

    let server = axum::serve(listener, app).with_graceful_shutdown(async {
        let _ = shutdown_rx.await;
    });

    let handle = tokio::spawn(async move {
        if let Err(err) = server.await {
            error!(%err, "server error");
        }
    });
    (addr, shutdown_tx, handle)
}

#[tokio::test]
async fn send_resource_posts_first_mid_last_entries() {
    let state = Arc::new(CaptureState::default());
    let (addr, shutdown_tx, server_handle) = spawn_server(state.clone()).await;
    tokio::time::sleep(Duration::from_millis(50)).await;

    let url = format!("http://{addr}/");
    let client = Client::new();
    let warmup = MangaPreview {
        r#type: "warmup".to_string(),
        name: "warmup".to_string(),
        url: "http://warmup.local".to_string(),
        slug: "warmup".to_string(),
        image_url: "http://warmup.local/image".to_string(),
    };
    client
        .post(&url)
        .json(&warmup)
        .send()
        .await
        .expect("warmup request failed")
        .error_for_status()
        .expect("warmup non-2xx");

    state.counter.store(0, Ordering::Relaxed);
    state.received.lock().await.clear();

    let result = send_resource(&url).await;
    assert!(result.is_ok(), "send_resource returned error: {result:?}");

    // stop server and wait for completion
    let _ = shutdown_tx.send(());
    server_handle.await.expect("server task panicked");

    let expected: Vec<MangaPreview> =
        serde_json::from_str(MANGALIB_STATIC_RESOURCE).expect("failed to parse bundled json");
    let total = expected.len();

    assert_eq!(
        total,
        state.counter.load(Ordering::Relaxed),
        "expected send_resource to post all entries"
    );

    let received = state.received.lock().await;
    assert_eq!(
        total,
        received.len(),
        "expected all resources to be sent and captured"
    );

    let mid_idx = total / 2;
    let pairs = {
        let first = received[0].clone();
        let mid = received[mid_idx].clone();
        let last = received.last().expect("last record not captured").clone();
        vec![(first, 0usize), (mid, mid_idx), (last, total - 1)]
    };
    drop(received);

    for (actual, idx) in pairs {
        let expected = &expected[idx];
        assert_eq!(actual.slug, expected.slug);
        assert_eq!(actual.name, expected.name);
        assert_eq!(actual.r#type, expected.r#type);
        assert_eq!(actual.url, expected.url);
        assert_eq!(actual.image_url, expected.image_url);
    }
}

#[tokio::test]
async fn send_resource_logs_errors_on_failure() {
    let buf = init_tracing();
    buf.lock().expect("log buffer poisoned").clear();

    let url = "http://127.0.0.1:1/";

    let result = send_resource(url).await;
    assert!(result.is_ok(), "send_resource should swallow send errors");

    let logs = String::from_utf8(buf.lock().expect("log buffer poisoned").clone())
        .expect("logs should be utf8");
    assert!(
        logs.contains("Failed to send resource"),
        "expected error log in send_resource"
    );
}
