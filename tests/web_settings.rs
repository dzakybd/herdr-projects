//! The web board's settings page: every setting of the open task with its plain label and what it means, in the
//! terminal page's order; changed and put back there; the task's own panel saying the same in words, its settings
//! above the long texts; a refused value refused in the board's words; read-only for a watcher and an archived task. And the accounts sheet: accounts made, read and changed
//! (core `agents::account_edit` and the `accounts` and `account-save` calls), refusing a path, a name there and shell injection.
use crate::common;
use common::board::{web, web_as};
use common::*;
use serde_json::{json, Value};
use sw::json::*;

fn row(s: &Value, name: &str) -> Value { s.at("settings").arr("rows").iter().find(|x| x.s("name") == name).cloned().unwrap_or_else(|| panic!("no row {name}")) }
fn set(w: &World, id: &str, name: &str, value: Value) -> Value { web(w, "set", json!({ "id": id, "name": name, "value": value })) }
fn rows_of(s: &Value) -> Vec<Value> { s.at("list").arr("groups").iter().flat_map(|g| g.arr("rows").to_vec()).collect() }

#[test]
fn every_setting_of_the_open_task_with_its_label_and_meaning_changed_and_put_back_there() {
    let w = world();
    let t = w.add("Look into the logs");
    assert!(web(&w, "board", json!({})).at("settings").is_null(), "no task open: nothing to set");
    let s = web(&w, "pick", json!({ "id": t, "open": true }));
    assert_eq!((s.at("settings").s("id"), s.at("settings").s("whose"), s.at("settings").at("canSet")), (t.as_str(), "this task", &json!(true)));
    // the terminal page's rows, in its order
    let want: Vec<String> = { let c = sw::webui::web::client("test"); let mut g = c.lock().unwrap(); let was = g.s.editing.replace(sw::board::state::Editing { id: t.clone(), ..Default::default() }); let r = sw::board::update::edit::rows(&g.s); g.s.editing = was; r.iter().map(|x| x.t("name")).collect() };
    assert_eq!(s.at("settings").arr("rows").iter().map(|x| x.t("name")).collect::<Vec<_>>(), want);
    assert_eq!(row(&s, "research.only"), json!({ "name": "research.only", "label": "Research only", "means": "its agent changes nothing in the repo; the result is a report, and nothing is pushed, opened or merged", "own": false, "value": "off", "from": "",
        "words": [{ "value": "true", "say": "on", "now": false }, { "value": "false", "say": "off", "now": true }], "hint": "" }));
    assert!(row(&s, "review.rounds").at("words").is_null()); assert!(row(&s, "review.rounds").s("hint").contains("a whole number, 0 or more"));
    // The two limits are there like every setting: the plain label, what the value may be, and what it means.
    let lt = row(&s, "limit.time");
    assert_eq!((lt.s("label"), lt.s("value"), lt.at("words")), ("Longest run", "none", &Value::Null));
    assert!(lt.s("hint").starts_with("a time, such as 90s or 15m; none for none. Means: the longest one run of an agent may take"), "{}", lt.s("hint"));
    assert!(regex::Regex::new(r"^the longest one run of an agent may take.*the task waits for you").unwrap().is_match(lt.s("means")), "{}", lt.s("means"));
    let lk = row(&s, "limit.tokens");
    assert_eq!(lk.s("label"), "Most tokens"); assert!(lk.s("hint").starts_with("a count of tokens, such as 500k or 2m; none for none"), "{}", lk.s("hint")); assert!(lk.s("means").starts_with("the most tokens the task may use"));
    // A refused value: the board's words say why (the Rust settings page keeps it in the box to be put right, and says so
    // as its message: no error is thrown back at the page).
    let s = set(&w, &t, "limit.time", json!("2 hours"));
    assert!(s.at("msg").s("text").contains("Did you mean 2h?"), "{}", s.at("msg")); assert_eq!(row(&s, "limit.time").s("value"), "none");
    let s = set(&w, &t, "limit.time", json!("2h")); assert_eq!((row(&s, "limit.time").s("value"), row(&s, "limit.time").at("own")), ("2h", &json!(true)));
    let s = set(&w, &t, "limit.time", Value::Null); assert_eq!(row(&s, "limit.time").s("value"), "none");
    // Changed from the page: the task's own, the label on its row follows, and the board says what it is now.
    let s = set(&w, &t, "research.only", json!("true"));
    assert_eq!(task(&t).at("settings"), &json!({ "research.only": true }));
    assert_eq!((row(&s, "research.only").s("value"), row(&s, "research.only").at("own"), row(&s, "research.only").s("from")), ("on", &json!(true), ""));
    assert!(!s.at("msg").s("text").starts_with("Research only"), "no confirmation: the row says it: {}", s.at("msg"));
    assert!(rows_of(&s).iter().find(|x| x.s("id") == t).unwrap().s("title").contains("Look into the logs [research]"));
    // The task's own panel says the same, in words: every setting it runs with, a `*` after a value set for this task
    // only. No value says where it comes from.
    {
        let facts: Vec<Value> = s.at("panel").arr("parts").iter().find(|p| p.s("head") == "Settings").unwrap().arr("items").iter().filter(|x| x.s("k") == "fact").cloned().collect();
        let said = |label: &str| facts.iter().find(|x| x.s("label") == label).map(|x| x.arr("lines").iter().flat_map(|l| l.as_array().cloned().unwrap_or_default()).map(|p| p.t("t")).collect::<String>()).unwrap_or_default();
        assert_eq!(facts.iter().map(|x| x.t("label")).filter(|k| !["Project", "Worktree", "Branch", "Note"].contains(&k.as_str())).collect::<Vec<_>>(), vec!["Agent", "Plan first", "Reviewer", "Review when", "Send back", "Check", "Research", "Workspace", "Also"]);
        assert_eq!(said("Research"), "only · no change to the repo *"); assert_eq!(said("Send back"), "major or worse · 3 rounds max");
        assert!(!s.at("panel").to_string().contains("as usual"));
    }
    // A long description never pushes them down the page: they stand right under the status, and under what the task needs from you.
    let heads = |hs: &[(&str, &str)]| sw::webui::model::panel::settings_up(hs.iter().map(|(h, c)| obj(json!({ "head": h, "color": c }))).collect()).iter().map(|p| p.t("head")).collect::<Vec<_>>();
    assert_eq!(heads(&[("Status", ""), ("Asks you", "red"), ("Description", ""), ("The agent said", ""), ("Settings", ""), ("Session", "")]), vec!["Status", "Asks you", "Settings", "Description", "The agent said", "Session"]);
    sw::store::tasks::patch(&t, |x| { x.put("description", (1..=80).map(|i| format!("Line {i} of a very long description.")).collect::<Vec<_>>().join("\n")); }).unwrap();
    let hs: Vec<String> = web(&w, "board", json!({})).at("panel").arr("parts").iter().map(|p| p.t("head")).collect();
    let at = |h: &str| hs.iter().position(|x| x == h).unwrap();
    assert!(at("Settings") < at("Description") && at("Settings") <= 2, "{hs:?}");
    // A typed value; a wrong one is refused in the board's words and changes nothing.
    let s = set(&w, &t, "review.rounds", json!("5")); assert_eq!((row(&s, "review.rounds").s("value"), task(&t).at("settings").at("review.rounds")), ("5", &json!(5)));
    let s = set(&w, &t, "review.rounds", json!("many")); assert!(s.at("msg").s("text").contains("review.rounds must be a whole number"), "{}", s.at("msg")); assert_eq!(task(&t).at("settings").at("review.rounds"), &json!(5));
    let s = set(&w, &t, "board.agents", json!("9")); assert!(s.at("msg").s("text").contains("board.agents is a board setting"), "{}", s.at("msg"));
    // Put back: what it would be without the task's own answer.
    let s = set(&w, &t, "research.only", Value::Null);
    assert_eq!(task(&t).at("settings"), &json!({ "review.rounds": 5 })); assert_eq!((row(&s, "research.only").s("value"), row(&s, "research.only").at("own")), ("off", &json!(false)));
    // A device that only watches changes nothing; nor does a change meant for a task that is no longer open.
    assert!(web_as(&w, "test", "set", json!({ "id": t, "name": "research.only", "value": "true", "role": "watch" })).unwrap_err().contains("can only watch"));
    let s = set(&w, "proj/999", "research.only", json!("true"));
    assert!(s.at("msg").s("text").contains("no longer the one that is open"), "{}", s.at("msg")); assert!(task(&t).at("settings").at("research.only").is_null());
    // Its key in More goes there; an archived task's settings are read-only.
    let s = web(&w, "press", json!({ "id": "settings" })); assert_eq!(s.at("settings").i("show"), 1); assert!(!s.at("panel").is_null());
    sw::actions::archive::archive(&t).unwrap();
    web(&w, "press", json!({ "id": "archiveView" }));
    assert_eq!(web(&w, "pick", json!({ "id": t, "open": true })).at("settings").at("canSet"), &json!(false));
}

