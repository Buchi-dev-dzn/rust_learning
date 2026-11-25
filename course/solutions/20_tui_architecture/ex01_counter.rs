//! # カウンタアプリの update (Undo 付き)
//!
//! Elm アーキテクチャの `update` と `view` を実装しよう。端末は使わない。
//!
//! `Msg` ごとの動作:
//! - `Increment` / `Decrement` … `count` を `step` だけ増減する。結果は `-1000..=1000` に収める (clamp)
//! - `SetStep(n)` … `step` を `n` にする。ただし `1..=100` に収める
//! - `Reset` … `count` を 0 にする
//! - `Undo` … 直前の `count` に戻す (履歴が空なら何もしない)
//! - `Quit` … `quit = true`
//!
//! 履歴のルール: `Increment`/`Decrement`/`Reset` で **`count` が実際に変わったときだけ**、
//! 変更前の値を `history` に積む (clamp で値が変わらなかったときや、0 のときの Reset は積まない)。
//! `Undo` は `history` から取り出して `count` に戻す。`SetStep` は履歴に関係しない。
//!
//! `view` は `"count: {count} (step {step}) undo:{履歴の数}"` を返す。例: `"count: 3 (step 1) undo:3"`
//!
//! - 関連: 第20章「Elm アーキテクチャ」
//!
//! テスト (`mod tests`) は編集しないこと。

#[derive(Debug, Clone, PartialEq)]
struct Model {
    count: i64,
    step: i64,
    history: Vec<i64>,
    quit: bool,
}

impl Default for Model {
    fn default() -> Self {
        Model { count: 0, step: 1, history: Vec::new(), quit: false }
    }
}

#[derive(Debug, Clone, Copy)]
enum Msg {
    Increment,
    Decrement,
    SetStep(i64),
    Reset,
    Undo,
    Quit,
}

/// count を new に変える。変わったときだけ履歴に積む
fn set_count(m: &mut Model, new: i64) {
    let new = new.clamp(-1000, 1000);
    if new != m.count {
        m.history.push(m.count);
        m.count = new;
    }
}

fn update(m: &mut Model, msg: Msg) {
    match msg {
        Msg::Increment => set_count(m, m.count + m.step),
        Msg::Decrement => set_count(m, m.count - m.step),
        Msg::SetStep(n) => m.step = n.clamp(1, 100),
        Msg::Reset => set_count(m, 0),
        Msg::Undo => {
            if let Some(prev) = m.history.pop() {
                m.count = prev;
            }
        }
        Msg::Quit => m.quit = true,
    }
}

fn view(m: &Model) -> String {
    format!("count: {} (step {}) undo:{}", m.count, m.step, m.history.len())
}

fn main() {
    let mut m = Model::default();
    for msg in [Msg::Increment, Msg::SetStep(10), Msg::Increment, Msg::Undo, Msg::Quit] {
        update(&mut m, msg);
        println!("{msg:?} -> {}", view(&m));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn run(msgs: &[Msg]) -> Model {
        let mut m = Model::default();
        for &msg in msgs {
            update(&mut m, msg);
        }
        m
    }

    #[test]
    fn inc_dec() {
        let m = run(&[Msg::Increment, Msg::Increment, Msg::Decrement, Msg::Increment]);
        assert_eq!(m.count, 2);
        assert_eq!(m.history, vec![0, 1, 2, 1]);
    }

    #[test]
    fn step() {
        let m = run(&[Msg::SetStep(5), Msg::Increment, Msg::Increment]);
        assert_eq!(m.count, 10);
        assert_eq!(run(&[Msg::SetStep(0)]).step, 1);
        assert_eq!(run(&[Msg::SetStep(-3)]).step, 1);
        assert_eq!(run(&[Msg::SetStep(500)]).step, 100);
        assert!(run(&[Msg::SetStep(5)]).history.is_empty());
    }

    #[test]
    fn clamp_and_history() {
        let mut m = Model { count: 995, step: 10, ..Default::default() };
        update(&mut m, Msg::Increment);
        assert_eq!(m.count, 1000);
        update(&mut m, Msg::Increment); // 変化なし → 履歴に積まない
        assert_eq!(m.count, 1000);
        assert_eq!(m.history, vec![995]);
        let mut m = Model { count: -1000, ..Default::default() };
        update(&mut m, Msg::Decrement);
        assert_eq!(m.count, -1000);
        assert!(m.history.is_empty());
    }

    #[test]
    fn reset_and_undo() {
        let m = run(&[Msg::Increment, Msg::Increment, Msg::Reset]);
        assert_eq!(m.count, 0);
        assert_eq!(m.history, vec![0, 1, 2]);
        let m = run(&[Msg::Increment, Msg::Increment, Msg::Reset, Msg::Undo]);
        assert_eq!(m.count, 2);
        let m = run(&[Msg::Reset]);
        assert!(m.history.is_empty());
        let m = run(&[Msg::Increment, Msg::Undo, Msg::Undo, Msg::Undo]);
        assert_eq!(m.count, 0);
        assert!(m.history.is_empty());
    }

    #[test]
    fn quit() {
        assert!(!run(&[Msg::Increment]).quit);
        assert!(run(&[Msg::Quit]).quit);
    }

    #[test]
    fn view_text() {
        assert_eq!(view(&Model::default()), "count: 0 (step 1) undo:0");
        let m = run(&[Msg::Increment, Msg::Increment, Msg::Increment]);
        assert_eq!(view(&m), "count: 3 (step 1) undo:3");
        let m = run(&[Msg::SetStep(7), Msg::Decrement]);
        assert_eq!(view(&m), "count: -7 (step 7) undo:1");
    }
}
