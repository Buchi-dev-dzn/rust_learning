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
    // shout は所有権を要求するので、複製を渡して text を手元に残す。
    let upper = shout(text.clone());
    format!("{text} / {upper}")
}

/// タイトルの長さと、タイトル自身をタプルで返す。
fn title_with_len(title: String) -> (usize, String) {
    // len() (コピーされる usize) を先に取ってから、title をムーブする。
    let len = title.len();
    (len, title)
}

fn append_bang(mut s: String) -> String {
    s.push('!');
    s
}

/// "go" に '!' を 3 回つけた "go!!!" を返す。
fn triple_bang() -> String {
    let mut s = String::from("go");
    // 渡して、返してもらったものを受け取り直す。
    for _ in 0..3 {
        s = append_bang(s);
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
