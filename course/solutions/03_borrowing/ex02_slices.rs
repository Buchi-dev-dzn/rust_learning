//! # 文字列スライス
//!
//! `todo!()` の部分を実装して、すべてのテストを通そう。
//!
//! - ゴール: 引数から借りた `&str` を返す関数を書く。バイト位置と文字の境界を意識する
//! - テスト (`mod tests`) は編集しないこと。
//! - 関連: 第3章「文字列スライス `&str`」「UTF-8 と境界」

/// コマンドライン ":open main.rs" のような文字列を、コマンド名と残りに分ける。
/// - 前後の空白は無視する
/// - 最初の空白より前がコマンド、後ろ (前後の空白を除いたもの) が引数
/// - 空白がなければ引数は ""
/// (例: split_command("  open  main.rs ") == ("open", "main.rs"))
fn split_command(line: &str) -> (&str, &str) {
    let line = line.trim();
    // find はバイト位置を返す。' ' は 1 バイトなので i と i+1 はどちらも文字の境界。
    match line.find(' ') {
        Some(i) => (&line[..i], line[i + 1..].trim()),
        None => (line, ""),
    }
}

/// 先頭から最大 `max_chars` 文字ぶんのスライスを返す (文字単位。バイト単位ではない)。
/// 文字数が足りなければ全体を返す。
/// (例: truncate_chars("こんにちは", 3) == "こんに")
fn truncate_chars(s: &str, max_chars: usize) -> &str {
    // nth(max_chars) は「max_chars + 1 文字目」の (バイト位置, 文字)。
    // その手前で切れば、ちょうど max_chars 文字になる。
    match s.char_indices().nth(max_chars) {
        Some((byte_pos, _)) => &s[..byte_pos],
        None => s,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits() {
        assert_eq!(split_command("open main.rs"), ("open", "main.rs"));
        assert_eq!(split_command("  open   main.rs  "), ("open", "main.rs"));
        assert_eq!(split_command("quit"), ("quit", ""));
        assert_eq!(split_command("   "), ("", ""));
        assert_eq!(split_command("s a b"), ("s", "a b"));
    }

    #[test]
    fn truncates() {
        assert_eq!(truncate_chars("hello", 3), "hel");
        assert_eq!(truncate_chars("hello", 10), "hello");
        assert_eq!(truncate_chars("こんにちは", 3), "こんに");
        assert_eq!(truncate_chars("aあb", 2), "aあ");
        assert_eq!(truncate_chars("abc", 0), "");
    }

    #[test]
    fn borrows_from_input() {
        // 返り値は入力の一部を指している (新しい String を作っていない) ことを確認
        let owned = String::from("write out.txt");
        let (cmd, arg) = split_command(&owned);
        assert!(owned.as_ptr() <= cmd.as_ptr());
        assert!(arg.as_ptr() < owned[owned.len()..].as_ptr());
    }
}
