//! F6: htop 風システムモニタの状態ロジック (模範解答)。
//!
//! 端末にもプロセス取得にも依存しない「状態 + 状態遷移」だけをここに置く。
//! データは Snapshot として外から注入するので、テストでは好きなデータを流し込める。

use std::cmp::Ordering;
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
        self.processes.iter().map(|p| p.cpu).sum()
    }
}

/// `ps -axo pid,pcpu,pmem,comm` の出力を解析する
pub fn parse_ps(output: &str) -> Vec<ProcessInfo> {
    fn next_field(s: &str) -> Option<(&str, &str)> {
        let s = s.trim_start();
        if s.is_empty() {
            return None;
        }
        let end = s.find(char::is_whitespace).unwrap_or(s.len());
        Some((&s[..end], &s[end..]))
    }
    let mut out = Vec::new();
    for line in output.lines() {
        let Some((pid, rest)) = next_field(line) else { continue };
        let Some((cpu, rest)) = next_field(rest) else { continue };
        let Some((mem, rest)) = next_field(rest) else { continue };
        let command = rest.trim();
        let (Ok(pid), Ok(cpu), Ok(mem)) = (pid.parse::<u32>(), cpu.parse::<f32>(), mem.parse::<f32>())
        else {
            continue;
        };
        if command.is_empty() {
            continue;
        }
        out.push(ProcessInfo::new(pid, cpu, mem, command));
    }
    out
}

/// 固定長のリングバッファ。容量を超えたら古いものから捨てる
#[derive(Debug, Clone)]
pub struct History {
    cap: usize,
    data: VecDeque<f32>,
}

impl History {
    pub fn new(capacity: usize) -> History {
        History {
            cap: capacity,
            data: VecDeque::with_capacity(capacity),
        }
    }
    pub fn push(&mut self, v: f32) {
        if self.cap == 0 {
            return;
        }
        if self.data.len() == self.cap {
            self.data.pop_front();
        }
        self.data.push_back(v);
    }
    pub fn len(&self) -> usize {
        self.data.len()
    }
    pub fn is_empty(&self) -> bool {
        self.data.is_empty()
    }
    pub fn capacity(&self) -> usize {
        self.cap
    }
    pub fn values(&self) -> Vec<f32> {
        self.data.iter().copied().collect()
    }
    pub fn latest(&self) -> Option<f32> {
        self.data.back().copied()
    }
    pub fn max(&self) -> f32 {
        self.data.iter().copied().reduce(f32::max).unwrap_or(0.0)
    }
    pub fn sparkline_data(&self) -> Vec<u64> {
        self.data.iter().map(|v| v.max(0.0).round() as u64).collect()
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
        App {
            processes: Vec::new(),
            history: History::new(history_capacity),
            sort_key: SortKey::Cpu,
            sort_desc: true,
            filter: String::new(),
            mode: Mode::Normal,
            paused: false,
            should_quit: false,
            selected: 0,
            offset: 0,
        }
    }

    // ------------------------------------------------------------ getters
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

    // ------------------------------------------------------------ list
    pub fn visible(&self) -> Vec<&ProcessInfo> {
        let needle = self.filter.to_lowercase();
        let mut v: Vec<&ProcessInfo> = self
            .processes
            .iter()
            .filter(|p| needle.is_empty() || p.command.to_lowercase().contains(&needle))
            .collect();
        let key = self.sort_key;
        let desc = self.sort_desc;
        v.sort_by(|a, b| {
            let ord = match key {
                SortKey::Pid => a.pid.cmp(&b.pid),
                SortKey::Cpu => a.cpu.partial_cmp(&b.cpu).unwrap_or(Ordering::Equal),
                SortKey::Mem => a.mem.partial_cmp(&b.mem).unwrap_or(Ordering::Equal),
                SortKey::Command => a.command.cmp(&b.command),
            };
            let ord = if desc { ord.reverse() } else { ord };
            ord.then(a.pid.cmp(&b.pid))
        });
        v
    }

