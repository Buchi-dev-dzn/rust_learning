//! # クロージャでキーバインド表を作る
//!
//! TUI アプリの「キー → 動作」の対応表を、クロージャで作ろう。
//! `TODO` の部分を直して、すべてのテストを通そう。テストは編集しないこと。
//!
//! - `make_counter`: 呼ぶたびに 1, 2, 3, ... を返すクロージャを返す (`FnMut`)
//! - `make_clamper`: 値を `0..=max` に収めるクロージャを返す (`Fn`)
//! - `Keymap`: `char` → `Box<dyn Fn(&mut App)>` の表
//!   - `bind(key, action)`: 登録 (同じキーは上書き)
//!   - `handle(key, app)`: 登録があれば実行して `true`、なければ `false`
//! - `default_keymap`: `j` で cursor+1 (最大 `app.len - 1`)、`k` で cursor-1 (最小 0)、`q` で quit
//! - 関連: 第11章「クロージャ」「クロージャを引数・戻り値・フィールドにする」

use std::collections::HashMap;

// move で count をクロージャの中に持たせる
pub fn make_counter() -> impl FnMut() -> u32 {
    let mut count = 0;
    move || {
        count += 1;
        count
    }
}

pub fn make_clamper(max: i64) -> impl Fn(i64) -> i64 {
    move |v| v.clamp(0, max)
}

#[derive(Debug, Default, PartialEq)]
pub struct App {
    pub cursor: usize,
    pub len: usize,
    pub quit: bool,
}

pub struct Keymap {
    actions: HashMap<char, Box<dyn Fn(&mut App)>>,
}

impl Keymap {
    pub fn new() -> Self {
        Keymap { actions: HashMap::new() }
    }

    // 'static: 表に入れたクロージャは、呼び出し元の一時的な変数を借用できない
    pub fn bind(&mut self, key: char, action: impl Fn(&mut App) + 'static) {
        self.actions.insert(key, Box::new(action));
    }

    pub fn handle(&self, key: char, app: &mut App) -> bool {
        match self.actions.get(&key) {
            Some(action) => {
                action(app);
                true
            }
            None => false,
        }
    }
}

pub fn default_keymap() -> Keymap {
    let mut km = Keymap::new();
    km.bind('j', |app| {
        if app.cursor + 1 < app.len {
            app.cursor += 1;
        }
    });
    // saturating_sub は 0 未満にならない引き算
    km.bind('k', |app| app.cursor = app.cursor.saturating_sub(1));
    km.bind('q', |app| app.quit = true);
    km
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counter_counts() {
        let mut c = make_counter();
        assert_eq!(c(), 1);
        assert_eq!(c(), 2);
        let mut other = make_counter();
        assert_eq!(other(), 1); // 別のカウンタは独立
        assert_eq!(c(), 3);
    }

    #[test]
    fn clamper_clamps() {
        let f = make_clamper(79);
        assert_eq!(f(-5), 0);
        assert_eq!(f(40), 40);
        assert_eq!(f(200), 79);
    }

    #[test]
    fn default_keys_move_cursor() {
        let km = default_keymap();
        let mut app = App { len: 3, ..Default::default() };
        assert!(km.handle('j', &mut app));
        assert!(km.handle('j', &mut app));
        assert!(km.handle('j', &mut app)); // 最後で止まる
        assert_eq!(app.cursor, 2);
        km.handle('k', &mut app);
        km.handle('k', &mut app);
        km.handle('k', &mut app); // 0 で止まる
        assert_eq!(app.cursor, 0);
        assert!(!app.quit);
        km.handle('q', &mut app);
        assert!(app.quit);
    }

    #[test]
    fn unknown_key_returns_false() {
        let km = default_keymap();
        let mut app = App { len: 3, ..Default::default() };
        assert!(!km.handle('x', &mut app));
        assert_eq!(app, App { len: 3, ..Default::default() });
    }

    #[test]
    fn custom_binding_with_captured_value() {
        let mut km = default_keymap();
        let page = 10;
        km.bind('G', move |app| app.cursor = app.len.saturating_sub(1));
        km.bind('d', move |app| app.cursor = (app.cursor + page).min(app.len - 1));
        let mut app = App { len: 25, ..Default::default() };
        km.handle('d', &mut app);
        assert_eq!(app.cursor, 10);
        km.handle('G', &mut app);
        assert_eq!(app.cursor, 24);
        // 上書き
        km.bind('q', |app| app.cursor = 0);
        km.handle('q', &mut app);
        assert_eq!(app.cursor, 0);
        assert!(!app.quit);
    }
}
