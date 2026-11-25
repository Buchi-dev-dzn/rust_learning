//! 進捗 (progress.json) の読み書き。

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Default, Clone, Serialize, Deserialize)]
pub struct Progress {
    /// 演習 ID → 初回合格時刻 (unix 秒)
    #[serde(default)]
    pub exercises: BTreeMap<String, u64>,
    /// 卒業課題 ID → 初回合格時刻
    #[serde(default)]
    pub finals: BTreeMap<String, u64>,
    /// プロジェクト ID → 完了した STEP 番号
    #[serde(default)]
    pub project_steps: BTreeMap<String, Vec<usize>>,
    /// 読了した章
    #[serde(default)]
    pub lessons_read: BTreeMap<String, u64>,
    /// 実行履歴 (新しい順、最大 100 件)
    #[serde(default)]
    pub history: Vec<HistoryEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryEntry {
    pub kind: String,
    pub id: String,
    pub ok: bool,
    pub at: u64,
}

pub fn now() -> u64 {
    SystemTime::now().duration_since(UNIX_EPOCH).map(|d| d.as_secs()).unwrap_or(0)
}

pub struct Store {
    path: PathBuf,
    pub data: Progress,
}

impl Store {
    pub fn open(root: &Path) -> Store {
        let path = root.join("progress.json");
        let data = fs::read_to_string(&path)
            .ok()
            .and_then(|s| serde_json::from_str(&s).ok())
            .unwrap_or_default();
        Store { path, data }
    }

    pub fn save(&self) {
        if let Ok(s) = serde_json::to_string_pretty(&self.data) {
            let tmp = self.path.with_extension("json.tmp");
            if fs::write(&tmp, s).is_ok() {
                let _ = fs::rename(&tmp, &self.path);
            }
        }
    }

    pub fn record(&mut self, kind: &str, id: &str, ok: bool) {
        let at = now();
        if ok {
            let map = match kind {
                "exercise" => Some(&mut self.data.exercises),
                "final" => Some(&mut self.data.finals),
                _ => None,
            };
            if let Some(map) = map {
                map.entry(id.to_string()).or_insert(at);
            }
        }
        self.data.history.insert(0, HistoryEntry { kind: kind.into(), id: id.into(), ok, at });
        self.data.history.truncate(100);
        self.save();
    }

    pub fn toggle_step(&mut self, project: &str, step: usize, done: bool) {
        let steps = self.data.project_steps.entry(project.to_string()).or_default();
        steps.retain(|s| *s != step);
        if done {
            steps.push(step);
            steps.sort();
        }
        self.save();
    }

    pub fn mark_read(&mut self, chapter: &str) {
        if !self.data.lessons_read.contains_key(chapter) {
            self.data.lessons_read.insert(chapter.to_string(), now());
            self.save();
        }
    }

    pub fn reset(&mut self) {
        self.data = Progress::default();
        self.save();
    }
}
