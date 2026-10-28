//! An account for several repos at once, from the web board's sheet: the account is set as each repo's own entry for the
//! roles asked (`<role>.account`, the coordinator's as that project coordinator's own), and where a role runs another
//! program than Claude Code it can be put on Claude, since an account is for Claude Code only.
use crate::json::*;
use crate::store::settings::change::set_value;
use crate::store::settings::coord::Coord;
use crate::store::settings::lookup::Scope;

const ROLES: [&str; 3] = ["worker", "reviewer", "coordinator"];

/// The program a repo's role runs now (its entry, else yours, else the built-in), by `show`'s own lookup.
fn program_of(repo: &str, role: &str) -> String {
    let rows = crate::board::data::mine(repo);
    let found = rows.arr("settings").iter().find(|r| r.s("name") == format!("{role}.agent")).map(|r| text(r.at("value"))).unwrap_or_default();
    crate::store::settings::values::agent_of(&found).kind
}
/// Apply what the sheet asked: `account` (a name from the account folders, or `none`), `roles`, `repos` (names the board
/// knows), and `claude` (put a role on Claude Code where it runs another program). The words for the page, or why not.
pub fn apply(args: &Value) -> Result<String, String> {
    let account = args.s("account").trim().to_string();
    if account != "none" && !crate::agents::account::choices().contains(&account) { return Err(format!("There is no account \"{account}\".")); }
    let roles: Vec<String> = args.arr("roles").iter().filter_map(|r| r.as_str()).filter(|r| ROLES.contains(r)).map(String::from).collect();
    if roles.is_empty() { return Err("Choose at least one role.".into()); }
    let known = crate::store::repos::known_repos();
    let repos: Vec<String> = args.arr("repos").iter().filter_map(|r| r.as_str()).map(String::from).collect();
    if repos.is_empty() { return Err("Choose at least one repo.".into()); }
    if let Some(bad) = repos.iter().find(|r| !known.contains_key(r.as_str())) { return Err(format!("No repo named \"{bad}\".")); }
    let socket = crate::herdr::cli::current_socket();
    let mut moved = vec![];
    for repo in &repos {
        for role in &roles {
            if role == "coordinator" {
                let key = crate::coordinator::co::Co::of(&socket, &format!("project:{repo}")).settings_key();
                set_value("coordinator.account", &json!(account), &Scope::coord(Coord::of(&key))).map_err(|e| e.msg)?;
                continue;
            }
            set_value(&format!("{role}.account"), &json!(account), &Scope::repo(repo)).map_err(|e| e.msg)?;
            if args.b("claude") && account != "none" && program_of(repo, role) != "claude" {
                set_value(&format!("{role}.agent"), &json!("claude"), &Scope::repo(repo)).map_err(|e| e.msg)?;
                moved.push(format!("{repo} {role}"));
            }
        }
    }
    let on = if moved.is_empty() { String::new() } else { format!(" Put on Claude Code: {}.", moved.join(", ")) };
    Ok(format!("Set {account} for {} on {} repo{}.{on}", roles.join(", "), repos.len(), if repos.len() == 1 { "" } else { "s" }))
}
