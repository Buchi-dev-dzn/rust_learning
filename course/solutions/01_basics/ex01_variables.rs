//! # 変数とミュータビリティ
//!
//! このファイルは **コンパイルが通りません**。エラーメッセージを読んで直そう。
//!
//! - ゴール: `mut` とシャドーイングの使い分けを体験する
//! - 直すのは `// TODO:` の付いた関数の中だけ。テスト (`mod tests`) は編集しないこと。
//! - 関連: 第1章「変数とミュータビリティ」「シャドーイング」

/// カーソルを右に `steps` 回動かした後の列を返す。
fn move_right(start: u16, steps: u16) -> u16 {
    // 値を書き換えるので `mut` が必要。
    let mut col = start;
    for _ in 0..steps {
        col += 1;
    }
    col
}

/// "  12  " のような文字列の前後の空白を除き、数値にして 2 倍したものを返す。
fn parse_and_double(input: &str) -> i32 {
    let value = input.trim();
    // 同じ名前で型の違う新しい変数を作る (シャドーイング)。
    let value: i32 = value.parse().unwrap();
    value * 2
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moves_right() {
        assert_eq!(move_right(0, 3), 3);
        assert_eq!(move_right(10, 0), 10);
    }

    #[test]
    fn parses_and_doubles() {
        assert_eq!(parse_and_double("  12  "), 24);
        assert_eq!(parse_and_double("-5"), -10);
    }
}
