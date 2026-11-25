//! HTML ページ (askama テンプレート) のビューモデルとハンドラ。

use crate::AppState;
use crate::course::{self, Course};
use crate::markdown;
use askama::Template;
use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use serde::Deserialize;
use std::fs;

// ------------------------------------------------------------------ 共通部品

#[derive(Clone)]
pub struct Link {
    pub href: String,
    pub label: String,
}

fn link(href: impl Into<String>, label: impl Into<String>) -> Link {
    Link { href: href.into(), label: label.into() }
}

pub struct PageMeta {
    pub title: String,
    pub nav: &'static str,
    pub crumbs: Vec<Link>,
    pub prev: Option<Link>,
    pub next: Option<Link>,
    pub pct: u32,
    pub wide: bool,
}

impl PageMeta {
    fn new(st: &AppState, title: impl Into<String>, nav: &'static str) -> PageMeta {
        PageMeta {
            title: title.into(),
            nav,
            crumbs: vec![link("/", "ホーム")],
            prev: None,
            next: None,
            pct: overall_pct(st),
            wide: false,
        }
    }
    fn crumb(mut self, href: impl Into<String>, label: impl Into<String>) -> Self {
        self.crumbs.push(link(href, label));
        self
    }
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct ExView {
    pub id: String,
    pub href: String,
    pub title: String,
    pub label: String,
    pub chapter_title: String,
    pub done: bool,
}

#[allow(dead_code)]
#[derive(Clone)]
pub struct ChapterView {
    pub id: String,
    pub number: usize,
    pub title: String,
    pub hours: String,
    pub href: String,
    pub read: bool,
    pub ex_total: usize,
    pub ex_done: usize,
    pub complete: bool,
    pub exercises: Vec<ExView>,
}

#[derive(Clone)]
pub struct ProjView {
    pub id: String,
    pub title: String,
    pub summary: String,
    pub stars: String,
    pub href: String,
    pub chapters: String,
    pub steps_total: usize,
    pub steps_done: usize,
    pub tui: bool,
}

#[derive(Clone)]
pub struct FinalView {
    pub id: String,
    pub title: String,
    pub stars: String,
    pub tech: String,
    pub tested: String,
    pub href: String,
    pub done: bool,
    pub tui: bool,
}

pub struct StageView {
    pub id: u32,
    pub title: String,
    pub summary: String,
    pub goal: String,
    pub hours: String,
    pub href: String,
    pub pct: u32,
    pub chapters: Vec<ChapterView>,
    pub projects: Vec<ProjView>,
    pub finals: Vec<FinalView>,
}

pub struct HistView {
    pub label: String,
    pub href: String,
    pub ok: bool,
    pub at: u64,
}

fn load(st: &AppState) -> Result<Course, Response> {
    Course::load(&st.root).map_err(|e| {
        (StatusCode::INTERNAL_SERVER_ERROR, Html(format!("<h1>course.toml の読み込みに失敗</h1><pre>{}</pre>", markdown::escape(&e))))
            .into_response()
    })
}

fn render<T: Template>(t: T) -> Response {
    match t.render() {
        Ok(s) => Html(s).into_response(),
        Err(e) => (StatusCode::INTERNAL_SERVER_ERROR, format!("template error: {e}")).into_response(),
    }
}

pub fn not_found(st: &AppState, what: &str) -> Response {
    let page = PageMeta::new(st, "見つかりません", "");
    (StatusCode::NOT_FOUND, render(NotFoundTpl { page, what: what.to_string() })).into_response()
}

fn stars(n: u32) -> String {
    (0..5).map(|i| if i < n { '★' } else { '☆' }).collect()
}

fn ex_view(st: &AppState, c: &Course, e: &course::Exercise) -> ExView {
    let done = st.store.lock().unwrap().data.exercises.contains_key(&e.id);
    ExView {
        id: e.id.clone(),
        href: format!("/exercises/{}", e.id),
        title: e.title.clone(),
        label: format!("{}-{}", e.chapter_number, e.index),
        chapter_title: c.chapter(&e.chapter).map(|c| c.title.clone()).unwrap_or_default(),
        done,
    }
}

fn chapter_view(st: &AppState, c: &Course, ch: &course::Chapter) -> ChapterView {
    let exercises: Vec<ExView> = c.exercises_of(&ch.id).iter().map(|e| ex_view(st, c, e)).collect();
    let ex_done = exercises.iter().filter(|e| e.done).count();
    let read = st.store.lock().unwrap().data.lessons_read.contains_key(&ch.id);
    ChapterView {
        id: ch.id.clone(),
        number: ch.number,
        title: ch.title.clone(),
        hours: ch.hours.clone(),
        href: format!("/lessons/{}", ch.id),
        read,
        ex_total: exercises.len(),
        ex_done,
        complete: ex_done == exercises.len() && (read || !exercises.is_empty()),
        exercises,
    }
}

fn project_steps(st: &AppState, id: &str) -> (String, String, Vec<(String, String)>) {
    let md = fs::read_to_string(st.root.join("projects").join(id).join("STEPS.md")).unwrap_or_default();
    course::split_steps(&md)
}

fn proj_view(st: &AppState, c: &Course, p: &course::Project) -> ProjView {
    let (_, _, steps) = project_steps(st, &p.id);
    let done = st.store.lock().unwrap().data.project_steps.get(&p.id).map(|v| v.len()).unwrap_or(0);
    ProjView {
        id: p.id.clone(),
        title: p.title.clone(),
        summary: p.summary.clone(),
        stars: stars(p.difficulty),
        href: format!("/projects/{}", p.id),
        chapters: p
            .chapters
            .iter()
            .filter_map(|id| c.chapter(id).map(|ch| format!("第{}章", ch.number)))
            .collect::<Vec<_>>()
            .join(" / "),
        steps_total: steps.len(),
        steps_done: done.min(steps.len()),
        tui: p.tui,
    }
}

fn final_view(st: &AppState, f: &course::Final) -> FinalView {
    let done = st.store.lock().unwrap().data.finals.contains_key(&f.id);
    FinalView {
        id: f.id.clone(),
        title: f.title.clone(),
        stars: stars(f.difficulty),
        tech: f.tech.clone(),
        tested: f.tested.clone(),
        href: format!("/final/{}", f.id),
        done,
        tui: f.tui,
    }
}

fn stage_view(st: &AppState, c: &Course, s: &course::Stage) -> StageView {
    let chapters: Vec<ChapterView> =
        s.chapters.iter().filter_map(|id| c.chapter(id)).map(|ch| chapter_view(st, c, ch)).collect();
    let projects: Vec<ProjView> =
        s.projects.iter().filter_map(|id| c.project(id)).map(|p| proj_view(st, c, p)).collect();
    let finals: Vec<FinalView> =
        if s.id == 5 { c.finals.iter().map(|f| final_view(st, f)).collect() } else { vec![] };
    let total: usize = chapters.iter().map(|c| c.ex_total).sum::<usize>() + finals.len();
    let done: usize = chapters.iter().map(|c| c.ex_done).sum::<usize>() + finals.iter().filter(|f| f.done).count();
    StageView {
        id: s.id,
        title: s.title.clone(),
        summary: s.summary.clone(),
        goal: s.goal.clone(),
        hours: s.hours.clone(),
        href: format!("/stages/{}", s.id),
        pct: pct(done, total),
        chapters,
        projects,
        finals,
    }
}

fn pct(done: usize, total: usize) -> u32 {
    if total == 0 { 0 } else { (done * 100 / total) as u32 }
}

pub fn overall_pct(st: &AppState) -> u32 {
    let Ok(c) = Course::load(&st.root) else { return 0 };
    let p = st.store.lock().unwrap();
    let done = c.exercises.iter().filter(|e| p.data.exercises.contains_key(&e.id)).count()
        + c.finals.iter().filter(|f| p.data.finals.contains_key(&f.id)).count();
    pct(done, c.exercises.len() + c.finals.len())
}

fn next_exercise(st: &AppState, c: &Course) -> Option<course::Exercise> {
    let p = st.store.lock().unwrap();
    c.exercises.iter().find(|e| !p.data.exercises.contains_key(&e.id)).cloned()
}

fn history(st: &AppState, c: &Course, limit: usize) -> Vec<HistView> {
    let p = st.store.lock().unwrap();
    p.data
        .history
        .iter()
        .take(limit)
        .map(|h| {
            let (label, href) = match h.kind.as_str() {
                "exercise" => (
                    c.exercise(&h.id).map(|e| format!("演習 {}-{} {}", e.chapter_number, e.index, e.title)).unwrap_or(h.id.clone()),
                    format!("/exercises/{}", h.id),
                ),
                "final" => (
                    c.final_(&h.id).map(|f| f.title.clone()).unwrap_or(h.id.clone()),
                    format!("/final/{}", h.id),
                ),
                _ => (h.id.clone(), "#".into()),
            };
            HistView { label, href, ok: h.ok, at: h.at }
        })
        .collect()
}

fn list_rs_files(dir: &std::path::Path, root: &std::path::Path) -> Vec<String> {
    let mut out = Vec::new();
    fn walk(d: &std::path::Path, root: &std::path::Path, out: &mut Vec<String>) {
        let Ok(rd) = fs::read_dir(d) else { return };
        let mut entries: Vec<_> = rd.filter_map(|e| e.ok()).map(|e| e.path()).collect();
        entries.sort();
        for p in entries {
            if p.is_dir() {
                walk(&p, root, out);
            } else if p.extension().is_some_and(|x| x == "rs") {
                out.push(p.strip_prefix(root).unwrap_or(&p).display().to_string());
            }
        }
    }
    walk(dir, root, &mut out);
    // main.rs / lib.rs を先頭に
    out.sort_by_key(|p| (!(p.ends_with("/lib.rs") || p.ends_with("/main.rs")), p.clone()));
    out
}

// ------------------------------------------------------------------ テンプレート

#[derive(Template)]
#[template(path = "not_found.html")]
struct NotFoundTpl {
    page: PageMeta,
    what: String,
}

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTpl {
    page: PageMeta,
    next: Option<ExView>,
    ex_done: usize,
    ex_total: usize,
    finals_done: usize,
    finals_total: usize,
    stages: Vec<StageView>,
    history: Vec<HistView>,
}

pub async fn home(State(st): State<AppState>) -> Response {
    let c = match load(&st) { Ok(c) => c, Err(r) => return r };
    let next = next_exercise(&st, &c).map(|e| ex_view(&st, &c, &e));
    let stages: Vec<StageView> = c.stages.iter().map(|s| stage_view(&st, &c, s)).collect();
    let (ex_done, finals_done) = {
        let p = st.store.lock().unwrap();
        (
            c.exercises.iter().filter(|e| p.data.exercises.contains_key(&e.id)).count(),
            c.finals.iter().filter(|f| p.data.finals.contains_key(&f.id)).count(),
        )
    };
    let mut page = PageMeta::new(&st, "ホーム", "home");
    page.crumbs.clear();
    render(HomeTpl {
        page,
        next,
        ex_done,
        ex_total: c.exercises.len(),
        finals_done,
        finals_total: c.finals.len(),
        stages,
        history: history(&st, &c, 8),
    })
}

#[derive(Template)]
#[template(path = "roadmap.html")]
struct RoadmapTpl {
    page: PageMeta,
    stages: Vec<StageView>,
}

pub async fn roadmap(State(st): State<AppState>) -> Response {
    let c = match load(&st) { Ok(c) => c, Err(r) => return r };
    let stages = c.stages.iter().map(|s| stage_view(&st, &c, s)).collect();
    let page = PageMeta::new(&st, "ロードマップ", "roadmap").crumb("/roadmap", "ロードマップ");
    render(RoadmapTpl { page, stages })
}

#[derive(Template)]
#[template(path = "stage.html")]
struct StageTpl {
    page: PageMeta,
    stage: StageView,
}

pub async fn stage(State(st): State<AppState>, Path(id): Path<u32>) -> Response {
    let c = match load(&st) { Ok(c) => c, Err(r) => return r };
    let Some(s) = c.stage(id) else { return not_found(&st, "ステージ") };
    let stage = stage_view(&st, &c, s);
    let mut page = PageMeta::new(&st, format!("Stage {} {}", s.id, s.title), "roadmap")
        .crumb("/roadmap", "ロードマップ")
        .crumb(format!("/stages/{id}"), format!("Stage {}", s.id));
    page.prev = c.stage(id.wrapping_sub(1)).map(|p| link(format!("/stages/{}", p.id), format!("Stage {} {}", p.id, p.title)));
    page.next = c.stage(id + 1).map(|p| link(format!("/stages/{}", p.id), format!("Stage {} {}", p.id, p.title)));
    render(StageTpl { page, stage })
}

#[derive(Template)]
#[template(path = "lessons.html")]
struct LessonsTpl {
    page: PageMeta,
    stages: Vec<StageView>,
}

pub async fn lessons(State(st): State<AppState>) -> Response {
    let c = match load(&st) { Ok(c) => c, Err(r) => return r };
    let stages = c.stages.iter().map(|s| stage_view(&st, &c, s)).collect();
    let page = PageMeta::new(&st, "解説", "lessons").crumb("/lessons", "解説");
    render(LessonsTpl { page, stages })
}

#[derive(Template)]
#[template(path = "lesson.html")]
struct LessonTpl {
    page: PageMeta,
    chapter: ChapterView,
    stage_title: String,
    html: String,
    toc: Vec<(String, String)>,
    missing: bool,
}

pub async fn lesson(State(st): State<AppState>, Path(id): Path<String>) -> Response {
    let c = match load(&st) { Ok(c) => c, Err(r) => return r };
    let Some(ch) = c.chapter(&id) else { return not_found(&st, "章") };
    let path = st.root.join("docs").join(format!("{id}.md"));
    let (md, missing) = match fs::read_to_string(&path) {
        Ok(s) => (s, false),
        Err(_) => (String::from("この章の解説はまだありません。"), true),
    };
    let r = markdown::render(&course::strip_title(&md));
    st.store.lock().unwrap().mark_read(&id);
    let chapter = chapter_view(&st, &c, ch);
    let stage = c.stage(ch.stage);
    let stage_title = stage.map(|s| format!("Stage {} {}", s.id, s.title)).unwrap_or_default();
    let idx = c.chapters.iter().position(|x| x.id == id).unwrap();
    let mut page = PageMeta::new(&st, format!("第{}章 {}", ch.number, ch.title), "lessons")
        .crumb(format!("/stages/{}", ch.stage), stage_title.clone())
        .crumb(format!("/lessons/{id}"), format!("第{}章", ch.number));
    page.prev = idx.checked_sub(1).and_then(|i| c.chapters.get(i)).map(|p| link(format!("/lessons/{}", p.id), format!("第{}章 {}", p.number, p.title)));
    page.next = c.chapters.get(idx + 1).map(|p| link(format!("/lessons/{}", p.id), format!("第{}章 {}", p.number, p.title)));
    render(LessonTpl { page, chapter, stage_title, html: r.html, toc: r.toc, missing })
}

#[derive(Template)]
#[template(path = "exercises.html")]
struct ExercisesTpl {
    page: PageMeta,
    stages: Vec<StageView>,
    done: usize,
    total: usize,
}

pub async fn exercises(State(st): State<AppState>) -> Response {
    let c = match load(&st) { Ok(c) => c, Err(r) => return r };
    let stages: Vec<StageView> = c.stages.iter().filter(|s| s.id != 5).map(|s| stage_view(&st, &c, s)).collect();
    let total = c.exercises.len();
    let done = stages.iter().flat_map(|s| &s.chapters).map(|c| c.ex_done).sum();
    let page = PageMeta::new(&st, "演習", "exercises").crumb("/exercises", "演習");
    render(ExercisesTpl { page, stages, done, total })
}

#[derive(Template)]
#[template(path = "exercise.html")]
struct ExerciseTpl {
    page: PageMeta,
    ex: ExView,
    chapter_href: String,
    chapter_label: String,
    description: String,
    code: String,
    file: String,
    hints: Vec<String>,
    has_solution: bool,
}

pub async fn exercise(State(st): State<AppState>, Path((ch, name)): Path<(String, String)>) -> Response {
    let c = match load(&st) { Ok(c) => c, Err(r) => return r };
    let id = format!("{ch}/{name}");
    let Some(e) = c.exercise(&id).cloned() else { return not_found(&st, "演習") };
    let chapter = c.chapter(&ch).unwrap();
    let file = format!("exercises/{id}.rs");
    crate::ensure_exercise_copy(&st.root, &id);
    let code = fs::read_to_string(st.root.join(&file)).unwrap_or_default();
    let hints_md = fs::read_to_string(st.root.join("course/hints").join(format!("{id}.md"))).unwrap_or_default();
    let hints = course::split_hints(&hints_md).iter().map(|h| markdown::render(h).html).collect();
    let has_solution = st.root.join("course/solutions").join(format!("{id}.rs")).exists();
    let idx = c.exercises.iter().position(|x| x.id == id).unwrap();
    let stage = c.stage(chapter.stage).map(|s| format!("Stage {}", s.id)).unwrap_or_default();
    let mut page = PageMeta::new(&st, format!("演習 {}-{} {}", e.chapter_number, e.index, e.title), "exercises")
        .crumb(format!("/stages/{}", chapter.stage), stage)
        .crumb(format!("/lessons/{ch}"), format!("第{}章 {}", chapter.number, chapter.title))
        .crumb(format!("/exercises/{id}"), format!("演習 {}-{}", e.chapter_number, e.index));
    page.wide = true;
    page.prev = idx.checked_sub(1).and_then(|i| c.exercises.get(i)).map(|p| link(format!("/exercises/{}", p.id), format!("{}-{} {}", p.chapter_number, p.index, p.title)));
    page.next = c.exercises.get(idx + 1).map(|p| link(format!("/exercises/{}", p.id), format!("{}-{} {}", p.chapter_number, p.index, p.title)));
    let ex = ex_view(&st, &c, &e);
    render(ExerciseTpl {
        page,
        ex,
        chapter_href: format!("/lessons/{ch}"),
        chapter_label: format!("第{}章 {}", chapter.number, chapter.title),
        description: markdown::render(&e.description).html,
        code,
        file,
        hints,
        has_solution,
    })
}

#[derive(Template)]
#[template(path = "solution.html")]
struct SolutionTpl {
    page: PageMeta,
    ex: ExView,
    solution: String,
    mine: String,
}

pub async fn solution(State(st): State<AppState>, Path((ch, name)): Path<(String, String)>) -> Response {
    let c = match load(&st) { Ok(c) => c, Err(r) => return r };
    let id = format!("{ch}/{name}");
    let Some(e) = c.exercise(&id).cloned() else { return not_found(&st, "演習") };
    let Ok(solution) = fs::read_to_string(st.root.join("course/solutions").join(format!("{id}.rs"))) else {
        return not_found(&st, "模範解答");
    };
    let mine = fs::read_to_string(st.root.join("exercises").join(format!("{id}.rs"))).unwrap_or_default();
    let ex = ex_view(&st, &c, &e);
    let mut page = PageMeta::new(&st, format!("解答: {}", e.title), "exercises")
        .crumb("/exercises", "演習")
        .crumb(format!("/exercises/{id}"), format!("演習 {}-{}", e.chapter_number, e.index))
        .crumb(format!("/exercises/{id}/solution"), "模範解答");
    page.wide = true;
    render(SolutionTpl { page, ex, solution, mine })
}

#[derive(Template)]
#[template(path = "projects.html")]
struct ProjectsTpl {
    page: PageMeta,
    projects: Vec<ProjView>,
}

pub async fn projects(State(st): State<AppState>) -> Response {
    let c = match load(&st) { Ok(c) => c, Err(r) => return r };
    let projects = c.projects.iter().map(|p| proj_view(&st, &c, p)).collect();
    let page = PageMeta::new(&st, "プロジェクト", "projects").crumb("/projects", "プロジェクト");
    render(ProjectsTpl { page, projects })
}

#[allow(dead_code)]
pub struct StepLink {
    pub n: usize,
    pub title: String,
    pub href: String,
    pub done: bool,
    pub current: bool,
}

#[derive(Template)]
#[template(path = "project.html")]
struct ProjectTpl {
    page: PageMeta,
    proj: ProjView,
    run: String,
    intro: String,
    steps: Vec<StepLink>,
    starter_files: Vec<String>,
}

fn step_links(st: &AppState, id: &str, steps: &[(String, String)], current: usize) -> Vec<StepLink> {
    let done = st.store.lock().unwrap().data.project_steps.get(id).cloned().unwrap_or_default();
    steps
        .iter()
        .enumerate()
        .map(|(i, (t, _))| StepLink {
            n: i + 1,
            title: t.clone(),
            href: format!("/projects/{id}/steps/{}", i + 1),
            done: done.contains(&(i + 1)),
            current: current == i + 1,
        })
        .collect()
}

pub async fn project(State(st): State<AppState>, Path(id): Path<String>) -> Response {
    let c = match load(&st) { Ok(c) => c, Err(r) => return r };
    let Some(p) = c.project(&id) else { return not_found(&st, "プロジェクト") };
    let (_, intro, steps) = project_steps(&st, &id);
    let proj = proj_view(&st, &c, p);
    let mut page = PageMeta::new(&st, p.title.clone(), "projects")
        .crumb("/projects", "プロジェクト")
        .crumb(format!("/projects/{id}"), p.title.clone());
    page.wide = true;
    if !steps.is_empty() {
        page.next = Some(link(format!("/projects/{id}/steps/1"), format!("{}", steps[0].0)));
    }
    let starter_files = list_rs_files(&st.root.join("projects").join(&id).join("starter/src"), &st.root);
    render(ProjectTpl {
        page,
        run: p.run.clone(),
        proj,
        intro: markdown::render(&intro).html,
        steps: step_links(&st, &id, &steps, 0),
        starter_files,
    })
}

#[derive(Template)]
#[template(path = "step.html")]
struct StepTpl {
    page: PageMeta,
    proj: ProjView,
    run: String,
    n: usize,
    title: String,
    html: String,
    toc: Vec<(String, String)>,
    done: bool,
    steps: Vec<StepLink>,
    starter_files: Vec<String>,
}

pub async fn project_step(State(st): State<AppState>, Path((id, n)): Path<(String, usize)>) -> Response {
    let c = match load(&st) { Ok(c) => c, Err(r) => return r };
    let Some(p) = c.project(&id) else { return not_found(&st, "プロジェクト") };
    let (_, _, steps) = project_steps(&st, &id);
    if n == 0 || n > steps.len() {
        return not_found(&st, "STEP");
    }
    let (title, body) = steps[n - 1].clone();
    let r = markdown::render(&body);
    let links = step_links(&st, &id, &steps, n);
    let done = links[n - 1].done;
    let mut page = PageMeta::new(&st, format!("{} — {}", p.title, title), "projects")
        .crumb("/projects", "プロジェクト")
        .crumb(format!("/projects/{id}"), p.title.clone())
        .crumb(format!("/projects/{id}/steps/{n}"), format!("STEP {n}"));
    page.wide = true;
    page.prev = Some(if n > 1 {
        link(format!("/projects/{id}/steps/{}", n - 1), steps[n - 2].0.clone())
    } else {
        link(format!("/projects/{id}"), "概要")
    });
    page.next = steps.get(n).map(|s| link(format!("/projects/{id}/steps/{}", n + 1), s.0.clone()));
    let starter_files = list_rs_files(&st.root.join("projects").join(&id).join("starter/src"), &st.root);
    render(StepTpl {
        page,
        run: p.run.clone(),
        proj: proj_view(&st, &c, p),
        n,
        title,
        html: r.html,
        toc: r.toc,
        done,
        steps: links,
        starter_files,
    })
}

#[derive(Template)]
#[template(path = "finals.html")]
struct FinalsTpl {
    page: PageMeta,
    finals: Vec<FinalView>,
    intro: String,
}

pub async fn finals(State(st): State<AppState>) -> Response {
    let c = match load(&st) { Ok(c) => c, Err(r) => return r };
    let finals = c.finals.iter().map(|f| final_view(&st, f)).collect();
    let page = PageMeta::new(&st, "卒業課題", "final").crumb("/stages/5", "Stage 5").crumb("/final", "卒業課題");
    let intro = fs::read_to_string(st.root.join("docs/23_final.md")).map(|m| markdown::render(&course::strip_title(&m)).html).unwrap_or_default();
    render(FinalsTpl { page, finals, intro })
}

#[derive(Template)]
#[template(path = "final.html")]
struct FinalTpl {
    page: PageMeta,
    f: FinalView,
    spec: String,
    toc: Vec<(String, String)>,
    hints: Vec<String>,
    files: Vec<String>,
    has_solution: bool,
}

pub async fn final_page(State(st): State<AppState>, Path(id): Path<String>) -> Response {
    let c = match load(&st) { Ok(c) => c, Err(r) => return r };
    let Some(f) = c.final_(&id) else { return not_found(&st, "卒業課題") };
    let dir = st.root.join("finals").join(&id);
    let spec_md = fs::read_to_string(dir.join("SPEC.md")).unwrap_or_else(|_| "仕様書はまだありません。".into());
    let r = markdown::render(&course::strip_title(&spec_md));
    let hints = course::split_hints(&fs::read_to_string(dir.join("HINTS.md")).unwrap_or_default())
        .iter()
        .map(|h| markdown::render(h).html)
        .collect();
    let files = list_rs_files(&dir.join("src"), &st.root);
    let has_solution = st.root.join("course/finals").join(&id).join("solution").exists();
    let idx = c.finals.iter().position(|x| x.id == id).unwrap();
    let mut page = PageMeta::new(&st, f.title.clone(), "final")
        .crumb("/final", "卒業課題")
        .crumb(format!("/final/{id}"), f.title.clone());
    page.wide = true;
    page.prev = idx.checked_sub(1).and_then(|i| c.finals.get(i)).map(|p| link(format!("/final/{}", p.id), p.title.clone()));
    page.next = c.finals.get(idx + 1).map(|p| link(format!("/final/{}", p.id), p.title.clone()));
    render(FinalTpl { page, f: final_view(&st, f), spec: r.html, toc: r.toc, hints, files, has_solution })
}

pub struct SolFile {
    pub path: String,
    pub code: String,
}

#[derive(Template)]
#[template(path = "final_solution.html")]
struct FinalSolutionTpl {
    page: PageMeta,
    f: FinalView,
    files: Vec<SolFile>,
}

pub async fn final_solution(State(st): State<AppState>, Path(id): Path<String>) -> Response {
    let c = match load(&st) { Ok(c) => c, Err(r) => return r };
    let Some(f) = c.final_(&id) else { return not_found(&st, "卒業課題") };
    let base = st.root.join("course/finals").join(&id).join("solution");
    let files = list_rs_files(&base, &base)
        .into_iter()
        .map(|p| SolFile { code: fs::read_to_string(base.join(&p)).unwrap_or_default(), path: p })
        .collect();
    let mut page = PageMeta::new(&st, format!("模範解答: {}", f.title), "final")
        .crumb("/final", "卒業課題")
        .crumb(format!("/final/{id}"), f.title.clone())
        .crumb(format!("/final/{id}/solution"), "模範解答");
    page.wide = true;
    render(FinalSolutionTpl { page, f: final_view(&st, f), files })
}

#[derive(Deserialize)]
pub struct TermQuery {
    pub cmd: Option<String>,
}

#[derive(Template)]
#[template(path = "terminal.html")]
struct TerminalTpl {
    page: PageMeta,
    cmd: String,
    presets: Vec<Link>,
}

pub async fn terminal(State(st): State<AppState>, Query(q): Query<TermQuery>) -> Response {
    let c = match load(&st) { Ok(c) => c, Err(r) => return r };
    let mut presets: Vec<Link> = c.projects.iter().map(|p| link(p.run.clone(), p.title.clone())).collect();
    presets.extend(c.finals.iter().filter(|f| f.tui).map(|f| link(format!("cargo run -p {}", f.id), f.title.clone())));
    let mut page = PageMeta::new(&st, "ターミナル", "terminal").crumb("/terminal", "ターミナル");
    page.wide = true;
    render(TerminalTpl { page, cmd: q.cmd.unwrap_or_default(), presets })
}

#[derive(Template)]
#[template(path = "progress.html")]
struct ProgressTpl {
    page: PageMeta,
    stages: Vec<StageView>,
    history: Vec<HistView>,
}

pub async fn progress(State(st): State<AppState>) -> Response {
    let c = match load(&st) { Ok(c) => c, Err(r) => return r };
    let stages = c.stages.iter().map(|s| stage_view(&st, &c, s)).collect();
    let page = PageMeta::new(&st, "進捗", "progress").crumb("/progress", "進捗");
    render(ProgressTpl { page, stages, history: history(&st, &c, 100) })
}

#[derive(Template)]
#[template(path = "doc.html")]
struct DocTpl {
    page: PageMeta,
    html: String,
    toc: Vec<(String, String)>,
}

pub async fn reference(State(st): State<AppState>) -> Response {
    let md = fs::read_to_string(st.root.join("docs/reference.md")).unwrap_or_else(|_| "# 早見表\n準備中".into());
    let title = course::md_title(&md).unwrap_or_else(|| "早見表".into());
    let r = markdown::render(&course::strip_title(&md));
    let page = PageMeta::new(&st, title, "reference").crumb("/reference", "早見表");
    render(DocTpl { page, html: r.html, toc: r.toc })
}

pub async fn fallback(State(st): State<AppState>) -> Response {
    not_found(&st, "ページ")
}
