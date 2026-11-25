//! 色とスタイル、そして ANSI の SGR (Select Graphic Rendition) シーケンスへの変換。

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Color {
    Black,
    Red,
    Green,
    Yellow,
    Blue,
    Magenta,
    Cyan,
    Gray,
    DarkGray,
    White,
    /// 256 色パレットの番号
    Indexed(u8),
    /// 24 ビットカラー
    Rgb(u8, u8, u8),
}

impl Color {
    /// 前景色の SGR パラメータ。背景色は `bg = true` で 10 を足した番号になる
    fn sgr(self, bg: bool) -> String {
        let base = if bg { 10 } else { 0 };
        match self {
            Color::Black => (30 + base).to_string(),
            Color::Red => (31 + base).to_string(),
            Color::Green => (32 + base).to_string(),
            Color::Yellow => (33 + base).to_string(),
            Color::Blue => (34 + base).to_string(),
            Color::Magenta => (35 + base).to_string(),
            Color::Cyan => (36 + base).to_string(),
            Color::Gray => (37 + base).to_string(),
            Color::DarkGray => (90 + base).to_string(),
            Color::White => (97 + base).to_string(),
            Color::Indexed(n) => format!("{};5;{n}", 38 + base),
            Color::Rgb(r, g, b) => format!("{};2;{r};{g};{b}", 38 + base),
        }
    }
}

/// セル 1 つの見た目。`None` は「端末の既定の色」
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct Style {
    pub fg: Option<Color>,
    pub bg: Option<Color>,
    pub bold: bool,
    pub reversed: bool,
}

impl Style {
    pub const fn new() -> Self {
        Style { fg: None, bg: None, bold: false, reversed: false }
    }
    pub const fn fg(mut self, c: Color) -> Self {
        self.fg = Some(c);
        self
    }
    pub const fn bg(mut self, c: Color) -> Self {
        self.bg = Some(c);
        self
    }
    pub const fn bold(mut self) -> Self {
        self.bold = true;
        self
    }
    pub const fn reversed(mut self) -> Self {
        self.reversed = true;
        self
    }

    /// `other` で指定された項目だけを上書きする (重ね塗り)
    pub fn patch(self, other: Style) -> Style {
        Style {
            fg: other.fg.or(self.fg),
            bg: other.bg.or(self.bg),
            bold: self.bold || other.bold,
            reversed: self.reversed || other.reversed,
        }
    }

    /// このスタイルにする SGR シーケンス。毎回 0 (リセット) から組み立てるので、
    /// 直前のスタイルが何であっても正しくなる
    pub fn to_sgr(self) -> String {
        let mut params = vec!["0".to_string()];
        if self.bold {
            params.push("1".into());
        }
        if self.reversed {
            params.push("7".into());
        }
        if let Some(fg) = self.fg {
            params.push(fg.sgr(false));
        }
        if let Some(bg) = self.bg {
            params.push(bg.sgr(true));
        }
        format!("\x1b[{}m", params.join(";"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sgr_sequences() {
        assert_eq!(Style::new().to_sgr(), "\x1b[0m");
        assert_eq!(Style::new().fg(Color::Red).bold().to_sgr(), "\x1b[0;1;31m");
        assert_eq!(Style::new().bg(Color::Blue).to_sgr(), "\x1b[0;44m");
        assert_eq!(Style::new().fg(Color::Indexed(208)).to_sgr(), "\x1b[0;38;5;208m");
        assert_eq!(Style::new().bg(Color::Rgb(1, 2, 3)).to_sgr(), "\x1b[0;48;2;1;2;3m");
    }

    #[test]
    fn patch_overrides_only_given_fields() {
        let base = Style::new().fg(Color::Red).bg(Color::Black);
        let s = base.patch(Style::new().bg(Color::Blue).bold());
        assert_eq!(s, Style { fg: Some(Color::Red), bg: Some(Color::Blue), bold: true, reversed: false });
    }
}
