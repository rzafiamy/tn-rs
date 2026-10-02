//! HTTP API of the tn normalizer.
//!
//! - `GET /health` → `{"status":"healthy","version","lexicon_entries"}`
//! - `POST /v1/normalize` `{"text", "language"?, "mode"?}` →
//!   `{"text", "language", "mode"}`; `texts` (a list) instead of `text`
//!   returns `texts`. `language`: code or name (`fr`, `en-US`, `french`),
//!   default `en`; `mode`: `strict` (default) or `safe`.
//!
//! The lexicon file is re-read when its modification time changes, so edits
//! apply to the next request without a restart.

use axum::extract::State;
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::Deserialize;
use serde_json::json;
use std::path::PathBuf;
use std::sync::{Arc, RwLock};
use std::time::SystemTime;
use tn::{Lang, Lexicon, Mode, normalize_text};

/// Lexicon file and its last loaded version.
#[derive(Default)]
pub struct LexiconSource {
    path: Option<PathBuf>,
    loaded: RwLock<(Option<SystemTime>, Option<Arc<Lexicon>>)>,
}

impl LexiconSource {
    /// Loads `path` now (an error here stops the server from starting).
    pub fn open(path: Option<PathBuf>) -> Result<Self, String> {
        let src = LexiconSource {
            path,
            loaded: RwLock::new((None, None)),
        };
        src.current()?;
        Ok(src)
    }

    /// The lexicon, reloaded if the file changed. A file that became invalid
    /// keeps the previous version and reports the error.
    pub fn current(&self) -> Result<Option<Arc<Lexicon>>, String> {
        let Some(path) = &self.path else {
            return Ok(None);
        };
        let mtime = std::fs::metadata(path)
            .and_then(|m| m.modified())
            .map_err(|e| format!("cannot read lexicon {}: {e}", path.display()))?;
        {
            let loaded = self.loaded.read().unwrap();
            if loaded.0 == Some(mtime) {
                return Ok(loaded.1.clone());
            }
        }
        let lex = Arc::new(Lexicon::load(path)?);
        tracing::info!("lexicon {}: {} entries", path.display(), lex.len());
        *self.loaded.write().unwrap() = (Some(mtime), Some(lex.clone()));
        Ok(Some(lex))
    }
}

pub type AppState = Arc<LexiconSource>;

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/v1/normalize", post(normalize))
        .with_state(state)
}

async fn health(State(state): State<AppState>) -> Response {
    match state.current() {
        Ok(lex) => Json(json!({
            "status": "healthy",
            "version": env!("CARGO_PKG_VERSION"),
            "lexicon_entries": lex.map_or(0, |l| l.len()),
        }))
        .into_response(),
        Err(e) => error(StatusCode::SERVICE_UNAVAILABLE, &e),
    }
}

#[derive(Deserialize)]
struct NormalizeRequest {
    text: Option<String>,
    texts: Option<Vec<String>>,
    language: Option<String>,
    mode: Option<String>,
}

async fn normalize(State(state): State<AppState>, Json(req): Json<NormalizeRequest>) -> Response {
    let lang = Lang::from_code(req.language.as_deref().unwrap_or("en"));
    let mode = match req.mode.as_deref() {
        None => Mode::Strict,
        Some(m) => match Mode::parse(m) {
            Some(m) => m,
            None => return error(StatusCode::BAD_REQUEST, "mode must be 'strict' or 'safe'"),
        },
    };
    let lex = match state.current() {
        Ok(l) => l,
        Err(e) => {
            tracing::warn!("{e}");
            None
        }
    };
    let run = |t: &str| normalize_text(t, lang, mode, lex.as_deref());
    let body = match (req.text, req.texts) {
        (Some(t), None) => json!({"text": run(&t), "language": lang.code(), "mode": mode.as_str()}),
        (None, Some(ts)) => json!({
            "texts": ts.iter().map(|t| run(t)).collect::<Vec<_>>(),
            "language": lang.code(),
            "mode": mode.as_str(),
        }),
        _ => return error(StatusCode::BAD_REQUEST, "send either 'text' or 'texts'"),
    };
    Json(body).into_response()
}

fn error(status: StatusCode, message: &str) -> Response {
    (status, Json(json!({"error": {"message": message}}))).into_response()
}
