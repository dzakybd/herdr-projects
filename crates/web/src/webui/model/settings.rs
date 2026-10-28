//! The settings for the web board: every work setting of the open task (or automation) as a row the page can change.
//! The rows are the terminal board's settings page (board/update/edit.rs): the same settings, in the same order, each
//! with its plain label, what it means, its value, whether it is set for this task only, and what it may be. Nothing is decided
//! here: a change goes back through the board's own save (webui/web.rs: set_setting).
use crate::board::state::{Editing, State, Whose};
use crate::board::update::edit::{head_scope, rows, value_say, words_of};
use crate::json::*;
use crate::store::settings_list::entry_of;

/// What a role's model may be, by the account its row says it runs under: `none` and the models of that account. Typed
/// where the account names none.
fn model_words(x: &Obj, list: &[Obj]) -> Option<Vec<Value>> {
    let role = x.s("name").split('.').next().unwrap_or("");
    let account = list.iter().find(|r| r.s("name") == format!("{role}.account")).map(|r| text(r.at("value"))).unwrap_or_default();
    let models = crate::agents::account::models_of(&account);
    if models.is_empty() { return None; }
    Some(std::iter::once("none".to_string()).chain(models).map(|v| json!(v)).collect())
}
/// One setting as a row of the page. An account is one of the folders there are, with the one it has now among them
/// even when it is a path that is not.
fn row_of(x: &Obj, list: &[Obj]) -> Value {
    let e = entry_of(x.s("name"));
    let mut words = if x.s("name").ends_with(".model") { model_words(x, list) } else { words_of(x.s("name")) };
    if let Some(ws) = words.as_mut() { if (x.s("name").ends_with(".account") || x.s("name").ends_with(".model")) && !ws.contains(x.at("value")) { ws.push(x.at("value").clone()); } }
    let from = [x.t("note"), x.t("later")].into_iter().filter(|w| !w.is_empty()).collect::<Vec<_>>().join(" · ");
    // An account says what it is (a login, or a gateway of its own), for the page to show beside the choice.
    let tags: Option<serde_json::Map<String, Value>> = words.as_ref().filter(|_| x.s("name").ends_with(".account")).map(|ws| ws.iter().map(|w| (text(w), json!(crate::agents::account::kind_of(&text(w))))).collect());
    let mut row = json!({
        "name": x.s("name"), "label": e.map(|e| e.label.to_string()).filter(|l| !l.is_empty()).unwrap_or_else(|| x.t("name")), "means": e.map(|e| e.means.to_string()).unwrap_or_default(), "own": x.b("own"),
        "value": crate::re!(r"\s*\n\s*").replace_all(&value_say(x.at("value")), " / "),
        // What a reader must know of it: why it does nothing here, or that it counts later. Never where it comes from.
        "from": from,
        // One of a few words: the page offers them. Anything else is typed, and `hint` says what it may be.
        "words": words.as_ref().map(|ws| ws.iter().map(|w| json!({ "value": text(w), "say": if w == "none" && x.s("name").ends_with(".model") { "default".to_string() } else { value_say(w) }, "now": w == x.at("value") })).collect::<Vec<_>>()),
        "hint": if words.is_some() { String::new() } else { crate::board::view::panel::page::value_words(x) },
    });
    if let Some(t) = tags { row["tags"] = json!(t); }
    row
}
/// Your settings, or your entry for a repo, for the heading that is open: the rows of the terminal board's page for it
/// (`whose_here`), read now. Null for a heading of another machine, whose are changed there.
pub fn scope_settings_of(s: &mut State) -> Value {
    let Ok(w) = head_scope(s) else { return Value::Null };
    let was = (s.editing.replace(Editing { id: s.sel.clone(), whose: w.clone(), ..Default::default() }), s.data.mine.take());
    s.data.mine = Some((w.wanted(), crate::board::data::mine(&w.wanted())));
    let list = rows(s);
    (s.editing, s.data.mine) = was;
    let whose = if let Whose::Repo(_) = w { "this project" } else { "your defaults" };
    json!({ "id": s.sel, "whose": whose, "canSet": true, "show": s.web.show_settings, "rows": list.iter().map(|x| row_of(x, &list)).collect::<Vec<_>>() })
}
/// `auto`: `r` is an automation. None for a row that has no settings (a message that waits for the Coordinator).
pub fn settings_of(s: &mut State, r: &Obj, auto: bool) -> Value {
    if !auto && !r.has("title") { return Value::Null; }
    let whose = if auto { "this automation" } else { "this task" };
    // An automation's are every work setting its next task will have, as it is; a task's are its settings page's.
    let list: Vec<Obj> = if auto { r.at("shown").arr("settings").iter().filter_map(|x| x.as_object().cloned()).collect() } else {
        let was = s.editing.replace(Editing { id: r.t("id"), ..Default::default() });
        let l = rows(s);
        s.editing = was;
        l
    };
    let rows: Vec<Value> = list.iter().map(|x| row_of(x, &list)).collect();
    // Read-only: an archived task, and another machine's automation whose machine did not send its settings.
    let can_set = !r.b("archivedAt") && (!auto || crate::board::update::autos::can_set(r));
    // `show`: counts up each time the board is asked for this part by its key, so the page opens it and goes there.
    json!({ "id": if auto { r.t("row") } else { r.t("id") }, "whose": whose, "canSet": can_set, "show": s.web.show_settings, "rows": rows })
}
