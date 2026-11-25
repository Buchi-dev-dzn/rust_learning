//! F4: 差分レンダラの模範解答。
//!
//! 画面を「セルの 2 次元配列」として持ち、前フレームとの差分だけを ANSI で出力する。
//! ratatui の Buffer / Terminal::flush がやっていることの縮小版。

use std::fmt::Write as _;

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

pub fn char_width(c: char) -> usize {
    let u = c as u32;
    if u < 0x20 || (0x7F..0xA0).contains(&u) || (0x0300..=0x036F).contains(&u) || u == 0x200B {
        return 0;
    }
    let wide = matches!(u,
        0x1100..=0x115F
        | 0x2E80..=0x303E
        | 0x3041..=0x33FF
        | 0x3400..=0x4DBF
        | 0x4E00..=0x9FFF
        | 0xA000..=0xA4CF
        | 0xAC00..=0xD7A3
        | 0xF900..=0xFAFF
        | 0xFE30..=0xFE4F
        | 0xFF00..=0xFF60
        | 0xFFE0..=0xFFE6
        | 0x1F300..=0x1F64F
        | 0x1F900..=0x1F9FF
        | 0x20000..=0x3FFFD);
    if wide { 2 } else { 1 }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Buffer {
    width: usize,
    height: usize,
    cells: Vec<Cell>,
}

impl Buffer {
    pub fn new(width: usize, height: usize) -> Buffer {
        Buffer {
            width,
            height,
            cells: vec![Cell::blank(); width * height],
        }
    }

    pub fn width(&self) -> usize {
        self.width
    }
    pub fn height(&self) -> usize {
        self.height
    }

    pub fn get(&self, x: usize, y: usize) -> Cell {
        assert!(x < self.width && y < self.height, "out of bounds");
        self.cells[y * self.width + x]
    }

    fn idx(&self, x: usize, y: usize) -> usize {
        y * self.width + x
    }

    fn in_bounds(&self, x: usize, y: usize) -> bool {
        x < self.width && y < self.height
    }

    /// (x, y) のセルを壊す前に、そこにかかっている全角文字の片割れを空白に戻す
    fn break_wide_at(&mut self, x: usize, y: usize) {
        let i = self.idx(x, y);
        if self.cells[i].is_continuation() && x > 0 {
            let left = i - 1;
            self.cells[left].ch = ' ';
        }
        if x + 1 < self.width && self.cells[i + 1].is_continuation() {
            self.cells[i + 1].ch = ' ';
        }
    }

    pub fn set(&mut self, x: usize, y: usize, ch: char, style: Style) {
        if !self.in_bounds(x, y) {
            return;
        }
        let w = char_width(ch);
        if w == 0 {
            return;
        }
        self.break_wide_at(x, y);
        let i = self.idx(x, y);
        if w == 2 {
            if x + 1 >= self.width {
                // 右端に全角は入らない
                self.cells[i] = Cell { ch: ' ', style };
                return;
            }
            self.break_wide_at(x + 1, y);
            self.cells[i] = Cell { ch, style };
            self.cells[i + 1] = Cell { ch: Cell::CONTINUATION_CHAR, style };
        } else {
            self.cells[i] = Cell { ch, style };
        }
    }

    pub fn set_str(&mut self, x: usize, y: usize, s: &str, style: Style) -> usize {
        let mut cx = x;
        for ch in s.chars() {
            let w = char_width(ch);
            if w == 0 {
                continue;
            }
            if cx >= self.width {
                break;
            }
            self.set(cx, y, ch, style);
            cx += w;
        }
        cx.min(self.width)
    }

    pub fn clear(&mut self) {
        self.cells.fill(Cell::blank());
    }

    pub fn lines(&self) -> Vec<String> {
        (0..self.height)
            .map(|y| {
                self.cells[y * self.width..(y + 1) * self.width]
                    .iter()
                    .filter(|c| !c.is_continuation())
                    .map(|c| c.ch)
                    .collect()
            })
            .collect()
    }
}

fn push_color(out: &mut String, c: Color, base: u8) {
    match c {
        Color::Reset => {}
        Color::Indexed(n) => {
            let _ = write!(out, ";{};5;{}", base + 8, n);
        }
        named => {
            let k = match named {
                Color::Black => 0,
                Color::Red => 1,
                Color::Green => 2,
                Color::Yellow => 3,
                Color::Blue => 4,
                Color::Magenta => 5,
                Color::Cyan => 6,
                _ => 7, // White
            };
            let _ = write!(out, ";{}", base + k);
        }
    }
}

pub fn sgr(style: &Style) -> String {
    let mut s = String::from("\x1b[0");
    if style.bold {
        s.push_str(";1");
    }
    push_color(&mut s, style.fg, 30);
    push_color(&mut s, style.bg, 40);
    s.push('m');
    s
}

pub fn diff(prev: &Buffer, next: &Buffer) -> String {
    let mut out = String::new();
    let blank;
    let prev = if prev.width != next.width || prev.height != next.height {
        out.push_str("\x1b[2J");
        blank = Buffer::new(next.width, next.height);
        &blank
    } else {
        prev
    };
    let mut cursor: Option<(usize, usize)> = None;
    let mut current = Style::default();
    for y in 0..next.height {
        for x in 0..next.width {
            let n = next.get(x, y);
            if n.is_continuation() || n == prev.get(x, y) {
                continue;
            }
            if cursor != Some((x, y)) {
                let _ = write!(out, "\x1b[{};{}H", y + 1, x + 1);
            }
            if n.style != current {
                out.push_str(&sgr(&n.style));
                current = n.style;
            }
            out.push(n.ch);
            let nx = x + char_width(n.ch);
            // 右端まで書いた後のカーソル位置は端末によって違うので「不明」にする
            cursor = if nx >= next.width { None } else { Some((nx, y)) };
        }
    }
    if current != Style::default() {
        out.push_str("\x1b[0m");
    }
    out
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
        let out = match &self.prev {
            Some(p) => diff(p, next),
            None => diff(&Buffer::new(0, 0), next),
        };
        self.prev = Some(next.clone());
        out
    }

    pub fn invalidate(&mut self) {
        self.prev = None;
    }
}
