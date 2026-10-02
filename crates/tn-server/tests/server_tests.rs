use axum::body::Body;
use axum::http::{Request, StatusCode};
use serde_json::{Value, json};
use std::sync::Arc;
use tn_server::{LexiconSource, router};
use tower::ServiceExt;

async fn call(
    app: axum::Router,
    method: &str,
    uri: &str,
    body: Option<Value>,
) -> (StatusCode, Value) {
    let req = Request::builder()
        .method(method)
        .uri(uri)
        .header("content-type", "application/json");
    let req = match body {
        Some(b) => req.body(Body::from(b.to_string())).unwrap(),
        None => req.body(Body::empty()).unwrap(),
    };
    let resp = app.oneshot(req).await.unwrap();
    let status = resp.status();
    let bytes = axum::body::to_bytes(resp.into_body(), 1 << 20)
        .await
        .unwrap();
    (
        status,
        serde_json::from_slice(&bytes).unwrap_or(Value::Null),
    )
}

fn app(lexicon: Option<std::path::PathBuf>) -> axum::Router {
    router(Arc::new(LexiconSource::open(lexicon).unwrap()))
}

/// covers: REQ-API-001
#[tokio::test]
async fn health_reports_version() {
    let (status, body) = call(app(None), "GET", "/health", None).await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["status"], "healthy");
    assert_eq!(body["lexicon_entries"], 0);
}

/// covers: REQ-API-001, REQ-TXT-001
#[tokio::test]
async fn normalizes_text_and_batches() {
    let (status, body) = call(
        app(None),
        "POST",
        "/v1/normalize",
        Some(json!({"text": "Rendez-vous à 9h30.", "language": "fr-FR"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["text"], "Rendez-vous à neuf heures trente.");
    assert_eq!(body["language"], "fr");
    assert_eq!(body["mode"], "strict");

    let (status, body) = call(
        app(None),
        "POST",
        "/v1/normalize",
        Some(json!({"texts": ["Gate 12B.", "It is 5%."], "language": "en", "mode": "safe"})),
    )
    .await;
    assert_eq!(status, StatusCode::OK);
    assert_eq!(body["texts"], json!(["Gate 12B.", "It is five percent."]));
}

/// covers: REQ-API-001
#[tokio::test]
async fn rejects_bad_requests() {
    let (status, _) = call(
        app(None),
        "POST",
        "/v1/normalize",
        Some(json!({"language": "fr"})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    let (status, body) = call(
        app(None),
        "POST",
        "/v1/normalize",
        Some(json!({"text": "x", "mode": "loose"})),
    )
    .await;
    assert_eq!(status, StatusCode::BAD_REQUEST);
    assert!(body["error"]["message"].as_str().unwrap().contains("mode"));
}

/// covers: REQ-LEX-001, REQ-LEX-003
#[tokio::test]
async fn lexicon_is_reloaded_when_edited() {
    let dir = std::env::temp_dir().join(format!("tn-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("lexicon.tsv");
    std::fs::write(&path, "Nvidia\tène vidia\n").unwrap();
    let app = app(Some(path.clone()));
    let ask = |app: axum::Router| async move {
        call(
            app,
            "POST",
            "/v1/normalize",
            Some(json!({"text": "Nvidia.", "language": "fr"})),
        )
        .await
        .1
    };
    assert_eq!(ask(app.clone()).await["text"], "ène vidia.");
    // a later modification time triggers a reload
    std::thread::sleep(std::time::Duration::from_millis(1100));
    std::fs::write(&path, "Nvidia\tenne vidia\n").unwrap();
    assert_eq!(ask(app.clone()).await["text"], "enne vidia.");
    std::fs::remove_dir_all(&dir).unwrap();
}