#[test]
fn the_repo_new_tasks_go_to_is_picked_on_the_page_among_the_repos_the_board_knows() {
    let w = world();
    w.add("A task");
    let s = web(&w, "board", json!({}));
    let repos = s.at("list").at("head").arr("repos");
    assert!(repos.iter().any(|r| r.s("name") == "proj" && r.at("now") == &json!(true)), "{repos:?}");
    assert!(web_as(&w, "test", "repo", json!({ "to": "no-such-repo" })).unwrap_err().contains("No repo named"));
    assert!(web_as(&w, "test", "repo", json!({ "to": "../etc", "role": "watch" })).unwrap_err().contains("only watch"));
    let again = web(&w, "repo", json!({ "to": "proj" }));
    assert_eq!(again.at("list").at("head").s("where"), "proj");
    // A pane that is no pane's name, or none that herdr has, leaves it where it was.
    for bad in ["", "../../x", "w9:p9"] { assert_eq!(web(&w, "follow", json!({ "pane": bad })).at("list").at("head").s("where"), "proj", "{bad}"); }
}

#[test]
fn an_automation_is_made_on_the_page_and_a_name_or_schedule_the_board_cannot_take_is_refused() {
    let w = world();
    w.add("A task");
    let made = |args: Value| web_as(&w, "test", "automation", args);
    assert!(made(json!({ "name": " ", "prompt": "x", "schedule": "daily 09:00" })).unwrap_err().contains("name"));
    assert!(made(json!({ "name": "Late", "prompt": "x", "schedule": "whenever it suits" })).is_err());
    assert!(made(json!({ "name": "Look", "prompt": "x", "schedule": "daily 09:00", "role": "watch" })).unwrap_err().contains("only watch"));
    let ok = made(json!({ "name": "Check my pull requests", "prompt": "Look at the reviews on my pull requests", "schedule": "daily 09:00", "repo": "proj", "research": true })).unwrap();
    assert!(ok.s("said").contains("Added the automation \"Check my pull requests\""), "{}", ok.s("said"));
}

