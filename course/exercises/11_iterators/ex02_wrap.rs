//! # テキストの折り返しと切り詰め
//!
//! TUI の段落表示やステータスバーで必須の処理を書こう。
//! `TODO` の部分を直して、すべてのテストを通そう。テストは編集しないこと。
//!
//! 幅は **文字数** (`chars().count()`) で数える (全角の表示幅は第16章で扱う)。
//!
//! - `wrap(text, width)`: 単語単位で折り返して、各行が `width` 文字以下になるようにする
//!   - 入力の改行は保つ (入力の各行を別々に折り返す)。空白だけの行は `""` になる
//!   - 単語間の空白は 1 つにまとめる
//!   - `width` より長い単語は `width` 文字ずつに分割する
//!   - `width` は 1 以上と仮定してよい
//! - `truncate(s, width)`: `width` 文字を超えるなら、`width - 1` 文字 + `…` にする
//!   (`width == 0` なら空文字列)
//! - 関連: 第11章「例: テキストの折り返し」

pub fn wrap(text: &str, width: usize) -> Vec<String> {
    // TODO: text.lines() の各行について split_whitespace() で単語を流し、
    //       行バッファ current に詰めていく。入りきらなければ out に push する
    todo!()
}

pub fn truncate(s: &str, width: usize) -> String {
    // TODO: chars().count() で長さを測り、chars().take(..).collect() で切り出す
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn simple_wrap() {
        assert_eq!(wrap("the quick brown fox", 10), vec!["the quick", "brown fox"]);
        assert_eq!(wrap("a b c", 100), vec!["a b c"]);
    }

    #[test]
    fn exact_fit() {
        assert_eq!(wrap("abc def", 7), vec!["abc def"]);
        assert_eq!(wrap("abc def", 6), vec!["abc", "def"]);
    }

    #[test]
    fn collapses_spaces() {
        assert_eq!(wrap("  a    b  ", 10), vec!["a b"]);
    }

    #[test]
    fn keeps_newlines() {
        assert_eq!(wrap("a\n\nb c", 10), vec!["a", "", "b c"]);
    }

    #[test]
    fn long_word_is_split() {
        assert_eq!(wrap("abcdefghij kl", 4), vec!["abcd", "efgh", "ij", "kl"]);
        assert_eq!(wrap("x abcdefgh", 4), vec!["x", "abcd", "efgh"]);
        assert_eq!(wrap("abcdef g", 3), vec!["abc", "def", "g"]);
    }

    #[test]
    fn counts_chars_not_bytes() {
        assert_eq!(wrap("日本語 テキスト 表示", 7), vec!["日本語", "テキスト 表示"]);
    }

    #[test]
    fn all_lines_fit() {
        let text = "Rust iterators are lazy and composable, which makes them great for rendering.";
        for w in 1..30 {
            for line in wrap(text, w) {
                assert!(line.chars().count() <= w, "width {w}: {line:?}");
            }
        }
    }

    #[test]
    fn truncate_works() {
        assert_eq!(truncate("hello", 10), "hello");
        assert_eq!(truncate("hello", 5), "hello");
        assert_eq!(truncate("hello world", 6), "hello…");
        assert_eq!(truncate("日本語の文章", 4), "日本語…");
        assert_eq!(truncate("abc", 1), "…");
        assert_eq!(truncate("abc", 0), "");
    }
}
