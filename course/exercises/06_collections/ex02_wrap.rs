//! # テキストの折り返し
//!
//! 幅の決まった領域 (ペインやポップアップ) に文章を表示するための、単語単位の折り返しを実装しよう。
//!
//! - ゴール: `split_whitespace` で単語に分け、`String` を組み立てて `Vec<String>` に集める
//! - テスト (`mod tests`) は編集しないこと。
//! - 関連: 第6章「String」「行に分ける・つなぐ」「イテレータの基本」
//!
//! 仕様 (ASCII のみ考えればよい):
//! - 単語は空白で区切る。連続する空白は 1 つとみなす。
//! - 1 行は `width` 文字以下。単語の間は空白 1 つ。
//! - 1 単語が `width` より長い場合は、`width` 文字ずつに切って複数行にする。
//! - 入力が空 (または空白のみ) なら空の Vec。
//! - `width == 0` なら空の Vec。

fn wrap(text: &str, width: usize) -> Vec<String> {
    let mut lines: Vec<String> = Vec::new();
    // TODO: 「今組み立て中の行」を String で持ち、単語を 1 つずつ足していく。
    //       入らなくなったら lines に push して新しい行を始める。
    //       最後に組み立て中の行が空でなければ push するのを忘れずに。
    lines
}

/// 各行を幅 `width` まで右側を空白で埋め、上下左右を枠で囲む。
/// (例: frame(&["ab", "c"], 3) == ["+---+", "|ab |", "|c  |", "+---+"])
fn frame(lines: &[String], width: usize) -> Vec<String> {
    // TODO: format!("|{:<width$}|", line) で左寄せ+空白埋めができる
    //       上下の枠は "-".repeat(width)
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn v(xs: &[&str]) -> Vec<String> {
        xs.iter().map(|s| s.to_string()).collect()
    }

    #[test]
    fn wraps_words() {
        assert_eq!(
            wrap("the quick brown fox jumps", 10),
            v(&["the quick", "brown fox", "jumps"])
        );
        assert_eq!(wrap("a b c", 5), v(&["a b c"]));
        assert_eq!(wrap("a b c", 3), v(&["a b", "c"]));
    }

    #[test]
    fn collapses_spaces() {
        assert_eq!(wrap("  hello    world  ", 20), v(&["hello world"]));
        assert_eq!(wrap("   ", 5), Vec::<String>::new());
        assert_eq!(wrap("", 5), Vec::<String>::new());
    }

    #[test]
    fn splits_long_words() {
        assert_eq!(wrap("abcdefghij", 4), v(&["abcd", "efgh", "ij"]));
        assert_eq!(wrap("hi abcdefgh yo", 4), v(&["hi", "abcd", "efgh", "yo"]));
        assert_eq!(wrap("anything", 0), Vec::<String>::new());
    }

    #[test]
    fn frames() {
        assert_eq!(frame(&v(&["ab", "c"]), 3), v(&["+---+", "|ab |", "|c  |", "+---+"]));
        assert_eq!(frame(&[], 2), v(&["+--+", "+--+"]));
    }

    #[test]
    fn wrap_then_frame() {
        let boxed = frame(&wrap("Save changes before quit?", 12), 12);
        assert_eq!(
            boxed,
            v(&["+------------+", "|Save changes|", "|before quit?|", "+------------+"])
        );
    }
}
