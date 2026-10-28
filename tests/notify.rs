//! The board.notify setting: a task that needs you, is ready to approve or failed rings the bell on an open board or
//! runs your command, once per event; `none`, the built-in value, does neither.
use crate::common;
use common::*;
use serde_json::json;
use sw::json::*;
use sw::notify::event_of;
use sw::store::settings::change::{set_value, unset_value};
use sw::store::settings::lookup::{setting, Scope};
use sw::store::tasks::{update, Change};

fn pass() { sw::housekeeping::housekeeping(&sw::herdr::cli::current_socket(), sw::housekeeping::Force::Yes); }
thread_local! { static RAN: std::cell::RefCell<Vec<(String, Vec<String>)>> = const { std::cell::RefCell::new(vec![]) }; }
/// The jobs so far run, as `settle` does; a command for an event (a detached command is a job of its own in a test) is
/// kept, not run.
fn drain() { flush(); sw::runner::drain_jobs(&|job, id, extra| { if job == "notify" { RAN.with(|r| r.borrow_mut().push((id.to_string(), extra.to_vec()))); } else { sw::jobs::run_job(job, id, extra); } flush(); }); }
/// The commands run so far: (command, its environment).
fn ran() -> Vec<(String, Vec<String>)> { drain(); RAN.with(|r| r.borrow().clone()) }
fn put(id: &str, f: impl FnOnce(&mut Obj)) { update(id, |t| { f(t); Ok(Change::Write) }).unwrap(); sw::clock::advance(10); }
/// The board's loader comes in, as on every beat: how often did the bell ring.
fn rings(b: &mut common::board::Board) -> usize {
    use sw::board::data::{load, Wants};
    let d = load(&Wants { ids: vec![b.s.sel.clone()], cwd: b.s.ctx.cwd.clone(), own_pane: String::new(), web: false, ..Default::default() });
    sw::board::update::update(&mut b.s, sw::board::update::Event::Loaded(Box::new(d))).iter().filter(|e| matches!(e, sw::board::update::Effect::Bell)).count()
}

#[test]
fn board_notify_what_counts_as_an_event_and_none_bell_or_a_command_as_its_value() {
    let _w = world();
    assert_eq!(setting("board.notify", None), json!("none"));
    for (stage, event) in [("asks", "needs you"), ("reviewneeds", "needs you"), ("failed", "failed"), ("nosummary", "failed"), ("approval", "ready to approve"), ("working", ""), ("todo", ""), ("done", ""), ("merging", "")] {
        assert_eq!(event_of(&obj(json!({ "id": "p/1", "stage": stage }))), event, "{stage}");
    }
    assert_eq!(event_of(&obj(json!({ "id": "p/1", "stage": "approval", "prFailure": { "title": "x" } }))), "needs you");
    assert_eq!(event_of(&obj(json!({ "id": "p/1", "stage": "failed", "archivedAt": 1 }))), "");
    assert_eq!(set_value("board.notify", &json!("Bell"), &Scope::you()).unwrap().value, json!("bell"));
    assert_eq!(set_value("board.notify", &json!("notify-send \"$HERDR_PROJECTS_TITLE\""), &Scope::you()).unwrap().value, json!("notify-send \"$HERDR_PROJECTS_TITLE\""));
    assert_eq!(unset_value("board.notify", &Scope::you()).unwrap().value, json!("none"));
    let t = obj(json!({ "id": "proj/1" }));
    let e = set_value("board.notify", &json!("bell"), &Scope::task(&t)).unwrap_err().msg;
    assert!(e.contains("board.notify is a board setting: it is yours only"), "{e}");
}

#[test]
fn board_notify_a_command_runs_once_per_event_with_the_task_s_id_title_and_what_happened_the_bell_rings_once_on_an_open_board() {
    let w = world_with(Opts { repo: true, settings: json!({ "review.when": "never", "board.notify": "tell-me" }), ..Default::default() });
    // The task is taken to its approval as `to_approval` does, with each job seen as it is run: herdr's hook runs a
    // pass of its own on the way, and its command is the first one.
    w.script(json!({ "worker": [step(1)] }));
    let id = w.add("Ready soon");
    delegate(&id);
    for _ in 0..12 { drain(); let t = task(&id); if t.at("link").b("pane") { on_pane(&w, t.at("link").s("pane")); } if t.s("stage") == "approval" { break; } }
    assert_eq!(task(&id).s("stage"), "approval", "{}", task(&id).at("failure"));
    sw::clock::set(Some(sw::clock::now()));
    pass(); pass();
    let mut env = vec![format!("HERDR_PROJECTS_ID={id}"), "HERDR_PROJECTS_TITLE=Ready soon".to_string(), "HERDR_PROJECTS_EVENT=ready to approve".to_string()];
    assert_eq!(ran(), vec![("tell-me".to_string(), env.clone())]); // once, however often the board looks
    put(&id, |t| { t.put("stage", "failed"); t.put("msg", "Check failed"); });
    pass(); pass();
    let events: Vec<String> = ran().iter().map(|(_, e)| e[2].clone()).collect();
    env[2] = "HERDR_PROJECTS_EVENT=failed".into();
    assert_eq!(events, vec!["HERDR_PROJECTS_EVENT=ready to approve".to_string(), env[2].clone()]);
    put(&id, |t| { t.put("stage", "working"); }); pass();
    assert!(task(&id).at("notice").is_null()); // nothing to tell: and the next event is new again
    // the bell: an open board rings for what happens while it is open, once, and not for what was already so
    set_value("board.notify", &json!("bell"), &Scope::you()).unwrap();
    put(&id, |t| { t.put("stage", "approval"); }); pass();
    assert_eq!(ran().len(), 2); // no command while it is `bell`
    let mut b = common::board::open(&w.root, 40);
    assert_eq!(rings(&mut b) + rings(&mut b), 0);
    put(&id, |t| { t.put("stage", "asks"); t.put("kind", "question"); }); pass();
    assert_eq!(rings(&mut b) + rings(&mut b), 1);
    set_value("board.notify", &json!("none"), &Scope::you()).unwrap();
    put(&id, |t| { t.put("stage", "failed"); }); pass();
    assert_eq!(rings(&mut b), 0);
    assert_eq!(ran().len(), 2);
}

