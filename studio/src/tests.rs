//! 教材そのものの品質チェック (開発者向け)。`cargo test -p studio` で実行する。

use super::*;
use std::process::Command;

fn root() -> PathBuf {
    repo_root()
}

fn rustc_test(file: &FsPath, tag: &str) -> bool {
    let out_dir = std::env::temp_dir().join("studio-selftest");
    fs::create_dir_all(&out_dir).unwrap();
    let bin = out_dir.join(tag.replace('/', "__"));
    let ok = Command::new("rustc")
        .args(["--edition", "2024", "--test", "-A", "warnings"])
        .arg(file)
        .arg("-o")
        .arg(&bin)
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    ok && Command::new(&bin).arg("--test-threads=1").output().map(|o| o.status.success()).unwrap_or(false)
}

#[test]
fn course_toml_loads_and_is_consistent() {
    let c = course::Course::load(&root()).expect("course.toml");
    assert!(!c.stages.is_empty());
    for s in &c.stages {
        for ch in &s.chapters {
            assert!(c.chapter(ch).is_some(), "stage {} の章 {ch} が [[chapter]] にない", s.id);
        }
        for p in &s.projects {
            assert!(c.project(p).is_some(), "stage {} のプロジェクト {p} がない", s.id);
        }
    }
    for ch in &c.chapters {
        assert!(ch.number > 0, "章 {} がどのステージにも属していない", ch.id);
        assert!(root().join("docs").join(format!("{}.md", ch.id)).exists(), "docs/{}.md がない", ch.id);
    }
}

#[test]
fn every_exercise_has_solution_and_hints() {
    let c = course::Course::load(&root()).unwrap();
    assert!(c.exercises.len() >= 30, "演習が少なすぎる: {}", c.exercises.len());
    for e in &c.exercises {
        assert!(root().join("course/solutions").join(format!("{}.rs", e.id)).exists(), "解答なし: {}", e.id);
        assert!(root().join("course/hints").join(format!("{}.md", e.id)).exists(), "ヒントなし: {}", e.id);
        assert!(!e.title.is_empty() && e.title != e.name, "タイトル (//! # ...) なし: {}", e.id);
    }
}

#[test]
fn solutions_pass_and_pristine_exercises_fail() {
    let r = root();
    let c = course::Course::load(&r).unwrap();
    let mut bad = Vec::new();
    for e in &c.exercises {
        let sol = r.join("course/solutions").join(format!("{}.rs", e.id));
        let ex = r.join("course/exercises").join(format!("{}.rs", e.id));
        if !rustc_test(&sol, &format!("sol_{}", e.id)) {
            bad.push(format!("解答が不合格: {}", e.id));
        }
        if rustc_test(&ex, &format!("ex_{}", e.id)) {
            bad.push(format!("初期状態で合格してしまう: {}", e.id));
        }
    }
    assert!(bad.is_empty(), "{}", bad.join("\n"));
}

#[test]
fn projects_and_finals_have_docs() {
    let r = root();
    let c = course::Course::load(&r).unwrap();
    for p in &c.projects {
        let md = fs::read_to_string(r.join("projects").join(&p.id).join("STEPS.md")).unwrap_or_default();
        let (_, _, steps) = course::split_steps(&md);
        assert!(steps.len() >= 3, "{} の STEP が少ない", p.id);
        assert!(r.join("projects").join(&p.id).join("starter/Cargo.toml").exists());
    }
    for f in &c.finals {
        let d = r.join("finals").join(&f.id);
        assert!(d.join("SPEC.md").exists(), "{} SPEC.md", f.id);
        assert!(d.join("tests/acceptance.rs").exists(), "{} acceptance", f.id);
        assert!(r.join("course/finals").join(&f.id).join("solution/src").exists(), "{} solution", f.id);
        assert!(r.join("course/finals").join(&f.id).join("starter/src").exists(), "{} starter", f.id);
    }
}

#[test]
fn editable_paths() {
    assert!(editable("exercises/01_basics/ex01_x.rs"));
    assert!(editable("finals/f1_coreutils/src/lib.rs"));
    assert!(editable("finals/f1_coreutils/src/bin/x.rs"));
    assert!(editable("projects/p05_editor/starter/src/main.rs"));
    assert!(!editable("projects/p05_editor/src/main.rs"));
    assert!(!editable("finals/f1_coreutils/tests/acceptance.rs"));
    assert!(!editable("exercises/../studio/src/main.rs"));
    assert!(!editable("/etc/passwd"));
    assert!(!editable("exercises/01_basics/x.toml"));
}
