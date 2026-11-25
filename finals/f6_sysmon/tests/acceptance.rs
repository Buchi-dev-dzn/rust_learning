//! F6 受け入れテスト (編集しないこと)

use f6_sysmon::*;

fn p(pid: u32, cpu: f32, mem: f32, cmd: &str) -> ProcessInfo {
    ProcessInfo::new(pid, cpu, mem, cmd)
}

fn snap(list: Vec<ProcessInfo>) -> Snapshot {
    Snapshot { processes: list }
}

fn sample() -> Snapshot {
    snap(vec![
        p(10, 5.0, 1.0, "bash"),
        p(20, 50.0, 3.0, "firefox"),
        p(30, 5.0, 9.0, "Xorg"),
        p(40, 0.0, 0.5, "sshd"),
        p(5, 20.0, 2.0, "Firefox Helper"),
    ])
}

fn pids(app: &App) -> Vec<u32> {
    app.visible().iter().map(|p| p.pid).collect()
}

#[test]
fn parse_ps_output() {
    let out = "  PID  %CPU %MEM COMM\n    1   0.0  0.1 /sbin/launchd\n  812  12.5  1.2 /Applications/Google Chrome.app/Contents/MacOS/Google Chrome\n\ngarbage line here\n  99   1.0\n  77   0.3  0.0   \n";
    let v = parse_ps(out);
    assert_eq!(
        v,
        vec![
            p(1, 0.0, 0.1, "/sbin/launchd"),
            p(812, 12.5, 1.2, "/Applications/Google Chrome.app/Contents/MacOS/Google Chrome"),
        ]
    );
    assert!(parse_ps("").is_empty());
}

#[test]
fn snapshot_total_cpu() {
    assert_eq!(sample().total_cpu(), 80.0);
    assert_eq!(Snapshot::default().total_cpu(), 0.0);
}

#[test]
fn history_ring_buffer() {
    let mut h = History::new(3);
    assert!(h.is_empty());
    assert_eq!((h.capacity(), h.latest(), h.max()), (3, None, 0.0));
    for v in [1.0, 2.5, 7.0, 4.0] {
        h.push(v);
    }
    assert_eq!(h.len(), 3);
    assert_eq!(h.values(), vec![2.5, 7.0, 4.0]);
    assert_eq!(h.latest(), Some(4.0));
    assert_eq!(h.max(), 7.0);
    assert_eq!(h.sparkline_data(), vec![3, 7, 4]);
    h.push(-2.0);
    assert_eq!(h.sparkline_data(), vec![7, 4, 0]);
    let mut negative = History::new(2);
    negative.push(-3.0);
    negative.push(-2.0);
    assert_eq!(negative.max(), -2.0);
    assert_eq!(negative.sparkline_data(), vec![0, 0]);
    let mut z = History::new(0);
    z.push(1.0);
    assert!(z.is_empty());
}

#[test]
fn new_app_defaults() {
    let app = App::new(60);
    assert_eq!(app.sort_key(), SortKey::Cpu);
    assert!(app.sort_desc());
    assert_eq!((app.filter(), app.mode()), ("", Mode::Normal));
    assert!(!app.paused() && !app.should_quit());
    assert_eq!((app.selected(), app.process_count()), (0, 0));
    assert!(app.visible().is_empty());
    assert_eq!(app.selected_process(), None);
    assert_eq!(app.history().capacity(), 60);
}

#[test]
fn update_replaces_processes_and_records_history() {
    let mut app = App::new(10);
    app.update(sample());
    assert_eq!(app.process_count(), 5);
    app.update(snap(vec![p(1, 2.0, 0.0, "a"), p(2, 3.0, 0.0, "b")]));
    assert_eq!(app.process_count(), 2);
    assert_eq!(app.history().values(), vec![80.0, 5.0]);
}

#[test]
fn default_sort_is_cpu_desc_with_pid_tiebreak() {
    let mut app = App::new(10);
    app.update(sample());
    // cpu: 20→50, 5→20, 10→5, 30→5, 40→0 (同じ 5.0 は pid 昇順)
    assert_eq!(pids(&app), vec![20, 5, 10, 30, 40]);
}

