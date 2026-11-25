//! 自作のミニ TUI フレームワーク (依存は libc のみ)。
//!
//! 下の層から順に:
//! - `width`    : 文字の表示幅
//! - `geometry` : Rect (長方形の領域)
//! - `style`    : Color / Style と SGR シーケンス
//! - `buffer`   : Cell / Buffer (画面 1 枚ぶんのセル) と差分 (diff)
//! - `render`   : 差分 → 最小限の ANSI 出力
//! - `layout`   : Layout / Constraint による領域の分割
//! - `widgets`  : Widget トレイトと Block / Paragraph / List / Tabs / Gauge / Clear
//! - `input`    : バイト列 → Key
//! - `terminal` : raw モード (termios)、端末サイズ、描画、poll による入力待ち
//! - `demo`     : このフレームワークで作ったデモアプリ

pub mod buffer;
pub mod demo;
pub mod geometry;
pub mod input;
pub mod layout;
pub mod render;
pub mod style;
pub mod terminal;
pub mod widgets;
pub mod width;

pub use buffer::{Buffer, Cell};
pub use geometry::Rect;
pub use input::Key;
pub use layout::{Constraint, Direction, Layout};
pub use style::{Color, Style};
pub use widgets::Widget;
