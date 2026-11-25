//! # tick 付きイベントループと「必要なときだけ描画」
//!
//! ストップウォッチアプリを作ろう。イベントは `Key(char)` と `Tick(経過ミリ秒)` の 2 種類。
//! `update` は「画面の表示が変わったか (dirty)」を返し、`run` は dirty のときだけ `view` を呼ぶ。
//!
//! `Stopwatch::update(ev)`:
//! - `Tick(ms)` … 動作中 (`running`) なら `elapsed_ms` に `ms` を足す。停止中なら何もしない。
//!   表示は 0.1 秒単位なので、**表示が変わったとき (`elapsed_ms / 100` が変わったとき) だけ** `true`
//! - `Key(' ')` … 動作/停止を切り替える。`true`
//! - `Key('r')` … `elapsed_ms = 0`、停止する。状態が変わったなら `true`、もともと 0 で停止中なら `false`
//! - `Key('q')` … `quit = true`。`false`
//! - その他のキー … `false`
//!
//! `view()` … `"MM:SS.d ▶"` (動作中) / `"MM:SS.d ■"` (停止中)。例: 61234ms → `"01:01.2 ■"`
//!
//! `run(events)` … 最初に 1 回 `view` を記録し、イベントごとに `update`。
//! `quit` になったらすぐ終了 (そのイベントでは描画しない)。dirty なら `view` を記録する。
//! 記録したフレームの列を返す。
//!
//! - 関連: 第20章「イベントループ: 入力とタイマー」「必要なときだけ描く」
//!
//! テスト (`mod tests`) は編集しないこと。

#[derive(Debug, Clone, Copy, PartialEq)]
enum Event {
    Key(char),
    Tick(u64),
}

#[derive(Debug, Default)]
struct Stopwatch {
    running: bool,
    elapsed_ms: u64,
    quit: bool,
}

impl Stopwatch {
    // TODO: 状態を更新し、表示が変わったら true
    fn update(&mut self, ev: Event) -> bool {
        todo!()
    }

    // TODO: "MM:SS.d ▶" / "MM:SS.d ■"
    fn view(&self) -> String {
        todo!()
    }
}

// TODO: 初回描画 → イベントごとに update → quit なら終了 → dirty なら描画
fn run(events: impl IntoIterator<Item = Event>) -> Vec<String> {
    todo!()
}

fn main() {
    use Event::*;
    let mut events = vec![Key(' ')];
    events.extend(std::iter::repeat(Tick(50)).take(10));
    events.extend([Key(' '), Tick(50), Key('q')]);
    for f in run(events) {
        println!("{f}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use Event::*;

    #[test]
    fn view_format() {
        let mut s = Stopwatch::default();
        assert_eq!(s.view(), "00:00.0 ■");
        s.elapsed_ms = 61_234;
        assert_eq!(s.view(), "01:01.2 ■");
        s.running = true;
        s.elapsed_ms = 599_999;
        assert_eq!(s.view(), "09:59.9 ▶");
        s.elapsed_ms = 3_600_000;
        assert_eq!(s.view(), "60:00.0 ▶");
    }

    #[test]
    fn tick_only_when_running() {
        let mut s = Stopwatch::default();
        assert!(!s.update(Tick(500)));
        assert_eq!(s.elapsed_ms, 0);
        assert!(s.update(Key(' ')));
        assert!(s.running);
        assert!(!s.update(Tick(50))); // 0.05 秒: 表示は 00:00.0 のまま
        assert!(s.update(Tick(50))); // 0.10 秒: 表示が変わる
        assert_eq!(s.elapsed_ms, 100);
        assert!(s.update(Tick(250)));
    }

    #[test]
    fn reset() {
        let mut s = Stopwatch::default();
        assert!(!s.update(Key('r'))); // 何も変わらない
        s.update(Key(' '));
        s.update(Tick(300));
        assert!(s.update(Key('r')));
        assert_eq!((s.running, s.elapsed_ms), (false, 0));
    }

    #[test]
    fn other_keys_and_quit() {
        let mut s = Stopwatch::default();
        assert!(!s.update(Key('x')));
        assert!(!s.update(Key('q')));
        assert!(s.quit);
    }

    #[test]
    fn run_draws_only_when_dirty() {
        let mut evs = vec![Key(' ')];
        evs.extend(std::iter::repeat(Tick(50)).take(4)); // 0.2 秒 (2 回だけ表示が変わる)
        evs.extend([Key('x'), Key(' '), Tick(50), Key('q'), Key(' ')]);
        assert_eq!(
            run(evs),
            vec!["00:00.0 ■", "00:00.0 ▶", "00:00.1 ▶", "00:00.2 ▶", "00:00.2 ■"]
        );
    }

    #[test]
    fn run_initial_frame_only() {
        assert_eq!(run([Key('q'), Key(' ')]), vec!["00:00.0 ■"]);
        assert_eq!(run([]), vec!["00:00.0 ■"]);
    }
}