const NOTIFY_JS: &str = include_str!("../crates/web/src/webui/page/notify.js");
const BOARD_JS: &str = include_str!("../crates/web/src/webui/page/board.js");
const START_JS: &str = include_str!("../crates/web/src/webui/page/start.js");
const HOSTILE: &str = "x\" & (do shell script \"touch /tmp/pwned\") & \"\\ $(id) `id` -e";

#[test]
fn board_notify_mac_is_a_value_of_its_own_and_a_task_s_title_never_gets_into_the_script() {
    let _w = world();
    assert_eq!(set_value("board.notify", &json!("Mac"), &Scope::you()).unwrap().value, json!("mac"));
    assert_eq!(set_value("board.notify", &json!("bell"), &Scope::you()).unwrap().value, json!("bell"));
    assert_eq!(set_value("board.notify", &json!("notify-send x"), &Scope::you()).unwrap().value, json!("notify-send x"));
    for bad in [json!(""), json!("  "), json!(3)] { assert!(set_value("board.notify", &bad, &Scope::you()).unwrap_err().msg.contains("none, bell, mac, or a command"), "{bad}"); }
    // the script is the same whatever the title: the title and the event are the last two arguments, after `--`, as they are
    let plain = sw::notify::mac_args("Fix it", "failed");
    let args = sw::notify::mac_args(HOSTILE, "needs you");
    assert_eq!(args.len(), 9);
    assert_eq!(args[..6], plain[..6], "the script does not change with the title");
    assert_eq!(args[6..], ["--", HOSTILE, "needs you"]);
    assert!(args[..7].iter().all(|a| !a.contains("pwned") && !a.contains("Fix it")), "{args:?}");
    assert!(args[..6].iter().step_by(2).all(|a| a == "-e") && args[1] == "on run argv" && args[5] == "end run");
    assert!(args[3].starts_with("display notification (item 1 of argv) with title \"Herdr board\" subtitle (item 2 of argv)"), "{}", args[3]);
}

#[test]
fn board_notify_mac_notifies_once_per_event_and_the_command_is_not_run() {
    let w = world_with(Opts { repo: true, settings: json!({ "board.notify": "mac" }), ..Default::default() });
    let id = w.add("Quote \"this\" $HOME");
    sw::clock::set(Some(sw::clock::now()));
    put(&id, |t| { t.put("stage", "failed"); });
    pass(); pass();
    let env = vec![format!("HERDR_PROJECTS_ID={id}"), "HERDR_PROJECTS_TITLE=Quote \"this\" $HOME".to_string(), "HERDR_PROJECTS_EVENT=failed".to_string()];
    assert_eq!(ran(), vec![("mac".to_string(), env)]);
    // the web board's row says the event, as the page's bell needs it
    let b = common::board::open(&w.root, 40);
    let l = sw::webui::model::list::list_of(&b.s, &b.theme);
    let events: Vec<String> = l.arr("groups").iter().flat_map(|g| g.arr("rows").iter().map(|r| r.t("event")).collect::<Vec<_>>()).collect();
    assert_eq!(events, vec!["failed".to_string()]);
}

/// The two arguments reach osascript as data: its own script, with the reply put in place of the notification.
#[cfg(target_os = "macos")]
#[test]
fn real_programs_osascript_takes_a_hostile_title_and_the_event_as_data() {
    let mut args = sw::notify::mac_args(HOSTILE, "needs you");
    args[3] = "return (item 1 of argv) & \"|\" & (item 2 of argv)".into();
    let out = std::process::Command::new("osascript").args(&args).output().unwrap();
    assert!(out.status.success(), "{}", String::from_utf8_lossy(&out.stderr));
    assert_eq!(String::from_utf8_lossy(&out.stdout).trim_end_matches('\n'), format!("{HOSTILE}|needs you"));
    assert!(!std::path::Path::new("/tmp/pwned").exists());
}

#[test]
fn board_notify_the_web_board_s_bell_is_a_page_file_that_is_served_and_loaded() {
    assert!(sw::webui::page::file("notify.js").is_some_and(|f| f == NOTIFY_JS) && sw::webui::page::file("notify.css").is_some());
    assert!(START_JS.contains(r#""boards", "notify"]"#), "its styles are linked");
    assert!(BOARD_JS.contains(r#"from "./notify.js""#) && BOARD_JS.contains("notifyEl(") && BOARD_JS.contains("watch(snap)"));
    // asked first, kept in the browser, and said where it cannot work, never failed
    assert!(NOTIFY_JS.contains("Notification.requestPermission()") && NOTIFY_JS.contains("localStorage.setItem(KEY") && NOTIFY_JS.contains("Safari tab"));
}
