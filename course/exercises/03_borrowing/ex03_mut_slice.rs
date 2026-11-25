//! # 可変スライスでセルを塗る
//!
//! `todo!()` の部分を実装して、すべてのテストを通そう。
//!
//! - ゴール: `&mut [T]` を受け取り、範囲外を安全に扱いながら中身を書き換える
//! - テスト (`mod tests`) は編集しないこと。
//! - 関連: 第3章「配列・Vec のスライス `&[T]`」「参照の参照外し」
//!
//! ターミナル 1 行ぶんのセル (`[char]`) を描画するミニ関数たちです。
//! 後の章で作る画面バッファの基礎になります。

/// `row` の `start..end` を `ch` で塗る。範囲が行からはみ出す部分は無視する。
/// `start >= end` なら何もしない。
fn fill_range(row: &mut [char], start: usize, end: usize, ch: char) {
    // TODO: end を row.len() で頭打ちにしてから、&mut row[start..end] の各要素に代入する
    //       (start が大きすぎる場合にもパニックしないこと)
    todo!()
}

/// `row` の位置 `x` から文字列 `text` を書き込む。はみ出した文字は捨てる。
/// 書き込めた文字数を返す。
fn put_str(row: &mut [char], x: usize, text: &str) -> usize {
    // TODO: text.chars() を順に見て、row の範囲内なら書き込む
    todo!()
}

/// 行に含まれる、空白 ' ' でないセルの数を数える (読むだけなので &[char])。
fn count_filled(row: &[char]) -> usize {
    // TODO
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn s(row: &[char]) -> String {
        row.iter().collect()
    }

    #[test]
    fn fills() {
        let mut row = [' '; 8];
        fill_range(&mut row, 2, 5, '#');
        assert_eq!(s(&row), "  ###   ");
        fill_range(&mut row, 6, 100, '=');
        assert_eq!(s(&row), "  ### ==");
        fill_range(&mut row, 50, 60, 'x');
        fill_range(&mut row, 4, 2, 'x');
        assert_eq!(s(&row), "  ### ==");
    }

    #[test]
    fn puts() {
        let mut row = ['.'; 6];
        assert_eq!(put_str(&mut row, 1, "abc"), 3);
        assert_eq!(s(&row), ".abc..");
        assert_eq!(put_str(&mut row, 4, "日本語"), 2);
        assert_eq!(s(&row), ".abc日本");
        assert_eq!(put_str(&mut row, 9, "zz"), 0);
    }

    #[test]
    fn counts() {
        let row = ['a', ' ', 'b', ' ', ' '];
        assert_eq!(count_filled(&row), 2);
        let v = vec!['x'; 3];
        assert_eq!(count_filled(&v), 3);
        assert_eq!(count_filled(&v[..0]), 0);
    }
}
