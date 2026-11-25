//! ansi_raw: ライブラリを使わずにターミナルを制御する (完成版)。
//!
//! - `ansi`: ANSI エスケープシーケンスを組み立てる小さな関数群 (純粋)
//! - `key`: 入力バイト列 → `Key` のパーサ (純粋・テストあり)
//! - `app`: デモアプリの状態と更新・描画ロジック (純粋・テストあり)
//! - `terminal`: libc の termios / ioctl を使う端末依存部分

pub mod ansi;
pub mod app;
pub mod key;
pub mod terminal;
