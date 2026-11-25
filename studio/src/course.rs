//! コース定義 (course/course.toml) とファイル規約からの自動検出。
//! 教材の追加がすぐ反映されるよう、リクエストごとに読み直す (ファイルは小さいので十分速い)。

use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, Deserialize)]
pub struct Stage {
    pub id: u32,
    pub title: String,
    pub summary: String,
    pub goal: String,
    pub hours: String,
    pub chapters: Vec<String>,
    #[serde(default)]
    pub projects: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Chapter {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub hours: String,
    /// 章番号 (1 始まり) — ロード時に設定
    #[serde(skip)]
    pub number: usize,
    #[serde(skip)]
    pub stage: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Project {
    pub id: String,
    pub title: String,
    pub difficulty: u32,
    #[serde(default)]
    pub chapters: Vec<String>,
    pub summary: String,
    pub run: String,
    #[serde(default)]
    pub tui: bool,
    #[serde(skip)]
    pub stage: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Final {
    pub id: String,
    pub title: String,
    pub difficulty: u32,
    pub tech: String,
    pub tested: String,
    #[serde(default)]
    pub tui: bool,
}

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct Exercise {
    /// `<chapter>/<name>` (例: `02_ownership/ex01_move`)
    pub id: String,
    pub chapter: String,
    pub name: String,
    pub title: String,
    /// 課題文 (Markdown)
    pub description: String,
    /// 章内での番号 (1 始まり)
    pub index: usize,
    pub chapter_number: usize,
}

#[derive(Debug, Deserialize)]
struct CourseFile {
    stage: Vec<Stage>,
    chapter: Vec<Chapter>,
    #[serde(default)]
    project: Vec<Project>,
    #[serde(default, rename = "final")]
    finals: Vec<Final>,
}

#[derive(Debug, Clone)]
pub struct Course {
    pub stages: Vec<Stage>,
    pub chapters: Vec<Chapter>,
    pub projects: Vec<Project>,
    pub finals: Vec<Final>,
    pub exercises: Vec<Exercise>,
}

impl Course {
    pub fn load(root: &Path) -> Result<Course, String> {
        let path = root.join("course/course.toml");
        let text = fs::read_to_string(&path).map_err(|e| format!("{}: {e}", path.display()))?;
        let file: CourseFile = toml::from_str(&text).map_err(|e| format!("course.toml: {e}"))?;
        let mut chapters = file.chapter;
        let mut projects = file.project;
        // 章番号とステージをステージ定義の順に振る
        let mut number = 0;
        for stage in &file.stage {
            for cid in &stage.chapters {
                if let Some(ch) = chapters.iter_mut().find(|c| &c.id == cid) {
                    number += 1;
                    ch.number = number;
                    ch.stage = stage.id;
                }
            }
            for pid in &stage.projects {
                if let Some(p) = projects.iter_mut().find(|p| &p.id == pid) {
                    p.stage = stage.id;
                }
            }
        }
        chapters.sort_by_key(|c| c.number);

        let mut exercises = Vec::new();
        for ch in &chapters {
            let dir = root.join("course/exercises").join(&ch.id);
            let mut files: Vec<PathBuf> = match fs::read_dir(&dir) {
                Ok(rd) => rd
                    .filter_map(|e| e.ok().map(|e| e.path()))
                    .filter(|p| p.extension().is_some_and(|x| x == "rs"))
                    .collect(),
                Err(_) => Vec::new(),
            };
            files.sort();
            for (i, f) in files.iter().enumerate() {
                let name = f.file_stem().unwrap().to_string_lossy().to_string();
                let src = fs::read_to_string(f).unwrap_or_default();
                let (title, description) = parse_header(&src, &name);
                exercises.push(Exercise {
                    id: format!("{}/{}", ch.id, name),
                    chapter: ch.id.clone(),
                    name,
                    title,
                    description,
                    index: i + 1,
                    chapter_number: ch.number,
                });
            }
        }
        Ok(Course {
            stages: file.stage,
            chapters,
            projects,
            finals: file.finals,
            exercises,
        })
    }

    pub fn chapter(&self, id: &str) -> Option<&Chapter> {
        self.chapters.iter().find(|c| c.id == id)
    }
    pub fn stage(&self, id: u32) -> Option<&Stage> {
        self.stages.iter().find(|s| s.id == id)
    }
    pub fn exercise(&self, id: &str) -> Option<&Exercise> {
        self.exercises.iter().find(|e| e.id == id)
    }
    pub fn project(&self, id: &str) -> Option<&Project> {
        self.projects.iter().find(|p| p.id == id)
    }
    pub fn final_(&self, id: &str) -> Option<&Final> {
        self.finals.iter().find(|f| f.id == id)
    }
    pub fn exercises_of(&self, chapter: &str) -> Vec<Exercise> {
        self.exercises.iter().filter(|e| e.chapter == chapter).cloned().collect()
    }
}

/// 先頭の `//!` ブロックから (タイトル, 本文 Markdown) を取り出す
pub fn parse_header(src: &str, fallback: &str) -> (String, String) {
    let mut lines = Vec::new();
    for line in src.lines() {
        if let Some(rest) = line.strip_prefix("//!") {
            lines.push(rest.strip_prefix(' ').unwrap_or(rest).to_string());
        } else if line.trim().is_empty() && lines.is_empty() {
            continue;
        } else {
            break;
        }
    }
    let mut title = fallback.to_string();
    if let Some(first) = lines.first() {
        if let Some(t) = first.strip_prefix("# ") {
            title = t.trim().to_string();
            lines.remove(0);
        }
    }
    (title, lines.join("\n").trim().to_string())
}

/// STEPS.md / SPEC.md などを (タイトル, 前書き, [(見出し, 本文)]) に分割する。
/// `## STEP` で始まる見出しごとに区切る。
pub fn split_steps(md: &str) -> (String, String, Vec<(String, String)>) {
    let mut title = String::new();
    let mut intro = String::new();
    let mut steps: Vec<(String, String)> = Vec::new();
    let mut in_code = false;
    for line in md.lines() {
        if line.trim_start().starts_with("```") {
            in_code = !in_code;
        }
        if !in_code && title.is_empty() && line.starts_with("# ") {
            title = line[2..].trim().to_string();
            continue;
        }
        if !in_code && line.starts_with("## ") && line[3..].trim_start().to_uppercase().starts_with("STEP") {
            steps.push((line[3..].trim().to_string(), String::new()));
            continue;
        }
        let target = match steps.last_mut() {
            Some((_, body)) => body,
            None => &mut intro,
        };
        target.push_str(line);
        target.push('\n');
    }
    (title, intro, steps)
}

/// `---` だけの行で区切られたヒント
pub fn split_hints(md: &str) -> Vec<String> {
    let mut out = vec![String::new()];
    for line in md.lines() {
        if line.trim() == "---" {
            out.push(String::new());
        } else {
            let last = out.last_mut().unwrap();
            last.push_str(line);
            last.push('\n');
        }
    }
    out.into_iter().map(|s| s.trim().to_string()).filter(|s| !s.is_empty()).collect()
}

/// Markdown の 1 行目 `# ...` を取り出す
pub fn md_title(md: &str) -> Option<String> {
    md.lines().find(|l| l.starts_with("# ")).map(|l| l[2..].trim().to_string())
}

/// 先頭の `# タイトル` 行を取り除く (ページ側で見出しを出すため)
pub fn strip_title(md: &str) -> String {
    let mut done = false;
    let mut out = String::new();
    for line in md.lines() {
        if !done && line.starts_with("# ") {
            done = true;
            continue;
        }
        out.push_str(line);
        out.push('\n');
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn header_parse() {
        let src = "//! # タイトル\n//!\n//! 本文\nfn main() {}\n";
        let (t, d) = parse_header(src, "x");
        assert_eq!(t, "タイトル");
        assert_eq!(d, "本文");
    }

    #[test]
    fn steps_split() {
        let md = "# P\nintro\n## STEP 1: a\nA\n```\n## STEP no\n```\n## STEP 2: b\nB\n";
        let (t, i, s) = split_steps(md);
        assert_eq!(t, "P");
        assert_eq!(i.trim(), "intro");
        assert_eq!(s.len(), 2);
        assert!(s[0].1.contains("## STEP no"));
    }

    #[test]
    fn hints_split() {
        assert_eq!(split_hints("a\n---\nb\n"), vec!["a", "b"]);
    }
}
