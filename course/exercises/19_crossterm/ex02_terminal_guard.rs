//! # TerminalGuard: 失敗しても・パニックしても端末を戻す
//!
//! crossterm の `enable_raw_mode` や `execute!(EnterAlternateScreen)` を
//! `Command` enum と `Backend` トレイトで抽象化した。偽のバックエンドで動作を確かめながら、
//! 安全な `TerminalGuard` を作ろう。
//!
//! 1. `inverse(cmd)` … 各コマンドを打ち消すコマンドを返す
//!    (`EnableRawMode` ↔ `DisableRawMode`, `EnterAlternateScreen` ↔ `LeaveAlternateScreen`,
//!    `HideCursor` ↔ `ShowCursor`, `EnableMouse` ↔ `DisableMouse`)
//! 2. `TerminalGuard::new(backend, mouse)` … 次の順に実行する
//!    `EnableRawMode` → `EnterAlternateScreen` → `HideCursor` → (`mouse` が true なら) `EnableMouse`。
//!    途中で失敗したら、**それまでに成功したコマンドを逆順に打ち消して** から `Err` を返す (ロールバック)
//! 3. `restore()` … セットアップしたコマンドを逆順に打ち消す。
//!    - 途中でエラーが出ても残りは全部実行する (後片付けは最後までやる)
//!    - 2 回目以降の呼び出しでは何もしない (冪等。パニックフックと Drop の両方から呼ばれても安全)
//! 4. `Drop` で `restore()` を呼ぶ
//!
//! - 関連: 第19章「後片付け: Drop ガード + パニックフック」
//!
//! テスト (`mod tests`) は編集しないこと。

#[derive(Debug, Clone, Copy, PartialEq)]
enum Command {
    EnableRawMode,
    DisableRawMode,
    EnterAlternateScreen,
    LeaveAlternateScreen,
    HideCursor,
    ShowCursor,
    EnableMouse,
    DisableMouse,
}

trait Backend {
    fn execute(&mut self, cmd: Command) -> Result<(), String>;
}

// TODO: 打ち消すコマンドを返す
fn inverse(cmd: Command) -> Command {
    todo!()
}

struct TerminalGuard<B: Backend> {
    backend: B,
    done: Vec<Command>, // 成功したセットアップコマンド (restore で逆順に打ち消す)
}

impl<B: Backend> TerminalGuard<B> {
    // TODO: セットアップ。失敗したら成功済みの分を逆順に打ち消して Err
    fn new(mut backend: B, mouse: bool) -> Result<Self, String> {
        todo!()
    }

    // TODO: done を逆順に打ち消す。エラーでも続行。2 回目以降は何もしない
    fn restore(&mut self) {
        todo!()
    }

    fn backend_mut(&mut self) -> &mut B {
        &mut self.backend
    }
}

impl<B: Backend> Drop for TerminalGuard<B> {
    fn drop(&mut self) {
        // TODO: restore を呼ぶ
    }
}

// ---- 偽のバックエンド ----
use std::cell::RefCell;
use std::rc::Rc;

#[derive(Clone, Default)]
struct FakeBackend {
    log: Rc<RefCell<Vec<Command>>>, // 成功したコマンドだけ記録
    fail_on: Vec<Command>,         // これらのコマンドは失敗する
}

impl Backend for FakeBackend {
    fn execute(&mut self, cmd: Command) -> Result<(), String> {
        if self.fail_on.contains(&cmd) {
            return Err(format!("{cmd:?} failed"));
        }
        self.log.borrow_mut().push(cmd);
        Ok(())
    }
}

fn main() {
    let fake = FakeBackend::default();
    let log = fake.log.clone();
    {
        let _g = TerminalGuard::new(fake, true).unwrap();
        println!("アプリ実行中...");
    }
    println!("{:?}", log.borrow());
}

#[cfg(test)]
mod tests {
    use super::*;
    use Command::*;

    #[test]
    fn inverses() {
        for c in [EnableRawMode, EnterAlternateScreen, HideCursor, EnableMouse] {
            assert_ne!(inverse(c), c);
            assert_eq!(inverse(inverse(c)), c);
        }
        assert_eq!(inverse(EnableRawMode), DisableRawMode);
        assert_eq!(inverse(HideCursor), ShowCursor);
    }

    #[test]
    fn setup_and_drop() {
        let fake = FakeBackend::default();
        let log = fake.log.clone();
        {
            let _g = TerminalGuard::new(fake, false).unwrap();
            assert_eq!(*log.borrow(), vec![EnableRawMode, EnterAlternateScreen, HideCursor]);
        }
        assert_eq!(
            *log.borrow(),
            vec![EnableRawMode, EnterAlternateScreen, HideCursor, ShowCursor, LeaveAlternateScreen, DisableRawMode]
        );
    }

    #[test]
    fn mouse_option() {
        let fake = FakeBackend::default();
        let log = fake.log.clone();
        drop(TerminalGuard::new(fake, true).unwrap());
        assert_eq!(
            *log.borrow(),
            vec![
                EnableRawMode, EnterAlternateScreen, HideCursor, EnableMouse,
                DisableMouse, ShowCursor, LeaveAlternateScreen, DisableRawMode
            ]
        );
    }

    #[test]
    fn restore_is_idempotent() {
        let fake = FakeBackend::default();
        let log = fake.log.clone();
        let mut g = TerminalGuard::new(fake, false).unwrap();
        g.restore();
        g.restore();
        drop(g);
        assert_eq!(log.borrow().len(), 6);
    }

    #[test]
    fn rollback_on_setup_failure() {
        let fake = FakeBackend { fail_on: vec![HideCursor], ..Default::default() };
        let log = fake.log.clone();
        let r = TerminalGuard::new(fake, false);
        assert_eq!(r.err(), Some("HideCursor failed".to_string()));
        assert_eq!(
            *log.borrow(),
            vec![EnableRawMode, EnterAlternateScreen, LeaveAlternateScreen, DisableRawMode]
        );
    }

    #[test]
    fn restore_continues_after_error() {
        let fake = FakeBackend { fail_on: vec![ShowCursor], ..Default::default() };
        let log = fake.log.clone();
        drop(TerminalGuard::new(fake, false).unwrap());
        assert_eq!(
            *log.borrow(),
            vec![EnableRawMode, EnterAlternateScreen, HideCursor, LeaveAlternateScreen, DisableRawMode]
        );
    }

    #[test]
    fn restores_on_panic() {
        let fake = FakeBackend::default();
        let log = fake.log.clone();
        let r = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            let mut g = TerminalGuard::new(fake, false).unwrap();
            let _ = g.backend_mut();
            panic!("boom");
        }));
        assert!(r.is_err());
        assert_eq!(log.borrow().last(), Some(&DisableRawMode));
        assert_eq!(log.borrow().len(), 6);
    }
}
