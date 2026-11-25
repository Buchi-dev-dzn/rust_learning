//! # チャネルで TUI イベントループを作る
//!
//! 入力スレッドと tick スレッドからのイベントを、1 本のチャネルで受け取るループを作ろう。
//! `TODO` の部分を直して、すべてのテストを通そう。テストは編集しないこと。
//!
//! - `spawn_input(tx, keys)`: `keys` を順に `Event::Key` として送るスレッドを起動する
//! - `spawn_ticks(tx, n)`: `Event::Tick` を `n` 回送って終わるスレッドを起動する
//! - `spawn_ticker(tx, interval)`: `interval` ごとに `Event::Tick` を送り続け、
//!   送信に失敗したら (受信側が消えたら) 終わるスレッドを起動する
//! - `App::handle(event)`: `'q'` で `quit = true`、`'\u{8}'` (Backspace) で 1 文字削除、
//!   他の文字は `text` に追加、`Tick` で `ticks += 1`
//! - `run(rx)`: イベントを受け取り続けて `App` を更新する。
//!   `quit` になるか、送信側がすべて消えたら終了して `App` を返す
//! - 関連: 第14章「TUI イベントループの骨格」

use std::sync::mpsc::{self, Receiver, Sender};
use std::thread::{self, JoinHandle};
use std::time::Duration;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Event {
    Key(char),
    Tick,
}

pub fn spawn_input(tx: Sender<Event>, keys: Vec<char>) -> JoinHandle<()> {
    // TODO: thread::spawn(move || ...) の中で keys を 1 つずつ tx.send する
    todo!()
}

pub fn spawn_ticks(tx: Sender<Event>, n: usize) -> JoinHandle<()> {
    // TODO: Event::Tick を n 回送る
    todo!()
}

pub fn spawn_ticker(tx: Sender<Event>, interval: Duration) -> JoinHandle<()> {
    // TODO: loop { sleep → send } 。send が Err なら return
    todo!()
}

#[derive(Debug, Default, PartialEq)]
pub struct App {
    pub text: String,
    pub ticks: u32,
    pub quit: bool,
}

impl App {
    pub fn handle(&mut self, event: Event) {
        // TODO: match event { ... }
        todo!()
    }
}

pub fn run(rx: Receiver<Event>) -> App {
    let mut app = App::default();
    // TODO: quit になるまで rx.recv() で受け取り、app.handle に渡す。
    //       recv() が Err (送信側が全滅) なら抜ける
    app
}

/// 便利関数: チャネルを作って (Sender, Receiver) を返す
pub fn channel() -> (Sender<Event>, Receiver<Event>) {
    mpsc::channel()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handle_updates_state() {
        let mut app = App::default();
        for ev in [Event::Key('a'), Event::Key('b'), Event::Key('\u{8}'), Event::Tick, Event::Key('c')] {
            app.handle(ev);
        }
        assert_eq!(app, App { text: "ac".into(), ticks: 1, quit: false });
        app.handle(Event::Key('\u{8}'));
        app.handle(Event::Key('\u{8}'));
        app.handle(Event::Key('\u{8}')); // 空でも panic しない
        app.handle(Event::Key('q'));
        assert_eq!(app, App { text: "".into(), ticks: 1, quit: true });
    }

    #[test]
    fn input_thread_sends_keys_in_order() {
        let (tx, rx) = channel();
        let h = spawn_input(tx, vec!['x', 'y', 'z']);
        h.join().unwrap();
        let got: Vec<Event> = rx.iter().collect();
        assert_eq!(got, vec![Event::Key('x'), Event::Key('y'), Event::Key('z')]);
    }

    #[test]
    fn run_ends_when_all_senders_dropped() {
        let (tx, rx) = channel();
        let a = spawn_input(tx.clone(), "hello".chars().collect());
        let b = spawn_ticks(tx, 7); // 最後の Sender を move
        let app = run(rx);
        a.join().unwrap();
        b.join().unwrap();
        // 2 つのスレッドのイベントは混ざって届くが、キーの順番は保たれる
        assert_eq!(app.text, "hello");
        assert_eq!(app.ticks, 7);
        assert!(!app.quit);
    }

    #[test]
    fn run_stops_on_quit_and_ticker_shuts_down() {
        let (tx, rx) = channel();
        let ticker = spawn_ticker(tx.clone(), Duration::from_millis(1));
        let input = spawn_input(tx, vec!['o', 'k', 'q', 'n', 'o']);
        let app = run(rx);
        assert!(app.quit);
        assert_eq!(app.text, "ok"); // q の後のキーは処理しない
        // run が rx を drop したので、ticker は send に失敗して自分で終わるはず
        ticker.join().unwrap();
        input.join().unwrap();
    }
}
