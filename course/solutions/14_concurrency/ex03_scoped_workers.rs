//! # スレッドの戻り値とスコープ付きスレッド
//!
//! 大きなテキストを分担して処理し、結果を集めよう。
//! `TODO` の部分を直して、すべてのテストを通そう。テストは編集しないこと。
//!
//! - `spawn_square(n)`: 別スレッドで `n * n` を計算し、その `JoinHandle` を返す
//! - `count_matches(lines, pattern, n_threads)`: `lines` を `n_threads` 個程度の塊に分け、
//!   `thread::scope` で並列に「`pattern` を含む行数」を数えて合計する
//!   (`lines` は借用のまま使う。clone しない)
//! - `longest_line(lines, n_threads)`: 同様に並列で、最も長い行 (文字数) を返す。
//!   同じ長さなら先に出てくる行。空なら `None`
//! - `n_threads` は 1 以上と仮定してよい
//! - 関連: 第14章「スコープ付きスレッド thread::scope」

use std::thread::{self, JoinHandle};

pub fn spawn_square(n: u64) -> JoinHandle<u64> {
    thread::spawn(move || n * n)
}

/// n 個程度の塊に分けるときの 1 塊の大きさ (最低 1)
fn chunk_size(len: usize, n_threads: usize) -> usize {
    len.div_ceil(n_threads).max(1)
}

pub fn count_matches(lines: &[&str], pattern: &str, n_threads: usize) -> usize {
    let size = chunk_size(lines.len(), n_threads);
    // scope の中なら、外のローカル変数 (lines, pattern) を借用したまま spawn できる
    thread::scope(|s| {
        let handles: Vec<_> = lines
            .chunks(size)
            .map(|chunk| s.spawn(move || chunk.iter().filter(|l| l.contains(pattern)).count()))
            .collect();
        handles.into_iter().map(|h| h.join().unwrap()).sum()
    })
}

pub fn longest_line<'a>(lines: &[&'a str], n_threads: usize) -> Option<&'a str> {
    let size = chunk_size(lines.len(), n_threads);
    thread::scope(|s| {
        let handles: Vec<_> = lines
            .chunks(size)
            .map(|chunk| {
                s.spawn(move || {
                    // 各塊の中で最長の行 (同じ長さなら先のもの)
                    let mut best: Option<&'a str> = None;
                    for &l in chunk {
                        if best.is_none_or(|b| l.chars().count() > b.chars().count()) {
                            best = Some(l);
                        }
                    }
                    best
                })
            })
            .collect();
        // 塊の順に結果を比べるので「先に出てくる行」が保たれる
        let mut best: Option<&'a str> = None;
        for h in handles {
            if let Some(l) = h.join().unwrap() {
                if best.is_none_or(|b| l.chars().count() > b.chars().count()) {
                    best = Some(l);
                }
            }
        }
        best
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Vec<String> {
        (0..1000).map(|i| if i % 7 == 0 { format!("ERROR {i}") } else { format!("ok {i}") }).collect()
    }

    #[test]
    fn square_in_thread() {
        let handles: Vec<_> = (1..=5).map(spawn_square).collect();
        let results: Vec<u64> = handles.into_iter().map(|h| h.join().unwrap()).collect();
        assert_eq!(results, vec![1, 4, 9, 16, 25]);
    }

    #[test]
    fn count_matches_any_thread_count() {
        let owned = sample();
        let lines: Vec<&str> = owned.iter().map(|s| s.as_str()).collect();
        let expected = lines.iter().filter(|l| l.contains("ERROR")).count();
        for n in [1, 2, 3, 8, 64] {
            assert_eq!(count_matches(&lines, "ERROR", n), expected, "n_threads = {n}");
        }
        assert_eq!(count_matches(&["ab", "b", "a"], "a", 10), 2); // 行数よりスレッド数が多い
        assert_eq!(count_matches(&[], "x", 4), 0);
    }

    #[test]
    fn longest_line_works() {
        let lines = ["a", "abcd", "ab", "wxyz", "abc"];
        for n in 1..=6 {
            assert_eq!(longest_line(&lines, n), Some("abcd"), "n_threads = {n}");
        }
        assert_eq!(longest_line(&[], 3), None);
    }

    #[test]
    fn longest_line_counts_chars() {
        let lines = ["abcdef", "日本語テキスト"]; // 6 文字 / 7 文字 (バイト数では 6 / 21)
        assert_eq!(longest_line(&lines, 2), Some("日本語テキスト"));
    }

    #[test]
    fn result_outlives_scope() {
        let owned = String::from("short\nthe longest line\nmid line");
        let found;
        {
            let lines: Vec<&str> = owned.lines().collect();
            found = longest_line(&lines, 2);
        } // lines (Vec) は破棄されるが、結果は owned から借りている
        assert_eq!(found, Some("the longest line"));
    }
}
