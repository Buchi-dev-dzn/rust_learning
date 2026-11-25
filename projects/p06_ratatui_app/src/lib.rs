//! ratatui で作る 2 ペインのファイルエクスプローラ。
//!
//! - `app` : 状態 (カレントディレクトリ・一覧・選択・プレビュー) とキー操作。テスト可能。
//! - `ui`  : 状態を読んで画面を組み立てる。TestBackend でテストできる。
//!
//! 端末の初期化とイベントループは `main.rs` が行う。

pub mod app;
pub mod ui;

pub use app::App;