#[test]
fn an_accounts_row_says_which_program_its_role_runs_so_the_page_can_warn_where_it_is_not_claude() {
    let w = world();
    let id = w.add("Look into the logs");
    let s = web(&w, "pick", json!({ "id": id, "open": true }));
    assert_eq!((row(&s, "worker.account").s("program"), row(&s, "reviewer.account").s("program")), ("claude", "codex"));
    let s = set(&w, &id, "reviewer.agent", json!("claude"));
    assert_eq!(row(&s, "reviewer.account").s("program"), "claude");
    assert!(row(&s, "worker.effort").get("program").is_none(), "only an account row says it");
}

#[test]
fn a_new_coordinator_is_asked_for_only_for_a_level_of_this_machine_and_never_by_a_watcher() {
    let w = world();
    w.add("A task");
    assert!(web_as(&w, "test", "coordinator-new", json!({ "to": "project:no-such-repo" })).unwrap_err().contains("no coordinator"));
    assert!(web_as(&w, "test", "coordinator-new", json!({ "to": "../../x" })).unwrap_err().contains("no coordinator"));
    assert!(web_as(&w, "test", "coordinator-new", json!({ "to": "machine", "role": "watch" })).unwrap_err().contains("only watch"));
}

#[test]
fn an_account_is_set_for_several_repos_at_once_and_the_reviewer_is_put_on_claude_where_it_ran_another_program() {
    let w = world();
    let id = w.add("Look into the logs");
    std::fs::create_dir_all(sw::store::files::user_home().join(".claude-accounts/work")).unwrap();
    let go = |args: Value| web_as(&w, "test", "apply-account", args);
    assert!(go(json!({ "account": "nope", "roles": ["worker"], "repos": ["proj"] })).unwrap_err().contains("no account"));
    assert!(go(json!({ "account": "work", "roles": [], "repos": ["proj"] })).unwrap_err().contains("at least one role"));
    assert!(go(json!({ "account": "work", "roles": ["worker"], "repos": ["elsewhere"] })).unwrap_err().contains("No repo named"));
    assert!(go(json!({ "account": "work", "roles": ["worker"], "repos": ["proj"], "role": "watch" })).unwrap_err().contains("only watch"));
    let said = go(json!({ "account": "work", "roles": ["worker", "reviewer"], "repos": ["proj"], "claude": true })).unwrap().s("said").to_string();
    assert!(said.contains("Set work for worker, reviewer on 1 repo") && said.contains("proj reviewer"), "{said}");
    let s = web(&w, "pick", json!({ "id": id, "open": true }));
    assert_eq!((row(&s, "worker.account").s("value"), row(&s, "reviewer.account").s("value"), row(&s, "reviewer.agent").s("value")), ("work", "work", "claude"));
}

