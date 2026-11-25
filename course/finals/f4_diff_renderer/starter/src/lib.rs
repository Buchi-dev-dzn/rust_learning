//! F4: 差分レンダラ。
//!
//! 仕様は SPEC.md。`todo!()` を埋めて `cargo test -p f4_diff_renderer` を通そう。
//! 型と公開メソッドのシグネチャはテストが依存しているので変えないこと。
//! (Buffer / Renderer のフィールドや補助関数は自由に変えてよい)

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Color {
    #[default]
    Reset,
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    White,
    Indexed(u8),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Style {
    pub fg: Color,
    pub bg: Color,
    pub bold: bool,
}

impl Style {
    pub fn new() -> Style {
        Style::default()
    }
    pub fn fg(mut self, c: Color) -> Style {
        self.fg = c;
        self
    }
    pub fn bg(mut self, c: Color) -> Style {
        self.bg = c;
        self
    }
    pub fn bold(mut self) -> Style {
        self.bold = true;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Cell {
    pub ch: char,
    pub style: Style,
}

impl Cell {
    /// 全角文字の右半分を表す印
    pub const CONTINUATION_CHAR: char = '\0';

    pub fn blank() -> Cell {
        Cell { ch: ' ', style: Style::default() }
    }
    pub fn is_continuation(&self) -> bool {
        self.ch == Cell::CONTINUATION_CHAR
    }
}

impl Default for Cell {
    fn default() -> Cell {
        Cell::blank()
    }
}

/// 表示幅 (SPEC 2章)
pub fn char_width(c: char) -> usize {
    // TODO
    todo!()
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buffer {
    width: usize,
    height: usize,
    /// 行優先: (x, y) は cells[y * width + x]
    cells: Vec<Cell>,
}

impl Buffer {
    pub fn new(width: usize, height: usize) -> Buffer {
        // TODO
        todo!()
    }

    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get(&self, x: usize, y: usize) -> Cell {
        // TODO
        todo!()
    }

    /// 1 文字書く。全角文字の片割れの片付けを忘れずに (SPEC 3章)。
    pub fn set(&mut self, x: usize, y: usize, ch: char, style: Style) {
        // TODO
        todo!()
    }

    /// 文字列を書き、次に書く x を返す。
    pub fn set_str(&mut self, x: usize, y: usize, s: &str, style: Style) -> usize {
        // TODO
        todo!()
    }

    pub fn clear(&mut self) {
        // TODO
        todo!()
    }

    pub fn lines(&self) -> Vec<String> {
        // TODO
        todo!()
    }
}

/// スタイル指定のエスケープシーケンス (SPEC 4章)
pub fn sgr(style: &Style) -> String {
    // TODO
    todo!()
}

/// prev → next にするための最小限の出力 (SPEC 5章)
pub fn diff(prev: &Buffer, next: &Buffer) -> String {
    // TODO: カーソル位置 Option<(usize, usize)> と現在のスタイルを追跡する
    todo!()
}

#[derive(Debug, Default)]
pub struct Renderer {
    prev: Option<Buffer>,
}

impl Renderer {
    pub fn new() -> Renderer {
        Renderer { prev: None }
    }

    pub fn render(&mut self, next: &Buffer) -> String {
        // TODO
        todo!()
    }

    pub fn invalidate(&mut self) {
        // TODO
        todo!()
    }
}
