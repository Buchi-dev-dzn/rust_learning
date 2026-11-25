//! # Arc<Mutex<T>> で状態を共有する
//!
//! 複数のスレッドから同じデータを安全に読み書きしよう。
//! `TODO` の部分を直して、すべてのテストを通そう。テストは編集しないこと。
//!
//! - `parallel_count(threads, per_thread)`: `threads` 本のスレッドがそれぞれ
//!   `per_thread` 回、共有カウンタを 1 増やす。最終値を返す
//! - `SharedLog`: clone しても同じ `Vec<String>` を共有するスレッド安全なログ
//! - `run_jobs(jobs, workers)`: 共有キューから `workers` 本のスレッドがジョブを取り出し、
//!   大文字にした結果を共有の結果リストに入れる。結果をソートして返す
//! - 関連: 第14章「共有状態: Arc<Mutex<T>>」

use std::rc::Rc;
use std::cell::RefCell;
use std::sync::{Arc, Mutex};
use std::thread;

pub fn parallel_count(threads: usize, per_thread: usize) -> usize {
    // TODO: Arc::new(Mutex::new(0usize)) を作り、スレッドごとに Arc::clone して move で渡す。
    //       全スレッドを join してから値を返す
    todo!()
}

// TODO: Rc<RefCell<..>> はスレッド間で共有できない (Send でない)。
//       Arc<Mutex<..>> に変えて、メソッドも lock() を使うように書き換える
#[derive(Clone, Default)]
pub struct SharedLog {
    inner: Rc<RefCell<Vec<String>>>,
}

impl SharedLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&self, msg: impl Into<String>) {
        self.inner.borrow_mut().push(msg.into());
    }

    pub fn len(&self) -> usize {
        self.inner.borrow().len()
    }

    /// 中身のコピーを返す (ロックを長く持たないため)
    pub fn snapshot(&self) -> Vec<String> {
        self.inner.borrow().clone()
    }
}

pub fn run_jobs(jobs: Vec<String>, workers: usize) -> Vec<String> {
    // TODO:
    // - queue: Arc<Mutex<Vec<String>>> (jobs を入れる)
    // - results: Arc<Mutex<Vec<String>>>
    // - workers 本のスレッドが loop で queue から pop し、なくなったら break
    //   (pop したらすぐロックを手放すこと。ロックを持ったまま処理しない)
    // - 全員 join したら results をソートして返す
    todo!()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn counts_without_losing_updates() {
        assert_eq!(parallel_count(1, 10), 10);
        assert_eq!(parallel_count(8, 1000), 8000);
        assert_eq!(parallel_count(0, 5), 0);
    }

    #[test]
    fn shared_log_across_threads() {
        let log = SharedLog::new();
        let handles: Vec<_> = (0..4)
            .map(|i| {
                let log = log.clone();
                thread::spawn(move || {
                    for j in 0..25 {
                        log.push(format!("t{i}-{j}"));
                    }
                })
            })
            .collect();
        for h in handles {
            h.join().unwrap();
        }
        assert_eq!(log.len(), 100);
        let snap = log.snapshot();
        assert!(snap.contains(&"t3-24".to_string()));
        // スレッド内の順番は保たれる
        let t0: Vec<&String> = snap.iter().filter(|s| s.starts_with("t0-")).collect();
        assert_eq!(t0.first().map(|s| s.as_str()), Some("t0-0"));
        assert_eq!(t0.last().map(|s| s.as_str()), Some("t0-24"));
    }

    #[test]
    fn jobs_are_all_processed_once() {
        let jobs: Vec<String> = ["ls", "cat", "grep", "wc", "sort"].iter().map(|s| s.to_string()).collect();
        assert_eq!(run_jobs(jobs, 3), vec!["CAT", "GREP", "LS", "SORT", "WC"]);
    }

    #[test]
    fn many_jobs() {
        let jobs: Vec<String> = (0..500).map(|i| format!("job{i:03}")).collect();
        let out = run_jobs(jobs, 8);
        assert_eq!(out.len(), 500);
        assert_eq!(out[0], "JOB000");
        assert_eq!(out[499], "JOB499");
    }

    #[test]
    fn no_jobs() {
        assert_eq!(run_jobs(vec![], 4), Vec::<String>::new());
    }
}