#[test]
fn accounts_are_listed_made_and_changed_on_the_page_the_cards_lists_follow_and_a_watcher_may_do_none_of_it() {
    let w = world();
    let id = w.add("A task");
    web(&w, "pick", json!({ "id": id, "open": true }));
    let dir = sw::store::files::user_home().join(".claude-accounts");
    let save = |a: Value| web_as(&w, "test", "account-save", a);
    let gw = |extra: Value| { let mut a = json!({ "new": true, "name": "gw", "type": "gateway", "baseUrl": "https://gw.example.com", "keyVar": "MY_KEY", "model": "m-one", "models": "m-one\nm-two", "context": "350000" }); a.as_object_mut().unwrap().extend(extra.as_object().unwrap().clone()); a };
    assert!(web_as(&w, "test", "accounts", json!({ "role": "watch" })).unwrap_err().contains("only watch"));
    assert!(save(gw(json!({ "role": "watch" }))).unwrap_err().contains("only watch") && !dir.join("gw").exists());
    for (bad, why) in [(json!({ "name": "../x" }), "name"), (json!({ "type": "" }), "Pick"), (json!({ "keyVar": "sk-ant-abc" }), "NAME"), (json!({ "models": ["a$(rm", "-rf", "~)"] }), "model"), (json!({ "context": "lots" }), "whole number")] {
        let e = save(gw(bad.clone())).unwrap_err(); assert!(e.contains(why), "{bad}: {e}"); assert!(!dir.join("gw").exists() && !dir.join("x").exists());
    }
    // Made: the account list has it with its fields, and the open task's lists offer it, then its models.
    let out = save(gw(json!({}))).unwrap();
    assert_eq!(out.s("said"), "Account \"gw\" made.");
    assert_eq!(out.at("accounts"), &json!([{ "name": "gw", "kind": "gateway", "gateway": { "baseUrl": "https://gw.example.com", "keyVar": "MY_KEY", "keyLiteral": false, "model": "m-one", "models": ["m-one", "m-two"], "context": 350000 } }]));
    let values = |s: &Value, name: &str| row(s, name).arr("words").iter().map(|x| x.t("value")).collect::<Vec<_>>();
    assert_eq!(values(&out, "worker.account"), ["none", "gw"]);
    set(&w, &id, "worker.account", json!("gw"));
    let out = save(gw(json!({ "new": false, "models": ["m-one", "m-three"] }))).unwrap();
    assert_eq!(values(&out, "worker.model"), ["none", "m-one", "m-three"]);
    assert!(save(gw(json!({}))).unwrap_err().contains("already"));
    // A login: made; then a gateway turned into one only when confirmed.
    save(json!({ "new": true, "name": "lg", "type": "login" })).unwrap();
    assert_eq!(web_as(&w, "test", "accounts", json!({})).unwrap().arr("accounts").iter().map(|a| (a.t("name"), a.t("kind"))).collect::<Vec<_>>(), [("gw".to_string(), "gateway".to_string()), ("lg".to_string(), "login".to_string())]);
    let to_login = |confirm: bool| save(json!({ "new": false, "name": "gw", "type": "login", "confirm": confirm }));
    assert!(to_login(false).unwrap_err().contains("Confirm") && dir.join("gw/account.env").is_file());
    to_login(true).unwrap();
    assert!(!dir.join("gw/account.env").exists());
    assert!(save(json!({ "new": false, "name": "nope", "type": "login" })).is_err());
}

