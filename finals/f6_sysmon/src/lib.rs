//! F6: htop 風システムモニタの状態ロジック。
//!
//! 仕様は SPEC.md。`todo!()` を埋めて `cargo test -p f6_sysmon` を通そう。
//! 型と公開メソッドのシグネチャはテストが依存しているので変えないこと。
//! (非公開フィールドや補助関数は自由に変えてよい)

use std::collections::VecDeque;

#[derive(Debug, Clone, PartialEq)]
pub struct ProcessInfo {
    pub pid: u32,
    pub cpu: f32,
    pub mem: f32,
    pub command: String,
}

impl ProcessInfo {
    pub fn new(pid: u32, cpu: f32, mem: f32, command: &str) -> ProcessInfo {
        ProcessInfo {
            pid,
            cpu,
            mem,
            command: command.to_string(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct Snapshot {
    pub processes: Vec<ProcessInfo>,
}

impl Snapshot {
    pub fn total_cpu(&self) -> f32 {
        // TODO
        todo!()
    }
}

/// `ps -axo pid,pcpu,pmem,comm` の出力を解析する (SPEC 2章)
pub fn parse_ps(output: &str) -> Vec<ProcessInfo> {
    // TODO
    todo!()
}

/// 固定長のリングバッファ (SPEC 3章)
#[derive(Debug, Clone)]
pub struct History {
    cap: usize,
    data: VecDeque<f32>,
}

impl History {
    pub fn new(capacity: usize) -> History {
        todo!()
    }
    pub fn push(&mut self, v: f32) {
        todo!()
    }
    pub fn len(&self) -> usize {
        todo!()
    }
    pub fn is_empty(&self) -> bool {
        todo!()
    }
    pub fn capacity(&self) -> usize {
        todo!()
    }
    pub fn values(&self) -> Vec<f32> {
        todo!()
    }
    pub fn latest(&self) -> Option<f32> {
        todo!()
    }
    pub fn max(&self) -> f32 {
        todo!()
    }
    pub fn sparkline_data(&self) -> Vec<u64> {
        todo!()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SortKey {
    Pid,
    Cpu,
    Mem,
    Command,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Normal,
    Filter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Input {
    Char(char),
    Up,
    Down,
    PageUp,
    PageDown,
    Home,
    End,
    Enter,
    Esc,
    Backspace,
}

#[derive(Debug, Clone)]
pub struct App {
    processes: Vec<ProcessInfo>,
    history: History,
    sort_key: SortKey,
    sort_desc: bool,
    filter: String,
    mode: Mode,
    paused: bool,
    should_quit: bool,
    selected: usize,
    offset: usize,
}

impl App {
    pub fn new(history_capacity: usize) -> App {
        // TODO: 初期状態は SPEC 4.1
        todo!()
    }

    // ---- 読み取り用 (このままでよい)
    pub fn history(&self) -> &History {
        &self.history
    }
    pub fn sort_key(&self) -> SortKey {
        self.sort_key
    }
    pub fn sort_desc(&self) -> bool {
        self.sort_desc
    }
    pub fn filter(&self) -> &str {
        &self.filter
    }
    pub fn mode(&self) -> Mode {
        self.mode
    }
    pub fn paused(&self) -> bool {
        self.paused
    }
    pub fn should_quit(&self) -> bool {
        self.should_quit
    }
    pub fn selected(&self) -> usize {
        self.selected
    }
    pub fn process_count(&self) -> usize {
        self.processes.len()
    }

    // ---- 一覧 (SPEC 4.2)
    pub fn visible(&self) -> Vec<&ProcessInfo> {
        todo!()
    }

    pub fn selected_process(&self) -> Option<&ProcessInfo> {
        todo!()
    }

    // ---- 状態更新 (SPEC 4.3)
    pub fn update(&mut self, snapshot: Snapshot) {
        todo!()
    }

    pub fn set_sort(&mut self, key: SortKey) {
        todo!()
    }

    pub fn set_filter(&mut self, filter: &str) {
        todo!()
    }

    // ---- 選択とスクロール (SPEC 4.4)
    pub fn select_next(&mut self) {
        todo!()
    }
    pub fn select_prev(&mut self) {
        todo!()
    }
    pub fn page_down(&mut self, n: usize) {
        todo!()
    }
    pub fn page_up(&mut self, n: usize) {
        todo!()
    }
    pub fn select_first(&mut self) {
        todo!()
    }
    pub fn select_last(&mut self) {
        todo!()
    }
    pub fn scroll_offset(&mut self, height: usize) -> usize {
        todo!()
    }

    // ---- キー入力 (SPEC 4.5)
    pub fn handle_input(&mut self, input: Input, page_size: usize) {
        todo!()
    }
}
