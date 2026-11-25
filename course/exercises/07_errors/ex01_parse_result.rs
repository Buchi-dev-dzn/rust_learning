//! # Result と ? で入力を検証する
//!
//! `todo!()` の部分を実装して、すべてのテストを通そう。
//!
//! - ゴール: `Result` を返す関数を書き、`?` / `map_err` / `ok_or` でエラーを伝える
//! - テスト (`mod tests`) は編集しないこと。
//! - 関連: 第7章「Result<T, E>」「? 演算子」「独自のエラー型」
//!
//! `--size 80x24` のような端末サイズ指定を検証する関数を作る。

#[derive(Debug, PartialEq)]
enum SizeError {
    /// 'x' が含まれていない
    NoSeparator,
    /// 数値として読めない (読めなかった部分の文字列を持つ)
    NotANumber(String),
    /// 幅か高さが 0
    Zero,
    /// 幅か高さが上限 (1000) を超えている
    TooLarge(u32),
}

const MAX: u32 = 1000;

/// 1 つの数値を読む。読めなければ NotANumber(元の文字列)。
fn parse_dim(s: &str) -> Result<u32, SizeError> {
    // TODO: s.trim().parse::<u32>() の Err を map_err で SizeError::NotANumber に変換する
    todo!()
}

/// 0 と上限をチェックする。問題なければ Ok(n)。
fn check_dim(n: u32) -> Result<u32, SizeError> {
    // TODO
    todo!()
}

/// "WxH" を (幅, 高さ) に変換する。
/// エラーの優先順位: 区切りなし → 幅の数値エラー → 高さの数値エラー → 幅の範囲 → 高さの範囲
fn parse_size(s: &str) -> Result<(u32, u32), SizeError> {
    // TODO: split_once('x') は Option を返す。ok_or(SizeError::NoSeparator)? で Result にして伝播
    //       あとは parse_dim と check_dim を ? でつなぐ
    todo!()
}

/// 失敗したら既定値 80x24 を使う
fn size_or_default(s: &str) -> (u32, u32) {
    // TODO: unwrap_or を使う
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid() {
        assert_eq!(parse_size("80x24"), Ok((80, 24)));
        assert_eq!(parse_size(" 120 x 40 "), Ok((120, 40)));
        assert_eq!(parse_size("1000x1"), Ok((1000, 1)));
    }

    #[test]
    fn reports_errors() {
        assert_eq!(parse_size("80*24"), Err(SizeError::NoSeparator));
        assert_eq!(parse_size("abcx24"), Err(SizeError::NotANumber("abc".to_string())));
        assert_eq!(parse_size("80x-1"), Err(SizeError::NotANumber("-1".to_string())));
        assert_eq!(parse_size("0x24"), Err(SizeError::Zero));
        assert_eq!(parse_size("80x1001"), Err(SizeError::TooLarge(1001)));
        // 優先順位: 数値エラーは範囲エラーより先
        assert_eq!(parse_size("0xzz"), Err(SizeError::NotANumber("zz".to_string())));
    }

    #[test]
    fn defaults() {
        assert_eq!(size_or_default("100x50"), (100, 50));
        assert_eq!(size_or_default("oops"), (80, 24));
    }
}