fn gateway() -> sw::agents::account_edit::Gateway {
    sw::agents::account_edit::Gateway { base_url: "https://gw.example.com".into(), key_var: "MY_KEY".into(), model: "m-one".into(), models: vec!["m-one".into(), "m-two".into()], context: Some(350000), key_literal: false }
}

#[test]
fn a_gateway_is_made_read_and_changed_keeping_the_lines_it_does_not_own() {
    use sw::agents::account_edit::*;
    use std::os::unix::fs::PermissionsExt;
    let _w = world();
    use sw::store::files::user_home;
    create("gw", Some(&gateway())).unwrap();
    let env = user_home().join(".claude-accounts/gw/account.env");
    assert_eq!(read("gw").unwrap(), gateway());
    assert_eq!(std::fs::metadata(&env).unwrap().permissions().mode() & 0o777, 0o600);
    assert_eq!((sw::agents::account::kind_of("gw"), sw::agents::account::models_of("gw")), ("gateway", vec!["m-one".to_string(), "m-two".to_string()]));
    // A file written by hand: its comment, an `export`, a key written out and lines this does not own.
    std::fs::write(&env, "# mine\nexport ANTHROPIC_BASE_URL='http://old'\nANTHROPIC_AUTH_TOKEN=\"sk-literal\"\nFOO=bar\nCLAUDE_CODE_AUTO_COMPACT_WINDOW=300000\nCLAUDE_CODE_MAX_CONTEXT_TOKENS=1000\nACCOUNT_MODELS=\"a b\"\n").unwrap();
    let was = read("gw").unwrap();
    assert_eq!((was.base_url.as_str(), was.key_var.as_str(), was.key_literal, was.models.clone(), was.context), ("http://old", "", true, vec!["a".to_string(), "b".to_string()], Some(1000)));
    let next = Gateway { base_url: "https://new.example.com".into(), key_var: "NEW_KEY".into(), model: String::new(), models: vec!["a".into(), "c".into()], context: None, key_literal: false };
    save_gateway("gw", &next).unwrap();
    let text = std::fs::read_to_string(&env).unwrap();
    assert_eq!(text, "# mine\nANTHROPIC_BASE_URL=\"https://new.example.com\"\nANTHROPIC_AUTH_TOKEN=\"$NEW_KEY\"\nFOO=bar\nCLAUDE_CODE_AUTO_COMPACT_WINDOW=300000\nACCOUNT_MODELS=\"a c\"\n");
    assert_eq!(read("gw").unwrap(), next);
    save_gateway("gw", &next).unwrap();
    assert_eq!(std::fs::read_to_string(&env).unwrap(), text, "saved again as it is: nothing changes");
    assert_eq!(std::fs::metadata(&env).unwrap().permissions().mode() & 0o777, 0o600);
    assert!(std::fs::read_dir(env.parent().unwrap()).unwrap().all(|e| !e.unwrap().file_name().to_string_lossy().ends_with(".tmp")));
}

