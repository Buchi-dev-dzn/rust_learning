//! # ? まわりのエラーを直す
//!
//! このファイルは **コンパイルが通りません**。`?` 演算子と `Result` にまつわる典型的なエラーを直そう。
//!
//! - ゴール: 「`?` は Result を返す関数でしか使えない」「`?` は From でエラーを変換する」
//!   「成功値は `Ok(..)` で包む」の 3 点を、コンパイラのメッセージから読み取る
//! - `// TODO:` の付いた箇所だけを直すこと。テスト (`mod tests`) は変更しないこと。
//! - 関連: 第7章「? 演算子」「From と ? による自動変換」「よくあるコンパイルエラー」

use std::num::ParseIntError;

#[derive(Debug, PartialEq)]
enum LoadError {
    Parse(ParseIntError),
    Empty,
}

// `?` は Err(e) を返すときに From::from(e) を呼ぶ。これで ParseIntError → LoadError。
impl From<ParseIntError> for LoadError {
    fn from(e: ParseIntError) -> Self {
        LoadError::Parse(e)
    }
}

/// カンマ区切りの数値を読み込む。空文字列なら LoadError::Empty。
fn load_numbers(s: &str) -> Result<Vec<i64>, LoadError> {
    if s.trim().is_empty() {
        return Err(LoadError::Empty);
    }
    let mut out = Vec::new();
    for part in s.split(',') {
        let n = part.trim().parse::<i64>()?;
        out.push(n);
    }
    // 成功値は Ok で包む。
    Ok(out)
}

/// 数値の合計を返す。
// `?` を使うので戻り値は Result。
fn total(s: &str) -> Result<i64, LoadError> {
    let nums = load_numbers(s)?;
    Ok(nums.iter().sum())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn loads() {
        assert_eq!(load_numbers("1, 2,3"), Ok(vec![1, 2, 3]));
        assert_eq!(load_numbers("  "), Err(LoadError::Empty));
        assert!(matches!(load_numbers("1,x"), Err(LoadError::Parse(_))));
    }

    #[test]
    fn totals() {
        assert_eq!(total("10,20,-5"), Ok(25));
        assert_eq!(total(""), Err(LoadError::Empty));
        assert!(total("1,,2").is_err());
    }
}
