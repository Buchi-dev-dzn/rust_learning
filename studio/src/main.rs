//! Rust 学習サイト "studio"
//!
//! `cargo studio` で起動し、ブラウザで http://127.0.0.1:7878 を開く。
//! 演習のビルド・テスト、プロジェクトのビルド、ブラウザ内ターミナル (PTY) をすべてこのサーバが受け持つ。
//! 任意コードを実行するサーバなので 127.0.0.1 にのみ bind する。

mod course;
mod markdown;
mod pages;
mod progress;
mod pty;
mod runner;

use axum::Router;
use axum::body::Body;
use axum::extract::{Path, Query, State, WebSocketUpgrade};
use axum::http::StatusCode;
use axum::response::sse::{Event, KeepAlive, Sse};
use axum::response::{IntoResponse, Response};
use axum::routing::{get, post};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::{Path as FsPath, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};
use tokio::sync::broadcast;
use tokio_stream::StreamExt;
use tokio_stream::wrappers::BroadcastStream;
use tower_http::services::ServeDir;

#[derive(Clone)]
pub struct AppState {
    pub root: PathBuf,
    pub store: Arc<Mutex<progress::Store>>,
    pub events: broadcast::Sender<String>,
}

fn repo_root() -> PathBuf {
    if let Ok(r) = std::env::var("STUDIO_ROOT") {
        return PathBuf::from(r);
    }
    let here = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    here.parent().unwrap().canonicalize().unwrap_or_else(|_| here.parent().unwrap().to_path_buf())
}

/// course/exercises の原本から、学習者用の exercises/ へ未作成のファイルだけコピーする
pub fn sync_exercises(root: &FsPath) -> usize {
    let src = root.join("course/exercises");
    let mut copied = 0;
    let Ok(chapters) = fs::read_dir(&src) else { return 0 };
    for ch in chapters.flatten() {
        let Ok(files) = fs::read_dir(ch.path()) else { continue };
        for f in files.flatten() {
            let p = f.path();
            if p.extension().is_none_or(|x| x != "rs") {
                continue;
            }
            let rel = p.strip_prefix(&src).unwrap();
            let dst = root.join("exercises").join(rel);
            if !dst.exists() {
                let _ = fs::create_dir_all(dst.parent().unwrap());
                if fs::copy(&p, &dst).is_ok() {
                    copied += 1;
                }
            }
        }
    }
    copied
}

pub fn ensure_exercise_copy(root: &FsPath, id: &str) {
    let dst = root.join("exercises").join(format!("{id}.rs"));
    if !dst.exists() {
        let src = root.join("course/exercises").join(format!("{id}.rs"));
        let _ = fs::create_dir_all(dst.parent().unwrap());
        let _ = fs::copy(src, dst);
    }
}

/// 他サイトからのリクエスト (CSRF / DNS リバインディング) を拒否する。
/// このサーバは任意コードを実行できるため、Host と Origin がローカルであることを必須にする。
async fn local_only(req: axum::extract::Request, next: axum::middleware::Next) -> Response {
    fn is_local(host: &str) -> bool {
        let h = host.rsplit_once(':').map(|(h, _)| h).unwrap_or(host);
        matches!(h, "127.0.0.1" | "localhost" | "[::1]")
    }
    let headers = req.headers();
    let host_ok = headers.get("host").and_then(|v| v.to_str().ok()).is_some_and(is_local);
    let origin_ok = match headers.get("origin").and_then(|v| v.to_str().ok()) {
        None => true,
        Some(o) => o
            .strip_prefix("http://")
            .is_some_and(|rest| is_local(rest.trim_end_matches('/'))),
    };
    if !host_ok || !origin_ok {
        return (StatusCode::FORBIDDEN, "studio はローカルからのアクセスのみ受け付けます").into_response();
    }
    next.run(req).await
}

/// 演習 ID の検証 (パストラバーサル防止)
fn valid_id_part(s: &str) -> bool {
    !s.is_empty() && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
}

