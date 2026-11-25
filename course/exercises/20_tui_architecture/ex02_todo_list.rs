//! # ToDo リストの状態機械
//!
//! 選択カーソルと「追加入力モード」を持つ ToDo リスト TUI のロジック部分を作ろう。
//! `App::update` は `Msg` を受けて状態を変え、終了するなら `true` を返す。
//!
//! `Mode::Normal` のとき:
//! - `Up` … `selected` を 1 減らす (0 より小さくしない)
//! - `Down` … `selected` を 1 増やす (最後の項目より先に行かない。空なら 0 のまま)
//! - `Toggle` … 選択中の項目の `done` を反転 (空なら何もしない)
//! - `Delete` … 選択中の項目を削除。削除後 `selected` が範囲外なら最後の項目に合わせる (空なら 0)
//! - `StartAdd` … `Mode::Adding { input: 空文字列 }` へ
//! - `Quit` … `true` を返す
//! - `Input` / `Backspace` / `Submit` / `Cancel` … 何もしない
//!
//! `Mode::Adding { input }` のとき:
//! - `Input(c)` … `input` の末尾に `c` を追加 / `Backspace` … 末尾の 1 文字を削除
//! - `Submit` … `input` の前後の空白を除いた文字列が空でなければ、未完了の項目として末尾に追加し、
//!   `selected` をその項目に合わせる。空なら追加しない。どちらでも `Normal` に戻る
//! - `Cancel` … 追加せず `Normal` に戻る
//! - それ以外 (`Quit` を含む) … 何もしない (`false`)
//!
//! `view` は 1 項目 1 行の `Vec<String>` を返す:
//! - 選択中の行は `"> "`、それ以外は `"  "` で始まり、続けて `"[x] "` (完了) か `"[ ] "` (未完了)、タイトル
//! - 項目が 0 個なら `"  (項目なし)"` の 1 行
//! - `Adding` モードなら最後に `"追加: {input}_"` の行を足す
//!
//! - 関連: 第20章「状態の設計のコツ」
//!
//! テスト (`mod tests`) は編集しないこと。

#[derive(Debug, Clone, PartialEq)]
struct Item {
    title: String,
    done: bool,
}

#[derive(Debug, Clone, PartialEq)]
enum Mode {
    Normal,
    Adding { input: String },
}

#[derive(Debug, Clone, PartialEq)]
enum Msg {
    Up,
    Down,
    Toggle,
    Delete,
    StartAdd,
    Input(char),
    Backspace,
    Submit,
    Cancel,
    Quit,
}

#[derive(Debug)]
struct App {
    items: Vec<Item>,
    selected: usize,
    mode: Mode,
}

impl App {
    fn new(titles: &[&str]) -> Self {
        let items = titles.iter().map(|t| Item { title: t.to_string(), done: false }).collect();
        App { items, selected: 0, mode: Mode::Normal }
    }

    // TODO: モードと Msg に応じて状態を更新する。終了なら true
    fn update(&mut self, msg: Msg) -> bool {
        todo!()
    }

    // TODO: 画面の各行を作る
    fn view(&self) -> Vec<String> {
        todo!()
    }
}

fn main() {
    let mut app = App::new(&["牛乳を買う", "Rust の勉強"]);
    for msg in [Msg::Down, Msg::Toggle, Msg::StartAdd, Msg::Input('散'), Msg::Input('歩')] {
        app.update(msg);
    }
    for line in app.view() {
        println!("{line}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn app3() -> App {
        App::new(&["a", "b", "c"])
    }

    #[test]
    fn move_selection() {
        let mut app = app3();
        app.update(Msg::Up);
        assert_eq!(app.selected, 0);
        app.update(Msg::Down);
        app.update(Msg::Down);
        app.update(Msg::Down);
        assert_eq!(app.selected, 2);
        app.update(Msg::Up);
        assert_eq!(app.selected, 1);
        let mut empty = App::new(&[]);
        empty.update(Msg::Down);
        assert_eq!(empty.selected, 0);
    }

    #[test]
    fn toggle() {
        let mut app = app3();
        app.update(Msg::Down);
        app.update(Msg::Toggle);
        assert!(app.items[1].done);
        app.update(Msg::Toggle);
        assert!(!app.items[1].done);
        let mut empty = App::new(&[]);
        empty.update(Msg::Toggle); // パニックしない
    }

    #[test]
    fn delete_keeps_selection_in_range() {
        let mut app = app3();
        app.update(Msg::Down);
        app.update(Msg::Down);
        app.update(Msg::Delete); // c を削除
        assert_eq!(app.items.len(), 2);
        assert_eq!(app.selected, 1);
        app.update(Msg::Up);
        app.update(Msg::Delete); // a を削除
        assert_eq!(app.items[0].title, "b");
        assert_eq!(app.selected, 0);
        app.update(Msg::Delete);
        assert!(app.items.is_empty());
        assert_eq!(app.selected, 0);
        app.update(Msg::Delete); // 空でもパニックしない
    }

    #[test]
    fn add_item() {
        let mut app = app3();
        app.update(Msg::StartAdd);
        for c in " xy".chars() {
            app.update(Msg::Input(c));
        }
        app.update(Msg::Input('z'));
        app.update(Msg::Backspace);
        assert_eq!(app.mode, Mode::Adding { input: " xy".into() });
        assert!(!app.update(Msg::Quit)); // 入力中は終了しない
        app.update(Msg::Down); // 入力中は無視
        assert_eq!(app.selected, 0);
        app.update(Msg::Submit);
        assert_eq!(app.mode, Mode::Normal);
        assert_eq!(app.items.len(), 4);
        assert_eq!(app.items[3], Item { title: "xy".into(), done: false });
        assert_eq!(app.selected, 3);
    }

    #[test]
    fn empty_submit_and_cancel() {
        let mut app = app3();
        app.update(Msg::StartAdd);
        app.update(Msg::Input(' '));
        app.update(Msg::Submit);
        assert_eq!(app.items.len(), 3);
        assert_eq!(app.mode, Mode::Normal);
        app.update(Msg::StartAdd);
        app.update(Msg::Input('q'));
        app.update(Msg::Cancel);
        assert_eq!(app.items.len(), 3);
        assert_eq!(app.mode, Mode::Normal);
        // Normal では Input などは無視
        app.update(Msg::Input('x'));
        app.update(Msg::Submit);
        assert_eq!(app.items.len(), 3);
    }

    #[test]
    fn quit() {
        let mut app = app3();
        assert!(!app.update(Msg::Down));
        assert!(app.update(Msg::Quit));
    }

    #[test]
    fn view_lines() {
        let mut app = App::new(&["牛乳", "勉強"]);
        app.update(Msg::Down);
        app.update(Msg::Toggle);
        assert_eq!(app.view(), vec!["  [ ] 牛乳", "> [x] 勉強"]);
        app.update(Msg::StartAdd);
        app.update(Msg::Input('散'));
        assert_eq!(app.view(), vec!["  [ ] 牛乳", "> [x] 勉強", "追加: 散_"]);
        assert_eq!(App::new(&[]).view(), vec!["  (項目なし)"]);
    }
}
