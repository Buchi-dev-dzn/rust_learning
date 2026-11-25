//! # メソッドまわりのエラーを直す
//!
//! このファイルは **コンパイルが通りません**。
//! derive の不足、レシーバ (`self` / `&self` / `&mut self`) の選び間違いを直そう。
//!
//! - ゴール: コンパイラのメッセージから「どのトレイトが足りないか」「どのレシーバが正しいか」を判断する
//! - `// TODO:` の行の近くだけを直すこと。テスト (`mod tests`) は変更しないこと。
//! - 関連: 第4章「メソッドと impl」「よくあるコンパイルエラー」

// assert_eq! には PartialEq (==) と Debug ({:?} で失敗時に表示) が必要。
#[derive(Debug, PartialEq)]
struct Rgb(u8, u8, u8);

struct Theme {
    name: String,
    fg: Rgb,
    bg: Rgb,
}

impl Theme {
    fn new(name: &str) -> Self {
        Theme { name: name.to_string(), fg: Rgb(255, 255, 255), bg: Rgb(0, 0, 0) }
    }

    // 読むだけなので &self。フィールドへの参照 &str を返す。
    // (&str と "dark" は == で比較できる)
    fn name(&self) -> &str {
        &self.name
    }

    // 自分を書き換えるので &mut self。
    fn invert(&mut self) {
        std::mem::swap(&mut self.fg, &mut self.bg);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn theme_works() {
        let mut t = Theme::new("dark");
        assert_eq!(t.name(), "dark");
        t.invert();
        assert_eq!(t.fg, Rgb(0, 0, 0));
        assert_eq!(t.bg, Rgb(255, 255, 255));
        assert_eq!(t.name(), "dark"); // 2 回目も呼べること
    }
}
