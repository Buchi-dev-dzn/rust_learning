//! kilo 風テキストエディタのコアロジック (starter)。
//!
//! - `width`  : 文字の表示幅 (全角 = 2 列)
//! - `row`    : 1 行分のテキストと、文字位置 ⇔ 画面列の変換
//! - `editor` : カーソル・スクロール・編集・保存・検索・描画内容の生成
//!
//! 端末の操作 (raw モード・実際の出力) は `main.rs` だけが行う。
//! STEPS.md の順に `TODO(STEP n)` を埋めていこう。

// starter では未使用の引数が多いので警告を抑える (完成したら消してよい)
#![allow(unused_variables, unused_imports, dead_code)]

pub mod editor;
pub mod row;
pub mod width;

pub use editor::{Editor, Key, Screen};