#[test]
fn set_sort_toggles_and_defaults() {
    let mut app = App::new(10);
    app.update(sample());
    app.set_sort(SortKey::Cpu);
    assert!(!app.sort_desc());
    assert_eq!(pids(&app), vec![40, 10, 30, 5, 20]);
    app.set_sort(SortKey::Pid);
    assert!(!app.sort_desc());
    assert_eq!(pids(&app), vec![5, 10, 20, 30, 40]);
    app.set_sort(SortKey::Pid);
    assert_eq!(pids(&app), vec![40, 30, 20, 10, 5]);
    app.set_sort(SortKey::Mem);
    assert!(app.sort_desc());
    assert_eq!(pids(&app), vec![30, 20, 5, 10, 40]);
    app.set_sort(SortKey::Command);
    assert!(!app.sort_desc());
    // String の順序: 大文字が先
    assert_eq!(pids(&app), vec![5, 30, 10, 20, 40]);
}

#[test]
fn filter_is_case_insensitive_substring() {
    let mut app = App::new(10);
    app.update(sample());
    app.set_filter("FIRE");
    assert_eq!(pids(&app), vec![20, 5]);
    assert_eq!(app.process_count(), 5);
    app.set_filter("nothing");
    assert!(app.visible().is_empty());
    assert_eq!((app.selected(), app.selected_process()), (0, None));
    app.set_filter("");
    assert_eq!(app.visible().len(), 5);
}

#[test]
fn selection_movement_clamps() {
    let mut app = App::new(10);
    app.update(sample());
    app.select_prev();
    assert_eq!(app.selected(), 0);
    app.select_next();
    app.select_next();
    assert_eq!(app.selected_process().unwrap().pid, 10);
    app.page_down(10);
    assert_eq!(app.selected(), 4);
    app.select_next();
    assert_eq!(app.selected(), 4);
    app.page_up(3);
    assert_eq!(app.selected(), 1);
    app.select_last();
    assert_eq!(app.selected(), 4);
    app.select_first();
    assert_eq!(app.selected(), 0);
}

#[test]
fn selection_follows_pid_across_updates_sort_and_filter() {
    let mut app = App::new(10);
    app.update(sample());
    app.select_next(); // pid 5 (Firefox Helper)
    assert_eq!(app.selected_process().unwrap().pid, 5);
    // CPU が変わって順位が変わっても同じプロセスを選び続ける
    app.update(snap(vec![p(5, 1.0, 2.0, "Firefox Helper"), p(20, 50.0, 3.0, "firefox"), p(10, 30.0, 1.0, "bash")]));
    assert_eq!(app.selected_process().unwrap().pid, 5);
    assert_eq!(app.selected(), 2);
    app.set_sort(SortKey::Pid);
    assert_eq!(app.selected(), 0);
    app.set_filter("fire");
    assert_eq!(app.selected_process().unwrap().pid, 5);
}

#[test]
fn selection_clamps_when_selected_process_disappears() {
    let mut app = App::new(10);
    app.update(sample());
    app.select_last(); // index 4 (pid 40)
    app.update(snap(vec![p(1, 1.0, 0.0, "a"), p(2, 2.0, 0.0, "b")]));
    assert_eq!(app.selected(), 1);
    app.update(snap(vec![]));
    assert_eq!(app.selected(), 0);
    assert_eq!(app.selected_process(), None);
}

#[test]
fn paused_ignores_updates() {
    let mut app = App::new(10);
    app.update(sample());
    app.handle_input(Input::Char(' '), 10);
    assert!(app.paused());
    app.update(snap(vec![p(1, 99.0, 0.0, "x")]));
    assert_eq!(app.process_count(), 5);
    assert_eq!(app.history().len(), 1);
    app.handle_input(Input::Char(' '), 10);
    app.update(snap(vec![p(1, 99.0, 0.0, "x")]));
    assert_eq!(app.process_count(), 1);
    assert_eq!(app.history().latest(), Some(99.0));
}

#[test]
fn scroll_offset_minimal_movement() {
    let mut app = App::new(10);
    let list: Vec<ProcessInfo> = (1..=20).map(|i| p(i, 0.0, 0.0, "proc")).collect();
    app.update(snap(list));
    app.set_sort(SortKey::Pid); // pid 昇順 = index i は pid i+1
    assert_eq!(app.scroll_offset(5), 0);
    app.page_down(4);
    assert_eq!(app.scroll_offset(5), 0);
    app.select_next(); // 5
    assert_eq!(app.scroll_offset(5), 1);
    app.page_down(10); // 15
    assert_eq!(app.scroll_offset(5), 11);
    app.page_up(2); // 13 は見えている → 動かない
    assert_eq!(app.scroll_offset(5), 11);
    app.page_up(3); // 10
    assert_eq!(app.scroll_offset(5), 10);
    assert_eq!(app.scroll_offset(0), 10);
    app.select_first();
    assert_eq!(app.scroll_offset(5), 0);
}