#[tokio::main]
async fn main() {
    let root = repo_root();
    let copied = sync_exercises(&root);
    let _ = fs::create_dir_all(root.join(".studio/bin"));
    let _ = fs::create_dir_all(root.join(".studio/play"));

    let (events, _) = broadcast::channel(256);
    let state = AppState { root: root.clone(), store: Arc::new(Mutex::new(progress::Store::open(&root))), events };
    tokio::spawn(watch_files(state.clone()));

    let app = Router::new()
        .route("/", get(pages::home))
        .route("/roadmap", get(pages::roadmap))
        .route("/stages/{id}", get(pages::stage))
        .route("/lessons", get(pages::lessons))
        .route("/lessons/{id}", get(pages::lesson))
        .route("/exercises", get(pages::exercises))
        .route("/exercises/{ch}/{name}", get(pages::exercise))
        .route("/exercises/{ch}/{name}/solution", get(pages::solution))
        .route("/projects", get(pages::projects))
        .route("/projects/{id}", get(pages::project))
        .route("/projects/{id}/steps/{n}", get(pages::project_step))
        .route("/final", get(pages::finals))
        .route("/final/{id}", get(pages::final_page))
        .route("/final/{id}/solution", get(pages::final_solution))
        .route("/terminal", get(pages::terminal))
        .route("/progress", get(pages::progress))
        .route("/reference", get(pages::reference))
        // ---- API
        .route("/api/exercises/{ch}/{name}/source", get(api_ex_source).put(api_ex_save))
        .route("/api/exercises/{ch}/{name}/test", post(api_ex_test))
        .route("/api/exercises/{ch}/{name}/reset", post(api_ex_reset))
        .route("/api/playground", post(api_playground))
        .route("/api/cargo/{kind}/{id}/{action}", post(api_cargo))
        .route("/api/files", get(api_file_get).put(api_file_put))
        .route("/api/finals/{id}/reset", post(api_final_reset))
        .route("/api/projects/{id}/steps/{n}", post(api_step))
        .route("/api/progress/reset", post(api_progress_reset))
        .route("/api/events", get(api_events))
        .route("/api/pty", get(api_pty))
        .nest_service("/static", ServeDir::new(root.join("studio/web")))
        .fallback(pages::fallback)
        .layer(axum::middleware::from_fn(local_only))
        .with_state(state);

    let base: u16 = std::env::var("STUDIO_PORT").ok().and_then(|p| p.parse().ok()).unwrap_or(7878);
    let mut listener = None;
    for port in base..base + 20 {
        if let Ok(l) = tokio::net::TcpListener::bind(("127.0.0.1", port)).await {
            listener = Some(l);
            break;
        }
        eprintln!("ポート {port} は使用中です。次を試します…");
    }
    let listener = listener.expect("空きポートが見つかりませんでした");
    let url = format!("http://{}", listener.local_addr().unwrap());
    println!();
    println!("  🦀 Rust 学習スタジオ を起動しました");
    println!("  ➜  ブラウザで開いてください: \x1b[1;36m{url}\x1b[0m");
    println!("  (リポジトリ: {})", root.display());
    if copied > 0 {
        println!("  演習ファイルを {copied} 個 exercises/ に用意しました");
    }
    println!("  終了するには Ctrl-C");
    println!();
    if std::env::var("STUDIO_NO_OPEN").is_err() && cfg!(target_os = "macos") {
        let _ = std::process::Command::new("open").arg(&url).spawn();
    }
    axum::serve(listener, app).await.unwrap();
}

// ------------------------------------------------------------------ 演習 API

fn ex_paths(st: &AppState, ch: &str, name: &str) -> Option<(String, PathBuf)> {
    if !valid_id_part(ch) || !valid_id_part(name) {
        return None;
    }
    let id = format!("{ch}/{name}");
    if !st.root.join("course/exercises").join(format!("{id}.rs")).exists() {
        return None;
    }
    ensure_exercise_copy(&st.root, &id);
    Some((id.clone(), st.root.join("exercises").join(format!("{id}.rs"))))
}

