//! kilo 風テキストエディタのコアロジック。
//!
//! - `width`  : 文字の表示幅 (全角 = 2 列)
//! - `row`    : 1 行分のテキストと、文字位置 ⇔ 画面列の変換
//! - `editor` : カーソル・スクロール・編集・保存・検索・描画内容の生成
//!
//! 端末の操作 (raw モード・実際の出力) は `main.rs` だけが行う。

pub mod editor;
pub mod row;
pub mod width;

pub use editor::{Editor, Key, Screen};
