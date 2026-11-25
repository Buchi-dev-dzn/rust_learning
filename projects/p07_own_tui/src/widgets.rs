//! ウィジェット: 「領域 (Rect) を受け取ってバッファに描く」もの。
//!
//! ratatui と同じく、ウィジェットは描画のたびに作り直す使い捨ての値。
//! 状態 (選択位置など) はアプリ側が持ち、ウィジェットを作るときに渡す。

use crate::buffer::Buffer;
use crate::geometry::Rect;
use crate::style::Style;
use crate::width::{char_width, str_width};

pub trait Widget {
    fn render(&self, area: Rect, buf: &mut Buffer);
}

// ------------------------------------------------------------------ Clear

/// 領域を空白で消す。ポップアップを重ねる前に使う
pub struct Clear;

impl Widget for Clear {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let area = buf.area.intersection(area);
        for y in area.y..area.bottom() {
            buf.set_stringn(area.x, y, &" ".repeat(area.width as usize), area.width as usize, Style::new());
        }
    }
}

// ------------------------------------------------------------------ Block

/// 枠線とタイトル。他のウィジェットを囲むのに使う
#[derive(Debug, Clone, Default)]
pub struct Block {
    pub title: Option<String>,
    pub borders: bool,
    pub style: Style,
    pub border_style: Style,
}

impl Block {
    pub fn new() -> Self {
        Block::default()
    }
    pub fn bordered() -> Self {
        Block { borders: true, ..Block::default() }
    }
    pub fn title(mut self, t: impl Into<String>) -> Self {
        self.title = Some(t.into());
        self
    }
    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }
    pub fn border_style(mut self, s: Style) -> Self {
        self.border_style = s;
        self
    }

    /// 枠の内側の領域
    pub fn inner(&self, area: Rect) -> Rect {
        if self.borders { area.inner(1) } else { area }
    }
}

impl Widget for Block {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let area = buf.area.intersection(area);
        if area.is_empty() {
            return;
        }
        buf.set_style(area, self.style);
        if self.borders && area.width >= 2 && area.height >= 2 {
            let s = self.style.patch(self.border_style);
            let (l, r, t, b) = (area.x, area.right() - 1, area.y, area.bottom() - 1);
            for x in l + 1..r {
                buf.set_string(x, t, "─", s);
                buf.set_string(x, b, "─", s);
            }
            for y in t + 1..b {
                buf.set_string(l, y, "│", s);
                buf.set_string(r, y, "│", s);
            }
            buf.set_string(l, t, "┌", s);
            buf.set_string(r, t, "┐", s);
            buf.set_string(l, b, "└", s);
            buf.set_string(r, b, "┘", s);
        }
        if let Some(title) = &self.title {
            let (x, max) = if self.borders {
                (area.x + 1, area.width.saturating_sub(2) as usize)
            } else {
                (area.x, area.width as usize)
            };
            buf.set_stringn(x, area.y, title, max, self.style.patch(self.border_style));
        }
    }
}

// ------------------------------------------------------------------ Paragraph

/// 複数行のテキスト。折り返しとスクロールに対応する
#[derive(Debug, Clone, Default)]
pub struct Paragraph {
    pub lines: Vec<(String, Style)>,
    pub block: Option<Block>,
    pub wrap: bool,
    pub scroll: u16,
}

impl Paragraph {
    pub fn new(text: &str) -> Self {
        Paragraph {
            lines: text.lines().map(|l| (l.to_string(), Style::new())).collect(),
            ..Paragraph::default()
        }
    }
    /// 行ごとにスタイルを指定して作る
    pub fn styled_lines(lines: Vec<(String, Style)>) -> Self {
        Paragraph { lines, ..Paragraph::default() }
    }
    pub fn block(mut self, b: Block) -> Self {
        self.block = Some(b);
        self
    }
    pub fn wrap(mut self, on: bool) -> Self {
        self.wrap = on;
        self
    }
    pub fn scroll(mut self, n: u16) -> Self {
        self.scroll = n;
        self
    }
}

/// 表示幅 `width` ごとに文字列を折り返す (全角文字の途中では切らない)
pub fn wrap_line(line: &str, width: usize) -> Vec<String> {
    if width == 0 {
        return Vec::new();
    }
    let mut out = vec![String::new()];
    let mut w = 0;
    for ch in line.chars() {
        let cw = char_width(ch);
        if w + cw > width {
            out.push(String::new());
            w = 0;
        }
        out.last_mut().unwrap().push(ch);
        w += cw;
    }
    out
}

