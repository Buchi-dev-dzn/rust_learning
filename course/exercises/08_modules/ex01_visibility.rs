//! # 公開範囲 (pub) のエラーを直す
//!
//! このファイルは **コンパイルが通りません** (E0603 / E0616 など: private な item へのアクセス)。
//! テストはクレートのルート直下の `tests` モジュールにあり、`terminal` モジュールの **外** から使っている。
//!
//! - ゴール: 「何を公開し、何を隠すか」を決めて `pub` を付ける
//! - テスト (`mod tests`) は変更しないこと。`terminal` モジュールの中に `pub` を足して直す。
//! - ただし `Screen` の `cells` フィールドは **非公開のまま** にすること
//!   (外からは `put` と `row_text` 経由でしか触れないようにして、範囲外書き込みを防ぐため)。
//! - 関連: 第8章「mod と pub」「非公開フィールドで不変条件を守る」

mod terminal {
    // TODO: 必要なところに pub を付ける
    mod style {
        #[derive(Debug, Clone, Copy, PartialEq)]
        enum Color {
            Default,
            Red,
        }

        fn sgr(c: Color) -> &'static str {
            match c {
                Color::Default => "\x1b[39m",
                Color::Red => "\x1b[31m",
            }
        }
    }

    struct Screen {
        width: usize,
        height: usize,
        cells: Vec<char>, // これは非公開のまま
    }

    impl Screen {
        fn new(width: usize, height: usize) -> Self {
            Screen { width, height, cells: vec![' '; width * height] }
        }

        /// (x, y) に文字を置く。範囲外なら何もしない
        fn put(&mut self, x: usize, y: usize, ch: char) {
            if x < self.width && y < self.height {
                self.cells[y * self.width + x] = ch;
            }
        }

        fn row_text(&self, y: usize) -> String {
            self.cells[y * self.width..(y + 1) * self.width].iter().collect()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::terminal::style::{sgr, Color};
    use crate::terminal::Screen;

    #[test]
    fn screen_api() {
        let mut s = Screen::new(4, 2);
        s.put(1, 0, 'a');
        s.put(3, 1, 'b');
        s.put(10, 10, 'x'); // 範囲外は無視
        assert_eq!(s.row_text(0), " a  ");
        assert_eq!(s.row_text(1), "   b");
        assert_eq!((s.width, s.height), (4, 2)); // サイズは読めてよい
    }

    #[test]
    fn styles() {
        assert_eq!(sgr(Color::Red), "\x1b[31m");
        assert_eq!(sgr(Color::Default), "\x1b[39m");
    }
}
