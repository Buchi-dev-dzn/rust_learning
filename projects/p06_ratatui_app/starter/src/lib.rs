//! ratatui で作る 2 ペインのファイルエクスプローラ (starter)。
//!
//! - `app` : 状態とキー操作 (テスト可能)
//! - `ui`  : 状態を読んで画面を組み立てる
//!
//! STEPS.md の順に `TODO(STEP n)` を埋めていこう。

// starter では未使用のものが多いので警告を抑える (完成したら消してよい)
#![allow(unused_variables, unused_imports, dead_code)]

pub mod app;
pub mod ui;

pub use app::App;
