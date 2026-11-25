//! # 参照を返す関数とライフタイム注釈
//!
//! `TODO` の部分を直して、すべてのテストを通そう。テストは編集しないこと。
//!
//! - `longer`: 長い方の `&str` を返す (同じ長さなら `a`)。ライフタイム注釈が必要
//! - `strip_comment`: `#` 以降を取り除き、末尾の空白も取り除いた部分を返す
//! - `nth_line`: n 行目 (0 始まり) を返す。なければ `None`
//! - `after_prefix`: `text` が `prefix` で始まるなら残りを返す。戻り値は `text` だけに依存する
//! - すべて **コピーせず** 元の文字列のスライスを返すこと
//! - 関連: 第10章「関数のライフタイム注釈」

// TODO: コンパイルエラー (E0106) をライフタイム注釈で直す
pub fn longer(a: &str, b: &str) -> &str {
    if a.len() >= b.len() { a } else { b }
}

// TODO: '#' の位置を find で探し、その手前までのスライスを trim_end() して返す
pub fn strip_comment(line: &str) -> &str {
    todo!()
}

// TODO: text.lines() を使う
pub fn nth_line(text: &str, n: usize) -> Option<&str> {
    todo!()
}

// TODO: 戻り値が text にだけ依存するようにライフタイムを付ける
//       (prefix まで同じライフタイムにするとテスト after_prefix_outlives_prefix が通らない)
pub fn after_prefix(text: &str, prefix: &str) -> Option<&str> {
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn longer_picks_longer() {
        assert_eq!(longer("abc", "de"), "abc");
        assert_eq!(longer("a", "xyz"), "xyz");
        assert_eq!(longer("ab", "cd"), "ab");
    }

    #[test]
    fn longer_with_different_scopes() {
        let outer = String::from("outer value");
        let res;
        {
            let inner = String::from("in");
            res = longer(&outer, &inner).to_string();
        }
        assert_eq!(res, "outer value");
    }

    #[test]
    fn strip_comment_works() {
        assert_eq!(strip_comment("set x = 1  # comment"), "set x = 1");
        assert_eq!(strip_comment("no comment"), "no comment");
        assert_eq!(strip_comment("# only"), "");
        assert_eq!(strip_comment("日本語 #コメント"), "日本語");
    }

    #[test]
    fn strip_comment_returns_a_slice_of_input() {
        let s = String::from("abc # x");
        let r = strip_comment(&s);
        // 同じメモリを指していること (コピーしていない)
        assert_eq!(r.as_ptr(), s.as_ptr());
    }

    #[test]
    fn nth_line_works() {
        let text = "one\ntwo\nthree";
        assert_eq!(nth_line(text, 0), Some("one"));
        assert_eq!(nth_line(text, 2), Some("three"));
        assert_eq!(nth_line(text, 3), None);
    }

    #[test]
    fn after_prefix_outlives_prefix() {
        let cmdline = String::from(":write notes.txt");
        let rest;
        {
            let prefix = String::from(":write ");
            rest = after_prefix(&cmdline, &prefix);
        } // prefix はここで破棄されるが、rest は cmdline だけに依存する
        assert_eq!(rest, Some("notes.txt"));
        assert_eq!(after_prefix("abc", "x"), None);
    }
}
