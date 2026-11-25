//! # エディタの行バッファ
//!
//! テキストエディタの心臓部「行バッファ」を `Vec<String>` で作ろう。
//! `todo!()` の部分を実装して、すべてのテストを通すこと。
//!
//! - ゴール: `Vec` の insert / remove と、`String` の文字単位の操作を組み合わせる
//! - テスト (`mod tests`) は編集しないこと。
//! - 関連: 第6章「Vec<T>」「文字単位の挿入と削除」「行に分ける・つなぐ」
//!
//! カーソル位置は (row, col) で、col は **文字単位** (バイト単位ではない)。
//! 日本語も正しく扱えること。

struct LineBuffer {
    lines: Vec<String>,
}

/// 文字位置 → バイト位置 (末尾以降なら s.len())。これは完成している。
fn byte_index(s: &str, char_idx: usize) -> usize {
    s.char_indices().nth(char_idx).map(|(i, _)| i).unwrap_or(s.len())
}

impl LineBuffer {
    /// テキストから作る。空のテキストなら空行 1 つ ("") を持つ。
    fn from_text(text: &str) -> Self {
        // TODO: text.lines() で分けて Vec<String> にする。空なら vec![String::new()]
        todo!()
    }

    /// 全体を "\n" でつないだ文字列を返す
    fn to_text(&self) -> String {
        // TODO: join が使える
        todo!()
    }

    /// (row, col) に文字を挿入する。新しいカーソル位置 (row, col + 1) を返す。
    fn insert_char(&mut self, row: usize, col: usize, ch: char) -> (usize, usize) {
        // TODO: byte_index でバイト位置に直してから String::insert
        todo!()
    }

    /// Enter キー: (row, col) で行を 2 つに分ける。新しいカーソル位置 (row + 1, 0) を返す。
    fn newline(&mut self, row: usize, col: usize) -> (usize, usize) {
        // TODO: String::split_off(バイト位置) は後半を新しい String として切り離す。
        //       それを Vec::insert(row + 1, ...) で次の行に入れる
        todo!()
    }

    /// Backspace キー: カーソルの左の 1 文字を消し、新しいカーソル位置を返す。
    /// - col > 0 なら同じ行の 1 文字を消す → (row, col - 1)
    /// - col == 0 かつ row > 0 なら、この行を前の行の末尾に連結する
    ///   → (row - 1, 連結前の前の行の文字数)
    /// - (0, 0) なら何もしない → (0, 0)
    fn backspace(&mut self, row: usize, col: usize) -> (usize, usize) {
        // TODO: 行の連結は Vec::remove(row) で取り出して、前の行に push_str
        todo!()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip() {
        assert_eq!(LineBuffer::from_text("a\nb\nc").lines, vec!["a", "b", "c"]);
        assert_eq!(LineBuffer::from_text("").lines, vec![""]);
        assert_eq!(LineBuffer::from_text("a\nb").to_text(), "a\nb");
    }

    #[test]
    fn inserts() {
        let mut b = LineBuffer::from_text("hllo\nあう");
        assert_eq!(b.insert_char(0, 1, 'e'), (0, 2));
        assert_eq!(b.insert_char(1, 1, 'い'), (1, 2));
        assert_eq!(b.insert_char(1, 3, '!'), (1, 4));
        assert_eq!(b.to_text(), "hello\nあいう!");
    }

    #[test]
    fn splits_lines() {
        let mut b = LineBuffer::from_text("こんにちは世界");
        assert_eq!(b.newline(0, 5), (1, 0));
        assert_eq!(b.lines, vec!["こんにちは", "世界"]);
        assert_eq!(b.newline(1, 2), (2, 0));
        assert_eq!(b.lines, vec!["こんにちは", "世界", ""]);
        assert_eq!(b.newline(0, 0), (1, 0));
        assert_eq!(b.lines, vec!["", "こんにちは", "世界", ""]);
    }

    #[test]
    fn backspaces() {
        let mut b = LineBuffer::from_text("ab\nいろは");
        assert_eq!(b.backspace(1, 2), (1, 1));
        assert_eq!(b.lines, vec!["ab", "いは"]);
        assert_eq!(b.backspace(1, 0), (0, 2));
        assert_eq!(b.lines, vec!["abいは"]);
        assert_eq!(b.backspace(0, 0), (0, 0));
        assert_eq!(b.lines, vec!["abいは"]);
        assert_eq!(b.backspace(0, 1), (0, 0));
        assert_eq!(b.to_text(), "bいは");
    }
}