#[test]
fn an_account_is_refused_for_a_path_for_a_name_there_and_for_a_value_that_would_inject_shell() {
    use sw::agents::account_edit::*;
    let _w = world();
    use sw::store::files::user_home;
    let accounts = user_home().join(".claude-accounts");
    for bad in ["../x", "a/b", "", ".hid", "Up", "-x", "a b", &"a".repeat(33)] { assert!(create(bad, None).is_err(), "{bad:?}"); }
    assert!(!user_home().join("x").exists());
    create("one", None).unwrap();
    assert!(create("one", None).unwrap_err().msg.contains("already"));
    let g = gateway();
    let worse = [
        Gateway { models: vec!["a$(rm".into(), "-rf".into(), "~)".into()], ..g.clone() }, Gateway { models: vec!["a$(rm -rf ~)".into()], ..g.clone() }, Gateway { model: "a`id`".into(), ..g.clone() },
        Gateway { base_url: "https://x.com\"; rm -rf ~; \"".into(), ..g.clone() }, Gateway { base_url: "ftp://x.com".into(), ..g.clone() }, Gateway { base_url: "https://x.com\nA=1".into(), ..g.clone() },
        Gateway { key_var: "A B".into(), ..g.clone() }, Gateway { key_var: "1KEY".into(), ..g.clone() }, Gateway { key_var: "$(id)".into(), ..g.clone() }, Gateway { key_var: "sk-ant-abc".into(), ..g.clone() },
        Gateway { key_var: String::new(), ..g.clone() }, Gateway { context: Some(0), ..g.clone() },
    ];
    for (n, bad) in worse.iter().enumerate() { assert!(create("bad", Some(bad)).is_err() && !accounts.join("bad").exists(), "{n}: {bad:?}"); assert!(save_gateway("one", bad).is_err(), "{n}"); }
    assert!(!accounts.join("one/account.env").exists());
    for path in ["../x", "..", "one/../one", "/etc"] { assert!(read(path).is_err() && save_gateway(path, &g).is_err() && make_login(path, true).is_err(), "{path}"); }
}

#[test]
fn a_login_is_a_gateway_by_its_account_env_and_a_gateway_a_login_again_only_when_confirmed() {
    use sw::agents::account::kind_of;
    use sw::agents::account_edit::*;
    let _w = world();
    use sw::store::files::user_home;
    create("lg", None).unwrap();
    assert!(read("lg").is_err());
    save_gateway("lg", &gateway()).unwrap();
    let env = user_home().join(".claude-accounts/lg/account.env");
    assert!(std::fs::read_to_string(&env).unwrap().starts_with("# Written by herdr-projects"));
    assert_eq!(kind_of("lg"), "gateway");
    create("plain", None).unwrap();
    assert_eq!(list().iter().map(|a| (a.name.as_str(), a.kind, a.gateway.is_some())).collect::<Vec<_>>(), [("lg", "gateway", true), ("plain", "login", false)]);
    assert!(make_login("lg", false).unwrap_err().msg.contains("Confirm") && env.is_file());
    make_login("lg", true).unwrap();
    assert_eq!((kind_of("lg"), env.exists()), ("login", false));
    make_login("lg", false).unwrap();
}