#[test]
fn handle_input_normal_mode() {
    let mut app = App::new(10);
    app.update(sample());
    app.handle_input(Input::Char('j'), 10);
    app.handle_input(Input::Down, 10);
    assert_eq!(app.selected(), 2);
    app.handle_input(Input::Char('k'), 10);
    assert_eq!(app.selected(), 1);
    app.handle_input(Input::Char('G'), 10);
    assert_eq!(app.selected(), 4);
    app.handle_input(Input::Char('g'), 10);
    assert_eq!(app.selected(), 0);
    app.handle_input(Input::PageDown, 3);
    assert_eq!(app.selected(), 3);
    app.handle_input(Input::End, 3);
    app.handle_input(Input::Home, 3);
    assert_eq!(app.selected(), 0);
    app.handle_input(Input::Char('p'), 10);
    assert_eq!(app.sort_key(), SortKey::Pid);
    app.handle_input(Input::Char('m'), 10);
    assert_eq!(app.sort_key(), SortKey::Mem);
    app.handle_input(Input::Char('n'), 10);
    assert_eq!(app.sort_key(), SortKey::Command);
    app.handle_input(Input::Char('c'), 10);
    assert_eq!((app.sort_key(), app.sort_desc()), (SortKey::Cpu, true));
    app.handle_input(Input::Char('x'), 10);
    assert!(!app.should_quit());
    app.handle_input(Input::Char('q'), 10);
    assert!(app.should_quit());
}

#[test]
fn handle_input_filter_mode() {
    let mut app = App::new(10);
    app.update(sample());
    app.handle_input(Input::Char('/'), 10);
    assert_eq!(app.mode(), Mode::Filter);
    for c in "sshq".chars() {
        app.handle_input(Input::Char(c), 10); // q も文字として入る
    }
    assert_eq!(app.filter(), "sshq");
    assert!(!app.should_quit());
    app.handle_input(Input::Backspace, 10);
    assert_eq!(pids(&app), vec![40]);
    app.handle_input(Input::Enter, 10);
    assert_eq!((app.mode(), app.filter()), (Mode::Normal, "ssh"));
    app.handle_input(Input::Esc, 10); // Normal の Esc はフィルタ解除
    assert_eq!(app.filter(), "");
    app.handle_input(Input::Char('/'), 10);
    app.handle_input(Input::Char('b'), 10);
    app.handle_input(Input::Down, 10); // 矢印は Filter モードでも移動
    app.handle_input(Input::Esc, 10);
    assert_eq!((app.mode(), app.filter()), (Mode::Normal, ""));
}

#[test]
fn filter_backspace_removes_one_unicode_character() {
    let mut app = App::new(2);
    app.handle_input(Input::Char('/'), 1);
    app.handle_input(Input::Char('猫'), 1);
    app.handle_input(Input::Char('犬'), 1);
    app.handle_input(Input::Backspace, 1);
    assert_eq!(app.filter(), "猫");
    app.handle_input(Input::Char('q'), 1);
    assert_eq!(app.filter(), "猫q");
    assert!(!app.should_quit());
}

#[test]
fn descending_ties_still_use_ascending_pid_and_filter_clamps_selection() {
    let mut app = App::new(2);
    app.update(snap(vec![
        p(9, 5.0, 1.0, "match a"),
        p(2, 5.0, 1.0, "match b"),
        p(4, 5.0, 1.0, "other"),
    ]));
    assert_eq!(pids(&app), vec![2, 4, 9]);
    app.set_sort(SortKey::Mem);
    assert_eq!(pids(&app), vec![2, 4, 9]);
    app.select_last();
    app.set_filter("other");
    assert_eq!(app.selected(), 0);
    assert_eq!(app.selected_process().unwrap().pid, 4);
    app.set_filter("absent");
    assert_eq!(app.selected(), 0);
    assert_eq!(app.selected_process(), None);
}