impl Widget for Paragraph {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let inner = match &self.block {
            Some(b) => {
                b.render(area, buf);
                b.inner(area)
            }
            None => area,
        };
        let inner = buf.area.intersection(inner);
        let width = inner.width as usize;
        let visual: Vec<(String, Style)> = if self.wrap {
            self.lines
                .iter()
                .flat_map(|(l, s)| wrap_line(l, width).into_iter().map(move |w| (w, *s)))
                .collect()
        } else {
            self.lines.clone()
        };
        for (i, (line, style)) in visual
            .iter()
            .skip(self.scroll as usize)
            .take(inner.height as usize)
            .enumerate()
        {
            buf.set_stringn(inner.x, inner.y + i as u16, line, width, *style);
        }
    }
}

// ------------------------------------------------------------------ List

#[derive(Debug, Clone)]
pub struct ListItem {
    pub text: String,
    pub style: Style,
}

impl ListItem {
    pub fn new(text: impl Into<String>) -> Self {
        ListItem { text: text.into(), style: Style::new() }
    }
    pub fn style(mut self, s: Style) -> Self {
        self.style = s;
        self
    }
}

/// 選択できるリスト。選択行が見えるように自動でスクロールする
#[derive(Debug, Clone, Default)]
pub struct List {
    pub items: Vec<ListItem>,
    pub selected: Option<usize>,
    pub highlight_style: Style,
    pub highlight_symbol: String,
    pub block: Option<Block>,
}

impl List {
    pub fn new(items: Vec<ListItem>) -> Self {
        List { items, ..List::default() }
    }
    pub fn selected(mut self, i: Option<usize>) -> Self {
        self.selected = i;
        self
    }
    pub fn highlight_style(mut self, s: Style) -> Self {
        self.highlight_style = s;
        self
    }
    pub fn highlight_symbol(mut self, s: impl Into<String>) -> Self {
        self.highlight_symbol = s.into();
        self
    }
    pub fn block(mut self, b: Block) -> Self {
        self.block = Some(b);
        self
    }

    /// 選択行が見える最小のスクロール量
    pub fn offset(&self, height: usize) -> usize {
        match self.selected {
            Some(sel) if height > 0 && sel >= height => sel + 1 - height,
            _ => 0,
        }
    }
}

impl Widget for List {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let inner = match &self.block {
            Some(b) => {
                b.render(area, buf);
                b.inner(area)
            }
            None => area,
        };
        let inner = buf.area.intersection(inner);
        if inner.is_empty() {
            return;
        }
        let height = inner.height as usize;
        let offset = self.offset(height);
        let sym_w = str_width(&self.highlight_symbol);
        let blank = " ".repeat(sym_w);
        for (row, (i, item)) in self.items.iter().enumerate().skip(offset).take(height).enumerate() {
            let y = inner.y + row as u16;
            let is_sel = self.selected == Some(i);
            let line_area = Rect::new(inner.x, y, inner.width, 1);
            let mut style = item.style;
            if is_sel {
                style = style.patch(self.highlight_style);
                buf.set_style(line_area, self.highlight_style); // 行全体を塗る
            }
            let sym = if is_sel { &self.highlight_symbol } else { &blank };
            let x = buf.set_stringn(inner.x, y, sym, inner.width as usize, style);
            let rest = (inner.right() - x) as usize;
            buf.set_stringn(x, y, &item.text, rest, style);
        }
    }
}

// ------------------------------------------------------------------ Tabs

/// タブの見出し行: " A │ B │ C "
#[derive(Debug, Clone, Default)]
pub struct Tabs {
    pub titles: Vec<String>,
    pub selected: usize,
    pub style: Style,
    pub highlight_style: Style,
}

impl Tabs {
    pub fn new(titles: Vec<String>) -> Self {
        Tabs { titles, ..Tabs::default() }
    }
    pub fn selected(mut self, i: usize) -> Self {
        self.selected = i;
        self
    }
    pub fn highlight_style(mut self, s: Style) -> Self {
        self.highlight_style = s;
        self
    }
}

impl Widget for Tabs {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let area = buf.area.intersection(area);
        if area.is_empty() {
            return;
        }
        let mut x = area.x;
        for (i, title) in self.titles.iter().enumerate() {
            if i > 0 {
                x = buf.set_stringn(x, area.y, "│", (area.right() - x) as usize, self.style);
            }
            let style = if i == self.selected {
                self.style.patch(self.highlight_style)
            } else {
                self.style
            };
            x = buf.set_stringn(x, area.y, &format!(" {title} "), (area.right() - x) as usize, style);
        }
    }
}

// ------------------------------------------------------------------ Gauge

