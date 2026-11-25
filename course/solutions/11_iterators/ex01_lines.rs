//! # イテレータで行を処理する
//!
//! ログビューアの部品をイテレータチェーンで作ろう。
//! `TODO` の部分を直して、すべてのテストを通そう。テストは編集しないこと。
//! **`for` ループを使わず**、イテレータのメソッドだけで書いてみよう。
//!
//! - `non_empty_lines`: 空白だけの行を除いた行 (行そのものは trim しない)
//! - `grep`: `pattern` を含む行を `(行番号(1始まり), 行)` で返す
//! - `visible`: スクロール位置 `scroll` から `height` 行分を返す
//! - `numbered`: 各行を `"  3 | text"` (行番号は幅 3 の右寄せ) に整形する
//! - `parse_numbers`: カンマ区切りの整数を読む。1 つでも不正なら `Err`
//! - 関連: 第11章「よく使うアダプタと消費メソッド」

use std::num::ParseIntError;

pub fn non_empty_lines(text: &str) -> Vec<&str> {
    text.lines().filter(|l| !l.trim().is_empty()).collect()
}

pub fn grep<'a>(text: &'a str, pattern: &str) -> Vec<(usize, &'a str)> {
    text.lines()
        .enumerate()
        .filter(|(_, l)| l.contains(pattern))
        .map(|(i, l)| (i + 1, l))
        .collect()
}

pub fn visible(lines: &[String], scroll: usize, height: usize) -> Vec<&str> {
    lines.iter().skip(scroll).take(height).map(|s| s.as_str()).collect()
}

pub fn numbered(lines: &[&str]) -> Vec<String> {
    lines
        .iter()
        .enumerate()
        .map(|(i, l)| format!("{:>3} | {}", i + 1, l))
        .collect()
}

// Result<Vec<_>, _> に collect すると、最初の Err で止まってそれを返す
pub fn parse_numbers(s: &str) -> Result<Vec<i32>, ParseIntError> {
    s.split(',').map(|p| p.trim().parse::<i32>()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    const LOG: &str = "INFO start\n\n  \nWARN disk 90%\nINFO tick\nERROR disk full\n";

    #[test]
    fn non_empty() {
        assert_eq!(
            non_empty_lines(LOG),
            vec!["INFO start", "WARN disk 90%", "INFO tick", "ERROR disk full"]
        );
        assert_eq!(non_empty_lines(""), Vec::<&str>::new());
    }

    #[test]
    fn grep_with_line_numbers() {
        assert_eq!(grep(LOG, "disk"), vec![(4, "WARN disk 90%"), (6, "ERROR disk full")]);
        assert_eq!(grep(LOG, "nothing"), vec![]);
    }

    #[test]
    fn visible_window() {
        let lines: Vec<String> = (0..10).map(|i| format!("L{i}")).collect();
        assert_eq!(visible(&lines, 0, 3), vec!["L0", "L1", "L2"]);
        assert_eq!(visible(&lines, 8, 5), vec!["L8", "L9"]);
        assert_eq!(visible(&lines, 20, 5), Vec::<&str>::new());
    }

    #[test]
    fn numbered_lines() {
        assert_eq!(numbered(&["a", "b"]), vec!["  1 | a", "  2 | b"]);
        let many: Vec<&str> = vec!["x"; 100];
        assert_eq!(numbered(&many)[99], "100 | x");
    }

    #[test]
    fn parse_ok_and_err() {
        assert_eq!(parse_numbers("1, 2,3"), Ok(vec![1, 2, 3]));
        assert!(parse_numbers("1,x,3").is_err());
        assert!(parse_numbers("").is_err());
    }
}
