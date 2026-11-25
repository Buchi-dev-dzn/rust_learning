//! # ムーブのエラーを直す
//!
//! このファイルは **コンパイルが通りません** (E0382: use of moved value)。
//! 各関数のエラーを読み、関数の中だけを直してテストを通そう。
//!
//! - ゴール: ムーブが起きる場所を見つけ、`clone` / 順序の入れ替え / 戻り値の受け取り で直す
//! - 関数のシグネチャ (引数と戻り値の型) と、テスト (`mod tests`) は変更しないこと。
//! - 関連: 第2章「ムーブ」「clone」「関数と所有権」

fn shout(s: String) -> String {
    s.to_uppercase()
}

/// 元の文字列と、大文字にした文字列を " / " でつないで返す。
/// 例: "hi" -> "hi / HI"
fn original_and_upper(text: String) -> String {
    // TODO: shout に渡した時点で text はムーブされてしまう
    let upper = shout(text);
    format!("{text} / {upper}")
}

/// タイトルの長さと、タイトル自身をタプルで返す。
fn title_with_len(title: String) -> (usize, String) {
    // TODO: タプルを作る順序に注目。title をムーブした後に title.len() を呼んでいる
    let pair = (title, title.len());
    (pair.1, pair.0)
}

fn append_bang(mut s: String) -> String {
    s.push('!');
    s
}

/// "go" に '!' を 3 回つけた "go!!!" を返す。
fn triple_bang() -> String {
    let s = String::from("go");
    // TODO: append_bang は所有権を受け取り、加工したものを「返して」くれる。
    //       返り値を受け取らないと s は失われる
    for _ in 0..3 {
        append_bang(s);
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_original() {
        assert_eq!(original_and_upper(String::from("hi")), "hi / HI");
    }

    #[test]
    fn title_len() {
        assert_eq!(title_with_len(String::from("menu")), (4, String::from("menu")));
    }

    #[test]
    fn bangs() {
        assert_eq!(triple_bang(), "go!!!");
    }
}