/// 進捗バー
#[derive(Debug, Clone, Default)]
pub struct Gauge {
    pub ratio: f64,
    pub label: String,
    pub filled_style: Style,
}

impl Gauge {
    pub fn new(ratio: f64) -> Self {
        Gauge { ratio: ratio.clamp(0.0, 1.0), ..Gauge::default() }
    }
    pub fn label(mut self, l: impl Into<String>) -> Self {
        self.label = l.into();
        self
    }
    pub fn filled_style(mut self, s: Style) -> Self {
        self.filled_style = s;
        self
    }
}

impl Widget for Gauge {
    fn render(&self, area: Rect, buf: &mut Buffer) {
        let area = buf.area.intersection(area);
        if area.is_empty() {
            return;
        }
        let filled = (area.width as f64 * self.ratio).round() as u16;
        buf.set_style(Rect::new(area.x, area.y, filled, area.height), self.filled_style);
        let lw = str_width(&self.label) as u16;
        let x = area.x + area.width.saturating_sub(lw) / 2;
        let y = area.y + area.height / 2;
        // ラベルは今のセルのスタイル (塗られた背景) を保ったまま書く
        let mut cx = x;
        for ch in self.label.chars() {
            if cx >= area.right() {
                break;
            }
            let style = buf.get(cx, y).style;
            cx = buf.set_stringn(cx, y, &ch.to_string(), (area.right() - cx) as usize, style);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::style::Color;

    fn lines(buf: &Buffer) -> Vec<String> {
        (buf.area.y..buf.area.bottom()).map(|y| buf.line(y)).collect()
    }

    #[test]
    fn block_draws_border_and_title() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 8, 3));
        Block::bordered().title("題名").render(buf.area, &mut buf);
        assert_eq!(lines(&buf), ["┌題名──┐", "│      │", "└──────┘"]);
        assert_eq!(Block::bordered().inner(buf.area), Rect::new(1, 1, 6, 1));
    }

    #[test]
    fn paragraph_clips_and_scrolls() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 4, 2));
        Paragraph::new("abcdef\nxy\nzz").scroll(1).render(buf.area, &mut buf);
        assert_eq!(lines(&buf), ["xy  ", "zz  "]);
    }

    #[test]
    fn paragraph_wraps_wide_chars() {
        assert_eq!(wrap_line("あいうabc", 5), ["あい", "うabc"]);
        let mut buf = Buffer::empty(Rect::new(0, 0, 5, 3));
        Paragraph::new("あいうabc").wrap(true).render(buf.area, &mut buf);
        assert_eq!(lines(&buf), ["あい ", "うabc", "     "]);
    }

    #[test]
    fn list_highlights_and_scrolls_to_selection() {
        let items: Vec<_> = ["a", "b", "c", "d"].iter().map(|s| ListItem::new(*s)).collect();
        let hl = Style::new().bg(Color::Blue);
        let list = List::new(items).selected(Some(3)).highlight_symbol("> ").highlight_style(hl);
        let mut buf = Buffer::empty(Rect::new(0, 0, 5, 2));
        list.render(buf.area, &mut buf);
        assert_eq!(lines(&buf), ["  c  ", "> d  "]);
        assert_eq!(buf.get(4, 1).style.bg, Some(Color::Blue)); // 行全体が塗られる
        assert_eq!(buf.get(4, 0).style.bg, None);
    }

    #[test]
    fn list_inside_block() {
        let items = vec![ListItem::new("タスク")];
        let list = List::new(items).selected(Some(0)).highlight_symbol(">").block(Block::bordered());
        let mut buf = Buffer::empty(Rect::new(0, 0, 9, 3));
        list.render(buf.area, &mut buf);
        assert_eq!(buf.line(1), "│>タスク│");
    }

    #[test]
    fn tabs_render() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 12, 1));
        let hl = Style::new().reversed();
        Tabs::new(vec!["A".into(), "B".into()]).selected(1).highlight_style(hl).render(buf.area, &mut buf);
        assert_eq!(buf.line(0), " A │ B      ");
        assert!(buf.get(5, 0).style.reversed);
        assert!(!buf.get(1, 0).style.reversed);
    }

    #[test]
    fn gauge_fills_ratio() {
        let mut buf = Buffer::empty(Rect::new(0, 0, 10, 1));
        let filled = Style::new().bg(Color::Green);
        Gauge::new(0.5).label("50%").filled_style(filled).render(buf.area, &mut buf);
        assert_eq!(buf.line(0), "   50%    ");
        assert_eq!(buf.get(4, 0).style.bg, Some(Color::Green));
        assert_eq!(buf.get(5, 0).style.bg, None);
    }
}
