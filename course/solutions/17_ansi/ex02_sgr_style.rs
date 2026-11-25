//! # SGR で色と装飾を組み立てる
//!
//! `Style` (前景色・背景色・太字・下線・反転) から、1 つの SGR シーケンス
//! `ESC [ 引数;引数;... m` を作る `sgr` 関数を実装しよう。
//!
//! 引数の並び順 (この順に `;` でつなぐ):
//! 1. `0` (常に先頭でリセットし、前の状態を引き継がないようにする)
//! 2. 太字 `1` / 下線 `4` / 反転 `7` (true のものだけ、この順)
//! 3. 前景色 (Some のとき) 4. 背景色 (Some のとき)
//!
//! 色の引数 (`fg_params` / `bg_params`):
//!
//! | Color | 前景 | 背景 |
//! |---|---|---|
//! | `Basic(n)` (n = 0〜7) | `30+n` | `40+n` |
//! | `Bright(n)` (n = 0〜7) | `90+n` | `100+n` |
//! | `Indexed(n)` | `38;5;n` | `48;5;n` |
//! | `Rgb(r,g,b)` | `38;2;r;g;b` | `48;2;r;g;b` |
//!
//! 例: 太字 + 前景 Rgb(255,0,0) → `"\x1b[0;1;38;2;255;0;0m"`、何も指定なし → `"\x1b[0m"`
//!
//! `paint(text, style)` は `sgr(style) + text + "\x1b[0m"` を返す。
//!
//! - 関連: 第17章「色と装飾: SGR」
//!
//! テスト (`mod tests`) は編集しないこと。

#[derive(Clone, Copy, Debug, PartialEq)]
enum Color {
    Basic(u8),
    Bright(u8),
    Indexed(u8),
    Rgb(u8, u8, u8),
}

#[derive(Clone, Copy, Debug, Default, PartialEq)]
struct Style {
    fg: Option<Color>,
    bg: Option<Color>,
    bold: bool,
    underline: bool,
    reverse: bool,
}

fn fg_params(c: Color) -> String {
    match c {
        Color::Basic(n) => format!("{}", 30 + n),
        Color::Bright(n) => format!("{}", 90 + n),
        Color::Indexed(n) => format!("38;5;{n}"),
        Color::Rgb(r, g, b) => format!("38;2;{r};{g};{b}"),
    }
}

fn bg_params(c: Color) -> String {
    match c {
        Color::Basic(n) => format!("{}", 40 + n),
        // 100 + n は u8 で最大 107 なので溢れない
        Color::Bright(n) => format!("{}", 100 + n),
        Color::Indexed(n) => format!("48;5;{n}"),
        Color::Rgb(r, g, b) => format!("48;2;{r};{g};{b}"),
    }
}

fn sgr(style: &Style) -> String {
    // 引数を Vec に集めて join(";") するのが一番簡単
    let mut params = vec!["0".to_string()];
    if style.bold {
        params.push("1".into());
    }
    if style.underline {
        params.push("4".into());
    }
    if style.reverse {
        params.push("7".into());
    }
    if let Some(c) = style.fg {
        params.push(fg_params(c));
    }
    if let Some(c) = style.bg {
        params.push(bg_params(c));
    }
    format!("\x1b[{}m", params.join(";"))
}

fn paint(text: &str, style: &Style) -> String {
    format!("{}{text}\x1b[0m", sgr(style))
}

fn main() {
    let title = Style { fg: Some(Color::Bright(3)), bold: true, ..Default::default() };
    let sel = Style { fg: Some(Color::Basic(0)), bg: Some(Color::Rgb(135, 206, 250)), ..Default::default() };
    println!("{}", paint("タイトル", &title));
    println!("{}", paint(" 選択中の行 ", &sel));
    println!("{:?}", sgr(&sel));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_params() {
        assert_eq!(fg_params(Color::Basic(1)), "31");
        assert_eq!(bg_params(Color::Basic(4)), "44");
        assert_eq!(fg_params(Color::Bright(2)), "92");
        assert_eq!(bg_params(Color::Bright(7)), "107");
        assert_eq!(fg_params(Color::Indexed(208)), "38;5;208");
        assert_eq!(bg_params(Color::Indexed(0)), "48;5;0");
        assert_eq!(fg_params(Color::Rgb(1, 2, 3)), "38;2;1;2;3");
        assert_eq!(bg_params(Color::Rgb(255, 255, 255)), "48;2;255;255;255");
    }

    #[test]
    fn plain_style() {
        assert_eq!(sgr(&Style::default()), "\x1b[0m");
    }

    #[test]
    fn attributes_order() {
        let s = Style { bold: true, underline: true, reverse: true, ..Default::default() };
        assert_eq!(sgr(&s), "\x1b[0;1;4;7m");
        let s = Style { reverse: true, ..Default::default() };
        assert_eq!(sgr(&s), "\x1b[0;7m");
    }

    #[test]
    fn colors_and_attrs() {
        let s = Style { fg: Some(Color::Rgb(255, 0, 0)), bold: true, ..Default::default() };
        assert_eq!(sgr(&s), "\x1b[0;1;38;2;255;0;0m");
        let s = Style {
            fg: Some(Color::Basic(7)),
            bg: Some(Color::Indexed(236)),
            underline: true,
            ..Default::default()
        };
        assert_eq!(sgr(&s), "\x1b[0;4;37;48;5;236m");
    }

    #[test]
    fn paint_wraps_text() {
        let s = Style { fg: Some(Color::Basic(2)), ..Default::default() };
        assert_eq!(paint("ok", &s), "\x1b[0;32mok\x1b[0m");
    }
}
