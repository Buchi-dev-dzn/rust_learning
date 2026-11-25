//! # wc を作る (BufRead と Write)
//!
//! `wc` コマンドのように、入力の **行数・単語数・バイト数** を数える関数と、
//! その結果を出力先に書く関数を完成させよう。
//!
//! - `count<R: BufRead>(r)` … 行数 (改行 `\n` の数)・単語数 (空白区切り)・バイト数を返す
//! - `report<W: Write>(w, stats, name)` … `"  行数  単語数  バイト数 name\n"` の形式で書く
//!   (各数値は幅 7 で右寄せ。例: `"      2       3      14 a.txt\n"`)
//! - 入出力をジェネリクスにしているので、テストでは `&[u8]` と `Vec<u8>` を使える
//! - 関連: 第15章「BufRead」「Write トレイト」
//!
//! テスト (`mod tests`) は編集しないこと。

use std::io::{self, BufRead, Write};

#[derive(Debug, PartialEq, Default)]
struct Stats {
    lines: usize,
    words: usize,
    bytes: usize,
}

fn count<R: BufRead>(mut r: R) -> io::Result<Stats> {
    let mut stats = Stats::default();
    let mut buf = String::new();
    loop {
        buf.clear();
        // read_line は改行を含めて読むので、バイト数を正確に数えられる
        let n = r.read_line(&mut buf)?;
        if n == 0 {
            break;
        }
        stats.bytes += n;
        if buf.ends_with('\n') {
            stats.lines += 1;
        }
        stats.words += buf.split_whitespace().count();
    }
    Ok(stats)
}

fn report<W: Write>(w: &mut W, s: &Stats, name: &str) -> io::Result<()> {
    writeln!(w, "{:>7} {:>7} {:>7} {}", s.lines, s.words, s.bytes, name)
}

fn main() {
    let s = count("hello world\nbye\n".as_bytes()).unwrap();
    report(&mut io::stdout(), &s, "-").unwrap();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_simple() {
        let s = count("hello world\nbye\n".as_bytes()).unwrap();
        assert_eq!(s, Stats { lines: 2, words: 3, bytes: 16 });
    }

    #[test]
    fn last_line_without_newline() {
        // wc と同じく、改行で終わらない最後の行は行数に数えない
        let s = count("a b\nc".as_bytes()).unwrap();
        assert_eq!(s, Stats { lines: 1, words: 3, bytes: 5 });
    }

    #[test]
    fn multibyte_counts_bytes() {
        let s = count("あい う\n".as_bytes()).unwrap();
        assert_eq!(s, Stats { lines: 1, words: 2, bytes: 11 });
    }

    #[test]
    fn empty_input() {
        assert_eq!(count("".as_bytes()).unwrap(), Stats::default());
    }

    #[test]
    fn report_format() {
        let mut out = Vec::new();
        report(&mut out, &Stats { lines: 2, words: 3, bytes: 14 }, "a.txt").unwrap();
        assert_eq!(String::from_utf8(out).unwrap(), "      2       3      14 a.txt\n");
    }
}
