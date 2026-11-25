//! # 所有権を受け取って返す
//!
//! `todo!()` の部分を実装して、すべてのテストを通そう。
//!
//! - ゴール: 所有権を受け取り、加工して返す関数を書く。`std::mem::take` で中身を抜き取る
//! - テスト (`mod tests`) は編集しないこと。
//! - 関連: 第2章「戻り値で所有権を返す」「std::mem::take と replace」

/// `s` の右側を空白で埋め、文字数 (char の数) が `width` になるようにして返す。
/// 既に `width` 以上なら何もせずそのまま返す。
/// (例: pad_right("ab".to_string(), 4) == "ab  ")
fn pad_right(mut s: String, width: usize) -> String {
    // 受け取った String を再利用するので、新しい確保が最小限で済む。
    let len = s.chars().count();
    for _ in len..width {
        s.push(' ');
    }
    s
}

/// 2 つの String の所有権を受け取り、"left | right" の形で 1 つにして返す。
fn join_status(mut left: String, right: String) -> String {
    left.push_str(" | ");
    left.push_str(&right);
    left // right はここで drop される
}

/// 入力欄 `input` に溜まった文字列を「確定」する。
/// 中身を所有権ごと取り出して返し、`input` 自体は空文字列にする。
fn submit(input: &mut String) -> String {
    // input を String::default() (空文字列) と入れ替え、元の中身を返す。
    std::mem::take(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pads() {
        assert_eq!(pad_right("ab".to_string(), 4), "ab  ");
        assert_eq!(pad_right("あい".to_string(), 3), "あい ");
        assert_eq!(pad_right("hello".to_string(), 3), "hello");
    }

    #[test]
    fn joins() {
        assert_eq!(join_status("NORMAL".to_string(), "1:1".to_string()), "NORMAL | 1:1");
    }

    #[test]
    fn submits() {
        let mut input = String::from(":wq");
        let cmd = submit(&mut input);
        assert_eq!(cmd, ":wq");
        assert_eq!(input, "");
        input.push('x');
        assert_eq!(submit(&mut input), "x");
    }
}
