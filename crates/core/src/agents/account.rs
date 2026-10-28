//! `<role>.account`: the Claude Code account a role's agent runs under, given to its pane as the config folder, and
//! the environment of the account's own `account.env` where it has one (a gateway's base URL and key, a model).
use crate::err::Res;
use crate::store::files::user_home;
use std::path::PathBuf;

/// The folder of accounts a bare name is looked for in, under the user's home.
const ACCOUNTS: &str = ".claude-accounts";

/// The config folder an account names: a path (`~/work-account`, or an absolute one), or a bare name, which is a
/// folder of `~/.claude-accounts/`. None for `none` or an empty value.
pub fn folder(account: &str) -> Option<PathBuf> {
    let a = account.trim();
    if a.is_empty() || a.eq_ignore_ascii_case("none") { return None; }
    if let Some(rest) = a.strip_prefix("~/") { return Some(user_home().join(rest)); }
    if a.contains('/') { return Some(PathBuf::from(a)); }
    Some(user_home().join(ACCOUNTS).join(a))
}
/// What the settings page offers for `<role>.account`: `none`, then each account folder of `~/.claude-accounts/` by its
/// name, in order.
pub fn choices() -> Vec<String> {
    let mut names: Vec<String> = std::fs::read_dir(user_home().join(ACCOUNTS)).into_iter().flatten().flatten()
        .filter(|e| e.path().is_dir()).filter_map(|e| e.file_name().into_string().ok()).filter(|n| !n.starts_with('.')).collect();
    names.sort();
    std::iter::once("none".to_string()).chain(names).collect()
}
/// The line that gives a pane's shell the account, before the program starts there (a leading space keeps it out of
/// the shell's history where that is set). An `account.env` in the folder is sourced after it, by the pane's own
/// shell, so it may name variables that shell already has (`ANTHROPIC_AUTH_TOKEN=\"$MODEL_IQ_API_KEY\"`) and the board
/// never holds a key. None where no account is set or the program is not Claude Code, which is the only one with
/// accounts of this kind. A folder that is not there is refused: a typo would otherwise start a
/// new, logged-out account.
pub fn export_line(kind: &str, account: &str) -> Res<Option<String>> {
    let Some(dir) = folder(account).filter(|_| kind == "claude") else { return Ok(None) };
    if !dir.is_dir() { crate::fail!("the account \"{}\" has no folder {}. Set the role's account to another, or none.", account.trim(), dir.display()); }
    let label = dir.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    let mut line = format!(" export CLAUDE_CONFIG_DIR={} CLAUDE_ACCOUNT_LABEL={}", crate::util::shq(&dir.display().to_string()), crate::util::shq(&label));
    let env = dir.join("account.env");
    if env.is_file() { line.push_str(&format!("; set -a; . {}; set +a", crate::util::shq(&env.display().to_string()))); }
    Ok(Some(line))
}