    pub fn selected_process(&self) -> Option<&ProcessInfo> {
        self.visible().get(self.selected).copied()
    }

    /// 表示リストが変わる操作の前後で呼ぶ: 同じ pid があればそれを、なければ同じ index (丸め) を選ぶ
    fn with_reselect(&mut self, f: impl FnOnce(&mut App)) {
        let old_pid = self.selected_process().map(|p| p.pid);
        let old_index = self.selected;
        f(self);
        let vis = self.visible();
        self.selected = match old_pid.and_then(|pid| vis.iter().position(|p| p.pid == pid)) {
            Some(i) => i,
            None => old_index.min(vis.len().saturating_sub(1)),
        };
    }

    pub fn update(&mut self, snapshot: Snapshot) {
        if self.paused {
            return;
        }
        self.history.push(snapshot.total_cpu());
        self.with_reselect(|app| app.processes = snapshot.processes);
    }

    pub fn set_sort(&mut self, key: SortKey) {
        self.with_reselect(|app| {
            if app.sort_key == key {
                app.sort_desc = !app.sort_desc;
            } else {
                app.sort_key = key;
                app.sort_desc = matches!(key, SortKey::Cpu | SortKey::Mem);
            }
        });
    }

    pub fn set_filter(&mut self, filter: &str) {
        let f = filter.to_string();
        self.with_reselect(|app| app.filter = f);
    }

    // ------------------------------------------------------------ selection
    fn last_index(&self) -> usize {
        self.visible().len().saturating_sub(1)
    }
    pub fn select_next(&mut self) {
        self.selected = self.selected.saturating_add(1).min(self.last_index());
    }
    pub fn select_prev(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }
    pub fn page_down(&mut self, n: usize) {
        self.selected = self.selected.saturating_add(n).min(self.last_index());
    }
    pub fn page_up(&mut self, n: usize) {
        self.selected = self.selected.saturating_sub(n);
    }
    pub fn select_first(&mut self) {
        self.selected = 0;
    }
    pub fn select_last(&mut self) {
        self.selected = self.last_index();
    }

    pub fn scroll_offset(&mut self, height: usize) -> usize {
        if height == 0 {
            return self.offset;
        }
        if self.selected < self.offset {
            self.offset = self.selected;
        } else if self.selected >= self.offset.saturating_add(height) {
            self.offset = self.selected + 1 - height;
        }
        self.offset
    }

    // ------------------------------------------------------------ input
    pub fn handle_input(&mut self, input: Input, page_size: usize) {
        match input {
            Input::Up => return self.select_prev(),
            Input::Down => return self.select_next(),
            Input::PageUp => return self.page_up(page_size),
            Input::PageDown => return self.page_down(page_size),
            Input::Home => return self.select_first(),
            Input::End => return self.select_last(),
            _ => {}
        }
        match self.mode {
            Mode::Filter => match input {
                Input::Char(c) => {
                    let mut f = self.filter.clone();
                    f.push(c);
                    self.set_filter(&f);
                }
                Input::Backspace => {
                    let mut f = self.filter.clone();
                    f.pop();
                    self.set_filter(&f);
                }
                Input::Enter => self.mode = Mode::Normal,
                Input::Esc => {
                    self.set_filter("");
                    self.mode = Mode::Normal;
                }
                _ => {}
            },
            Mode::Normal => match input {
                Input::Char('q') => self.should_quit = true,
                Input::Char('j') => self.select_next(),
                Input::Char('k') => self.select_prev(),
                Input::Char('g') => self.select_first(),
                Input::Char('G') => self.select_last(),
                Input::Char('c') => self.set_sort(SortKey::Cpu),
                Input::Char('m') => self.set_sort(SortKey::Mem),
                Input::Char('p') => self.set_sort(SortKey::Pid),
                Input::Char('n') => self.set_sort(SortKey::Command),
                Input::Char('/') => self.mode = Mode::Filter,
                Input::Char(' ') => self.paused = !self.paused,
                Input::Esc => self.set_filter(""),
                _ => {}
            },
        }
    }
}
