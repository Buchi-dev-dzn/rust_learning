//! # Color enum を設計する
//!
//! 端末の色を表す `Color` enum を **自分で定義** し、メソッドを実装しよう。
//!
//! - ゴール: データを持つバリアントを含む enum を設計し、`impl` でメソッドを付ける
//! - テスト (`mod tests`) は編集しないこと。
//! - 関連: 第5章「データを持つ enum」「enum にメソッドを付ける」
//!
//! 必要な仕様:
//! - バリアント: `Reset`, `Red`, `Green`, `Blue`, `Indexed(u8)` (256 色), `Rgb(u8, u8, u8)` (True Color)
//! - `Debug, Clone, Copy, PartialEq` を derive する
//! - `fn fg(self) -> String` : 前景色の SGR パラメータ
//!     Reset → "39", Red → "31", Green → "32", Blue → "34",
//!     Indexed(n) → "38;5;n", Rgb(r, g, b) → "38;2;r;g;b"
//! - `fn bg(self) -> String` : 背景色の SGR パラメータ
//!     Reset → "49", Red → "41", Green → "42", Blue → "44",
//!     Indexed(n) → "48;5;n", Rgb(r, g, b) → "48;2;r;g;b"
//! - `fn from_name(name: &str) -> Option<Color>` : "red" / "green" / "blue" / "reset" と、
//!     "#rrggbb" 形式 (16 進 6 桁) を解釈する。大文字小文字は区別しない。解釈できなければ None。
//!
//! ヒント: 16 進 2 桁は `u8::from_str_radix("ff", 16)` で数値にできる (Result を返す)。

#[derive(Debug, Clone, Copy, PartialEq)]
enum Color {
    Reset,
    Red,
    Green,
    Blue,
    Indexed(u8),
    Rgb(u8, u8, u8),
}

impl Color {
    fn fg(self) -> String {
        match self {
            Color::Reset => "39".to_string(),
            Color::Red => "31".to_string(),
            Color::Green => "32".to_string(),
            Color::Blue => "34".to_string(),
            Color::Indexed(n) => format!("38;5;{n}"),
            Color::Rgb(r, g, b) => format!("38;2;{r};{g};{b}"),
        }
    }

    fn bg(self) -> String {
        // 背景色は前景色のコードと規則的に対応している (3x → 4x, 38 → 48)。
        // ここでは素直に全部書く。
        match self {
            Color::Reset => "49".to_string(),
            Color::Red => "41".to_string(),
            Color::Green => "42".to_string(),
            Color::Blue => "44".to_string(),
            Color::Indexed(n) => format!("48;5;{n}"),
            Color::Rgb(r, g, b) => format!("48;2;{r};{g};{b}"),
        }
    }

    fn from_name(name: &str) -> Option<Color> {
        let lower = name.to_ascii_lowercase();
        match lower.as_str() {
            "reset" => Some(Color::Reset),
            "red" => Some(Color::Red),
            "green" => Some(Color::Green),
            "blue" => Some(Color::Blue),
            _ => {
                let hex = lower.strip_prefix('#')?; // ? は第7章。None なら即 None を返す
                // ASCII 以外が混ざると &hex[0..2] が文字の途中で切れてパニックし得るので確認する。
                if hex.len() != 6 || !hex.is_ascii() {
                    return None;
                }
                let r = u8::from_str_radix(&hex[0..2], 16).ok()?;
                let g = u8::from_str_radix(&hex[2..4], 16).ok()?;
                let b = u8::from_str_radix(&hex[4..6], 16).ok()?;
                Some(Color::Rgb(r, g, b))
            }
        }
    }
}

/// 前景色・背景色を設定するエスケープシーケンスを作る (これは完成している)
fn style(fg: Color, bg: Color) -> String {
    format!("\x1b[{};{}m", fg.fg(), bg.bg())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fg_codes() {
        assert_eq!(Color::Reset.fg(), "39");
        assert_eq!(Color::Red.fg(), "31");
        assert_eq!(Color::Green.fg(), "32");
        assert_eq!(Color::Blue.fg(), "34");
        assert_eq!(Color::Indexed(208).fg(), "38;5;208");
        assert_eq!(Color::Rgb(1, 2, 3).fg(), "38;2;1;2;3");
    }

    #[test]
    fn bg_codes() {
        assert_eq!(Color::Reset.bg(), "49");
        assert_eq!(Color::Red.bg(), "41");
        assert_eq!(Color::Blue.bg(), "44");
        assert_eq!(Color::Indexed(0).bg(), "48;5;0");
        assert_eq!(Color::Rgb(255, 0, 10).bg(), "48;2;255;0;10");
    }

    #[test]
    fn parses_names() {
        assert_eq!(Color::from_name("red"), Some(Color::Red));
        assert_eq!(Color::from_name("GREEN"), Some(Color::Green));
        assert_eq!(Color::from_name("Reset"), Some(Color::Reset));
        assert_eq!(Color::from_name("#ff8000"), Some(Color::Rgb(255, 128, 0)));
        assert_eq!(Color::from_name("#FFFFFF"), Some(Color::Rgb(255, 255, 255)));
        assert_eq!(Color::from_name("#ff80"), None);
        assert_eq!(Color::from_name("#gg0000"), None);
        assert_eq!(Color::from_name("purple"), None);
    }

    #[test]
    fn builds_style() {
        assert_eq!(style(Color::Red, Color::Reset), "\x1b[31;49m");
        assert_eq!(style(Color::Indexed(1), Color::Rgb(0, 0, 0)), "\x1b[38;5;1;48;2;0;0;0m");
    }

    #[test]
    fn is_copy() {
        let c = Color::Blue;
        let d = c; // Copy なのでムーブしない
        assert_eq!(c, d);
    }
}
