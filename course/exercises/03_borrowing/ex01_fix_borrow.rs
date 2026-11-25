//! # 借用エラーを直す
//!
//! このファイルは **コンパイルが通りません** (E0502 / E0499 / E0596 など)。
//! エラーメッセージの「借りた場所」「衝突した場所」「later used here」を読んで直そう。
//!
//! - ゴール: 借用ルール (共有はいくつでも / 可変は 1 つだけ) に沿ってコードを並べ替える
//! - テスト (`mod tests`) は変更しないこと。関数の中身と、必要なら引数の型を直す。
//! - 関連: 第3章「借用のルール」「よくあるコンパイルエラー」

/// 行リストの末尾に `new_line` を追加し、「追加前の最後の行 -> 追加した行」という報告を返す。
/// lines が空なら "(none) -> new_line"。
fn append_and_report(lines: &mut Vec<String>, new_line: &str) -> String {
    // TODO: last は lines を共有借用している。その借用が生きている間に push できない
    let last = match lines.last() {
        Some(l) => l.as_str(),
        None => "(none)",
    };
    lines.push(new_line.to_string());
    format!("{last} -> {new_line}")
}

/// 先頭と末尾の要素を入れ替える (要素が 2 つ未満なら何もしない)。
fn swap_ends(v: &mut Vec<i32>) {
    if v.len() < 2 {
        return;
    }
    let n = v.len();
    // TODO: 同じ Vec への可変参照を同時に 2 つ作ることはできない
    //       (ヒント: スライスには swap(i, j) というメソッドがある)
    let a = &mut v[0];
    let b = &mut v[n - 1];
    std::mem::swap(a, b);
}

/// 文字列を (ASCII の範囲で) 大文字に書き換える。
// TODO: 書き換えるのに共有参照 `&String` で受け取っている
fn shout_in_place(s: &String) {
    s.make_ascii_uppercase();
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reports() {
        let mut lines = vec![String::from("first")];
        assert_eq!(append_and_report(&mut lines, "second"), "first -> second");
        assert_eq!(lines, vec!["first", "second"]);
        let mut empty = Vec::new();
        assert_eq!(append_and_report(&mut empty, "x"), "(none) -> x");
    }

    #[test]
    fn swaps() {
        let mut v = vec![1, 2, 3, 4];
        swap_ends(&mut v);
        assert_eq!(v, vec![4, 2, 3, 1]);
        let mut one = vec![9];
        swap_ends(&mut one);
        assert_eq!(one, vec![9]);
    }

    #[test]
    fn shouts() {
        let mut s = String::from("quit");
        shout_in_place(&mut s);
        assert_eq!(s, "QUIT");
    }
}
