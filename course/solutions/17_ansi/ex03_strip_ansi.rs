//! # ANSI シーケンスを取り除く
//!
//! 色付きの文字列から、エスケープシーケンスを取り除いて「見える文字」だけを残す
//! `strip_ansi` を実装しよう。色付きの出力をテストしたり、色付き文字列の表示幅を
//! 計算したりするときに使う。
//!
//! ルール:
//! - CSI: `ESC [` の後、引数バイト (`0x30..=0x3F`: 数字・`;`・`?` など) と
//!   中間バイト (`0x20..=0x2F`) が 0 個以上続き、終端バイト (`0x40..=0x7E`: `m`, `H`, `J` など)
//!   1 つで終わる。この全体を取り除く
//! - `ESC` の後が `[` 以外の 1 文字 (例: `ESC 7`) なら、`ESC` とその 1 文字を取り除く
//! - 文字列の末尾で途切れた不完全なシーケンスは、そこから末尾まで取り除く
//! - それ以外の文字はそのまま残す (日本語も)
//!
//! `visible_len(s)` は `strip_ansi(s)` の char 数を返す。
//!
//! - 関連: 第17章「エスケープシーケンスの形」
//!
//! テスト (`mod tests`) は編集しないこと。

fn strip_ansi(s: &str) -> String {
    let mut out = String::new();
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c != '\x1b' {
            out.push(c);
            continue;
        }
        match chars.next() {
            Some('[') => {
                // 終端バイトが来るまで読み飛ばす
                while let Some(b) = chars.next() {
                    if ('\x40'..='\x7e').contains(&b) {
                        break;
                    }
                }
            }
            Some(_) => {}  // ESC + 1 文字
            None => {}     // 末尾の ESC
        }
    }
    out
}

fn visible_len(s: &str) -> usize {
    strip_ansi(s).chars().count()
}

fn main() {
    let s = "\x1b[1;31mError:\x1b[0m ファイルがありません";
    println!("{s}");
    println!("{:?} ({} 文字)", strip_ansi(s), visible_len(s));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn no_escape() {
        assert_eq!(strip_ansi("hello"), "hello");
        assert_eq!(strip_ansi(""), "");
        assert_eq!(strip_ansi("日本語"), "日本語");
    }

    #[test]
    fn sgr() {
        assert_eq!(strip_ansi("\x1b[31mred\x1b[0m"), "red");
        assert_eq!(strip_ansi("a\x1b[38;2;1;2;3mb\x1b[mc"), "abc");
    }

    #[test]
    fn cursor_and_private() {
        assert_eq!(strip_ansi("\x1b[2J\x1b[1;1Htop"), "top");
        assert_eq!(strip_ansi("\x1b[?1049h\x1b[?25lX\x1b[?25h"), "X");
    }

    #[test]
    fn two_byte_escape() {
        assert_eq!(strip_ansi("\x1b7saved\x1b8"), "saved");
    }

    #[test]
    fn incomplete_at_end() {
        assert_eq!(strip_ansi("ok\x1b[31"), "ok");
        assert_eq!(strip_ansi("ok\x1b"), "ok");
    }

    #[test]
    fn visible() {
        assert_eq!(visible_len("\x1b[1m太字\x1b[0m!"), 3);
    }
}
