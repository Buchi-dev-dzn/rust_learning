//! # カーソル位置の変換 (char 位置 / バイト位置 / 表示列)
//!
//! テキストエディタでは 1 行の文字列に対して 3 種類の座標を行き来する。
//!
//! - **char 位置** … カーソルが何文字目にいるか (0 始まり)
//! - **バイト位置** … `String::insert` などに渡す位置
//! - **表示列** … 画面の何列目にカーソルを描くか
//!
//! 次の関数を実装しよう (`char_width` は用意済み)。
//!
//! - `byte_of(line, ci)` … char 位置 `ci` のバイト位置。`ci` が文字数以上なら `line.len()`
//! - `col_of(line, ci)` … char 位置 `ci` の直前までの表示幅の合計 (`ci` が大きすぎれば全体の幅)
//! - `char_at_col(line, col)` … 表示列 `col` にある文字の char 位置
//!   - 全角文字の右半分 (2 列目) を指していたら、その全角文字の位置を返す
//!   - 行末より右なら文字数 (行末) を返す
//! - `insert_at(line, ci, c)` … char 位置 `ci` に文字 `c` を挿入する (行末より右なら末尾に追加)
//!
//! - 関連: 第16章「TUI でなぜ必須か」
//!
//! テスト (`mod tests`) は編集しないこと。

fn char_width(c: char) -> usize {
    let cp = c as u32;
    if matches!(cp, 0x3000..=0x9FFF | 0xFF01..=0xFF60) { 2 } else { 1 }
}

// TODO: char_indices().nth(ci) を使う
fn byte_of(line: &str, ci: usize) -> usize {
    todo!()
}

// TODO: 先頭 ci 文字の幅の合計
fn col_of(line: &str, ci: usize) -> usize {
    todo!()
}

// TODO: 幅を足しながら、col がどの文字の範囲に入るかを探す
fn char_at_col(line: &str, col: usize) -> usize {
    todo!()
}

// TODO: byte_of でバイト位置に変換してから String::insert
fn insert_at(line: &mut String, ci: usize, c: char) {
    todo!()
}

fn main() {
    let line = "let 名前 = 1;";
    for ci in 0..6 {
        println!("char {ci}: byte {} col {}", byte_of(line, ci), col_of(line, ci));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const LINE: &str = "a漢b字c"; // 列: a=0, 漢=1-2, b=3, 字=4-5, c=6

    #[test]
    fn byte_positions() {
        assert_eq!(byte_of(LINE, 0), 0);
        assert_eq!(byte_of(LINE, 1), 1);
        assert_eq!(byte_of(LINE, 2), 4);
        assert_eq!(byte_of(LINE, 3), 5);
        assert_eq!(byte_of(LINE, 5), 9);
        assert_eq!(byte_of(LINE, 99), 9);
    }

    #[test]
    fn columns() {
        assert_eq!(col_of(LINE, 0), 0);
        assert_eq!(col_of(LINE, 1), 1);
        assert_eq!(col_of(LINE, 2), 3);
        assert_eq!(col_of(LINE, 4), 6);
        assert_eq!(col_of(LINE, 5), 7);
        assert_eq!(col_of(LINE, 99), 7);
    }

    #[test]
    fn col_to_char() {
        assert_eq!(char_at_col(LINE, 0), 0);
        assert_eq!(char_at_col(LINE, 1), 1);
        assert_eq!(char_at_col(LINE, 2), 1); // 漢 の右半分
        assert_eq!(char_at_col(LINE, 3), 2);
        assert_eq!(char_at_col(LINE, 5), 3);
        assert_eq!(char_at_col(LINE, 6), 4);
        assert_eq!(char_at_col(LINE, 7), 5);
        assert_eq!(char_at_col(LINE, 100), 5);
        assert_eq!(char_at_col("", 3), 0);
    }

    #[test]
    fn roundtrip() {
        for ci in 0..=5 {
            assert_eq!(char_at_col(LINE, col_of(LINE, ci)), ci);
        }
    }

    #[test]
    fn insert() {
        let mut s = String::from("あいう");
        insert_at(&mut s, 1, 'X');
        assert_eq!(s, "あXいう");
        insert_at(&mut s, 0, '[');
        insert_at(&mut s, 99, ']');
        assert_eq!(s, "[あXいう]");
    }
}