async fn api_ex_source(State(st): State<AppState>, Path((ch, name)): Path<(String, String)>) -> Response {
    match ex_paths(&st, &ch, &name) {
        Some((_, p)) => fs::read_to_string(p).unwrap_or_default().into_response(),
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn api_ex_save(State(st): State<AppState>, Path((ch, name)): Path<(String, String)>, body: String) -> Response {
    match ex_paths(&st, &ch, &name) {
        Some((_, p)) => match fs::write(p, body) {
            Ok(_) => StatusCode::NO_CONTENT.into_response(),
            Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
        },
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn api_ex_reset(State(st): State<AppState>, Path((ch, name)): Path<(String, String)>) -> Response {
    match ex_paths(&st, &ch, &name) {
        Some((id, p)) => {
            let src = st.root.join("course/exercises").join(format!("{id}.rs"));
            let text = fs::read_to_string(src).unwrap_or_default();
            let _ = fs::write(p, &text);
            text.into_response()
        }
        None => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn api_ex_test(State(st): State<AppState>, Path((ch, name)): Path<(String, String)>, body: String) -> Response {
    let Some((id, file)) = ex_paths(&st, &ch, &name) else { return StatusCode::NOT_FOUND.into_response() };
    if !body.is_empty() {
        let _ = fs::write(&file, &body);
    }
    let bin = st.root.join(".studio/bin").join(format!("{ch}__{name}"));
    let steps = runner::rustc_test_steps(&st.root, &file, &bin);
    let store = st.store.clone();
    let body = runner::stream(steps, move |ok| {
        store.lock().unwrap().record("exercise", &id, ok);
    });
    stream_response(body)
}

fn stream_response(body: Body) -> Response {
    Response::builder()
        .header("content-type", "text/plain; charset=utf-8")
        .header("cache-control", "no-cache")
        .header("x-content-type-options", "nosniff")
        .body(body)
        .unwrap()
}

static PLAY_COUNTER: AtomicU64 = AtomicU64::new(0);

async fn api_playground(State(st): State<AppState>, code: String) -> Response {
    let n = PLAY_COUNTER.fetch_add(1, Ordering::Relaxed) % 16;
    let dir = st.root.join(".studio/play");
    let file = dir.join(format!("main_{n}.rs"));
    let bin = dir.join(format!("main_{n}"));
    if let Err(e) = fs::write(&file, code) {
        return (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response();
    }
    stream_response(runner::stream(runner::rustc_run_steps(&st.root, &file, &bin), |_| {}))
}

// ------------------------------------------------------------------ cargo (プロジェクト / 卒業課題)

async fn api_cargo(State(st): State<AppState>, Path((kind, id, action)): Path<(String, String, String)>) -> Response {
    let Ok(c) = course::Course::load(&st.root) else { return StatusCode::INTERNAL_SERVER_ERROR.into_response() };
    let package = match kind.as_str() {
        "project" if c.project(&id).is_some() => id.clone(),
        "starter" if c.project(&id).is_some() => format!("{id}_starter"),
        "final" if c.final_(&id).is_some() => id.clone(),
        _ => return StatusCode::NOT_FOUND.into_response(),
    };
    let sub = match action.as_str() {
        "build" => "build",
        "test" => "test",
        _ => return StatusCode::BAD_REQUEST.into_response(),
    };
    let step = runner::Step::new("cargo", &[sub, "-p", &package, "--color", "always"], st.root.clone())
        .timeout(900)
        .display(format!("cargo {sub} -p {package}"));
    let store = st.store.clone();
    let record = kind == "final" && sub == "test";
    stream_response(runner::stream(vec![step], move |ok| {
        if record {
            store.lock().unwrap().record("final", &id, ok);
        }
    }))
}

async fn api_final_reset(State(st): State<AppState>, Path(id): Path<String>) -> Response {
    if !valid_id_part(&id) {
        return StatusCode::NOT_FOUND.into_response();
    }
    let src = st.root.join("course/finals").join(&id).join("starter/src");
    let dst = st.root.join("finals").join(&id).join("src");
    if !src.exists() {
        return StatusCode::NOT_FOUND.into_response();
    }
    match copy_tree(&src, &dst) {
        Ok(n) => format!("{n} ファイルを初期状態に戻しました").into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

fn copy_tree(src: &FsPath, dst: &FsPath) -> std::io::Result<usize> {
    let mut n = 0;
    fs::create_dir_all(dst)?;
    for e in fs::read_dir(src)? {
        let e = e?;
        let p = e.path();
        let d = dst.join(e.file_name());
        if p.is_dir() {
            n += copy_tree(&p, &d)?;
        } else {
            fs::copy(&p, &d)?;
            n += 1;
        }
    }
    Ok(n)
}

// ------------------------------------------------------------------ 汎用ファイル API (編集可能な範囲のみ)

#[derive(Deserialize)]
struct FileQuery {
    path: String,
}

/// 編集を許可するパスか: exercises/**.rs, finals/<id>/src/**.rs, projects/<id>/starter/src/**.rs
fn editable(rel: &str) -> bool {
    if rel.contains("..") || rel.starts_with('/') || rel.contains('\\') || !rel.ends_with(".rs") {
        return false;
    }
    let parts: Vec<&str> = rel.split('/').collect();
    if parts.iter().any(|p| p.is_empty() || !p.chars().all(|c| c.is_ascii_alphanumeric() || "_-.".contains(c))) {
        return false;
    }
    match parts.as_slice() {
        ["exercises", _, ..] => true,
        ["finals", _, "src", _, ..] => true,
        ["projects", _, "starter", "src", _, ..] => true,
        _ => false,
    }
}

async fn api_file_get(State(st): State<AppState>, Query(q): Query<FileQuery>) -> Response {
    if !editable(&q.path) {
        return StatusCode::FORBIDDEN.into_response();
    }
    match fs::read_to_string(st.root.join(&q.path)) {
        Ok(s) => s.into_response(),
        Err(_) => StatusCode::NOT_FOUND.into_response(),
    }
}

async fn api_file_put(State(st): State<AppState>, Query(q): Query<FileQuery>, body: String) -> Response {
    if !editable(&q.path) {
        return StatusCode::FORBIDDEN.into_response();
    }
    let p = st.root.join(&q.path);
    if !p.exists() {
        return StatusCode::NOT_FOUND.into_response();
    }
    match fs::write(p, body) {
        Ok(_) => StatusCode::NO_CONTENT.into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()).into_response(),
    }
}

// ------------------------------------------------------------------ 進捗

#[derive(Deserialize)]
struct StepQuery {
    done: u8,
}

async fn api_step(State(st): State<AppState>, Path((id, n)): Path<(String, usize)>, Query(q): Query<StepQuery>) -> Response {
    if !valid_id_part(&id) {
        return StatusCode::NOT_FOUND.into_response();
    }
    st.store.lock().unwrap().toggle_step(&id, n, q.done == 1);
    StatusCode::NO_CONTENT.into_response()
}

async fn api_progress_reset(State(st): State<AppState>) -> Response {
    st.store.lock().unwrap().reset();
    StatusCode::NO_CONTENT.into_response()
}

// ------------------------------------------------------------------ ファイル変更通知 (SSE)

async fn api_events(State(st): State<AppState>) -> Sse<impl tokio_stream::Stream<Item = Result<Event, std::convert::Infallible>>> {
    let rx = BroadcastStream::new(st.events.subscribe());
    let stream = rx.filter_map(|m| m.ok()).map(|path| Ok(Event::default().event("change").data(path)));
    Sse::new(stream).keep_alive(KeepAlive::new().interval(Duration::from_secs(15)))
}

/// exercises/ と finals/*/src, projects/*/starter/src を 1 秒ごとに走査し、変更されたファイルを通知する
async fn watch_files(st: AppState) {
    let mut known: HashMap<PathBuf, SystemTime> = HashMap::new();
    let mut first = true;
    loop {
        let root = st.root.clone();
        let snapshot = tokio::task::spawn_blocking(move || scan(&root)).await.unwrap_or_default();
        for (path, mtime) in &snapshot {
            let changed = known.get(path).is_none_or(|old| old != mtime);
            if changed && !first {
                let rel = path.strip_prefix(&st.root).unwrap_or(path).display().to_string();
                let _ = st.events.send(rel);
            }
        }
        known = snapshot;
        first = false;
        tokio::time::sleep(Duration::from_millis(800)).await;
    }
}

fn scan(root: &FsPath) -> HashMap<PathBuf, SystemTime> {
    let mut out = HashMap::new();
    fn walk(d: &FsPath, out: &mut HashMap<PathBuf, SystemTime>, depth: usize) {
        if depth > 6 {
            return;
        }
        let Ok(rd) = fs::read_dir(d) else { return };
        for e in rd.flatten() {
            let p = e.path();
            if p.is_dir() {
                walk(&p, out, depth + 1);
            } else if p.extension().is_some_and(|x| x == "rs") {
                if let Ok(m) = e.metadata().and_then(|m| m.modified()) {
                    out.insert(p, m);
                }
            }
        }
    }
    walk(&root.join("exercises"), &mut out, 0);
    for base in ["finals", "projects"] {
        if let Ok(rd) = fs::read_dir(root.join(base)) {
            for e in rd.flatten() {
                let src = if base == "finals" { e.path().join("src") } else { e.path().join("starter/src") };
                walk(&src, &mut out, 0);
            }
        }
    }
    out
}

// ------------------------------------------------------------------ ブラウザ内ターミナル

#[derive(Deserialize)]
struct PtyQuery {
    cmd: Option<String>,
    cols: Option<u16>,
    rows: Option<u16>,
}

async fn api_pty(State(st): State<AppState>, Query(q): Query<PtyQuery>, ws: WebSocketUpgrade) -> Response {
    let root = st.root.clone();
    ws.on_upgrade(move |socket| pty::session(socket, root, q.cmd, q.cols.unwrap_or(100), q.rows.unwrap_or(30)))
}

#[cfg(test)]
mod tests;
