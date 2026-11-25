//! # メソッドまわりのエラーを直す
//!
//! このファイルは **コンパイルが通りません**。
//! derive の不足、レシーバ (`self` / `&self` / `&mut self`) の選び間違いを直そう。
//!
//! - ゴール: コンパイラのメッセージから「どのトレイトが足りないか」「どのレシーバが正しいか」を判断する
//! - `// TODO:` の行の近くだけを直すこと。テスト (`mod tests`) は変更しないこと。
//! - 関連: 第4章「メソッドと impl」「よくあるコンパイルエラー」

// TODO: テストで `assert_eq!` (== と {:?}) を使うのに必要なトレイトが足りない
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

    // TODO: 名前を見るだけなのに self の所有権を奪っている
    fn name(self) -> String {
        self.name
    }

    // TODO: 前景色と背景色を入れ替える。自分を書き換えるのに &self になっている
    fn invert(&self) {
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
