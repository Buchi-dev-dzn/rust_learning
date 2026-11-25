//! # Display と Default を実装する
//!
//! `TODO` の部分を直して、すべてのテストを通そう。テストは編集しないこと。
//!
//! - `Size` に `Display` を実装し、`80x24` のように表示できるようにする
//! - `Color` に `Display` を実装し、色名を小文字で表示する (`Reset` は `reset`)
//! - `Style` に `Default` を実装する (fg は `Color::Reset`、bold は `false`)
//! - `Describe` トレイトの `describe` にデフォルト実装を書く
//! - 関連: 第9章「標準ライブラリのトレイトを実装する」

use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Size {
    pub w: u16,
    pub h: u16,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Color {
    Reset,
    Red,
    Green,
    Blue,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Style {
    pub fg: Color,
    pub bold: bool,
}

// write! で Formatter に書き込む
impl fmt::Display for Size {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}x{}", self.w, self.h)
    }
}

impl fmt::Display for Color {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let name = match self {
            Color::Reset => "reset",
            Color::Red => "red",
            Color::Green => "green",
            Color::Blue => "blue",
        };
        // write_str でも write! でもよい
        f.write_str(name)
    }
}

impl Default for Style {
    fn default() -> Self {
        Style { fg: Color::Reset, bold: false }
    }
}

pub trait Describe {
    fn name(&self) -> String;

    // デフォルト実装は他のメソッド (name) を呼べる
    fn describe(&self) -> String {
        format!("<{}>", self.name())
    }
}

impl Describe for Size {
    fn name(&self) -> String {
        self.to_string()
    }
}

impl Describe for Color {
    fn name(&self) -> String {
        self.to_string()
    }
    fn describe(&self) -> String {
        format!("color:{}", self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn size_display() {
        assert_eq!(Size { w: 80, h: 24 }.to_string(), "80x24");
        assert_eq!(format!("[{}]", Size { w: 1, h: 2 }), "[1x2]");
    }

    #[test]
    fn color_display() {
        assert_eq!(Color::Reset.to_string(), "reset");
        assert_eq!(Color::Red.to_string(), "red");
        assert_eq!(format!("{}/{}", Color::Green, Color::Blue), "green/blue");
    }

    #[test]
    fn style_default() {
        let s = Style::default();
        assert_eq!(s, Style { fg: Color::Reset, bold: false });
        let t = Style { bold: true, ..Default::default() };
        assert!(t.bold);
    }

    #[test]
    fn describe_default_and_override() {
        assert_eq!(Size { w: 3, h: 4 }.describe(), "<3x4>");
        assert_eq!(Color::Red.describe(), "color:red");
    }
}
