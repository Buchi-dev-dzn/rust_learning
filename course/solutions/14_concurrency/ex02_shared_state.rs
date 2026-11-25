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

use std::sync::{Arc, Mutex};
use std::thread;

pub fn parallel_count(threads: usize, per_thread: usize) -> usize {
    let counter = Arc::new(Mutex::new(0usize));
    let handles: Vec<_> = (0..threads)
        .map(|_| {
            let c = Arc::clone(&counter); // スレッドごとに Arc を複製
            thread::spawn(move || {
                for _ in 0..per_thread {
                    *c.lock().unwrap() += 1; // ロックはこの文の終わりで解放される
                }
            })
        })
        .collect();
    for h in handles {
        h.join().unwrap();
    }
    *counter.lock().unwrap()
}

#[derive(Clone, Default)]
pub struct SharedLog {
    inner: Arc<Mutex<Vec<String>>>,
}

impl SharedLog {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(&self, msg: impl Into<String>) {
        self.inner.lock().unwrap().push(msg.into());
    }

    pub fn len(&self) -> usize {
        self.inner.lock().unwrap().len()
    }

    /// 中身のコピーを返す (ロックを長く持たないため)
    pub fn snapshot(&self) -> Vec<String> {
        self.inner.lock().unwrap().clone()
    }
}

pub fn run_jobs(jobs: Vec<String>, workers: usize) -> Vec<String> {
    let queue = Arc::new(Mutex::new(jobs));
    let results = Arc::new(Mutex::new(Vec::new()));
    let mut handles = Vec::new();
    for _ in 0..workers {
        let queue = Arc::clone(&queue);
        let results = Arc::clone(&results);
        handles.push(thread::spawn(move || {
            loop {
                // pop した直後にロックを手放す (処理中はロックを持たない)
                let job = queue.lock().unwrap().pop();
                let Some(job) = job else { break };
                let done = job.to_uppercase();
                results.lock().unwrap().push(done);
            }
        }));
    }
    for h in handles {
        h.join().unwrap();
    }
    let mut out = results.lock().unwrap().clone();
    out.sort();
    out
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
