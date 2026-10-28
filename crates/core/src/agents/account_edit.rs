//! Managing Claude Code accounts from the web board: the accounts there are, a new one made, and a gateway's fields
//! (base URL, the NAME of the variable that holds its key, default model, model list, context size) read from and
//! written to its `account.env`, keeping every line that is not one of those. The file is SOURCED by the pane's
//! shell, so every value is checked before it is written and quoted when it is. A key itself is never taken or kept.
use super::account::{choices, kind_of, ACCOUNTS};
use crate::err::Res;
use crate::refuse;
use crate::store::files::user_home;
use std::io::Write;
use std::os::unix::fs::{DirBuilderExt, OpenOptionsExt};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, Ordering};

/// What a gateway's `account.env` says of its own: `key_var` is the name of the variable the shell exports the key as
/// (`key_literal`: the file holds a key written out, which is never shown and goes when the file is saved).
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Gateway { pub base_url: String, pub key_var: String, pub model: String, pub models: Vec<String>, pub context: Option<u64>, pub key_literal: bool }
/// An account folder: its name, what it is (`login`, `gateway`, as `account::kind_of` says) and, for a gateway, its fields.
#[derive(Debug, Clone)]
pub struct Account { pub name: String, pub kind: &'static str, pub gateway: Option<Gateway> }

const ENV: &str = "account.env";
const CONTEXT: &str = "CLAUDE_CODE_MAX_CONTEXT_TOKENS";
const PREAMBLE: &str = "# Written by herdr-projects (Manage accounts). The key is read from the shell's own variable: none is kept here.\nunset ANTHROPIC_API_KEY CLAUDE_CODE_OAUTH_TOKEN CLAUDE_CODE_OAUTH_REFRESH_TOKEN\n";
static TMP: AtomicU64 = AtomicU64::new(0);

fn root() -> PathBuf { user_home().join(ACCOUNTS) }
fn ident(s: &str) -> bool { s.starts_with(|c: char| c.is_ascii_alphabetic() || c == '_') && s.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') }
/// A model name: no quote, space or shell metacharacter in it.
fn word(w: &str) -> bool { w.len() <= 100 && w.starts_with(|c: char| c.is_ascii_alphanumeric()) && w.chars().all(|c| c.is_ascii_alphanumeric() || "._:/@+-".contains(c)) }
fn short(s: &str) -> String { s.chars().take(40).collect() }

/// A new account's name: 1 to 32 of a to z, 0 to 9, `_` and `-`, not starting with `_` or `-`. So never a path.
pub fn check_name(name: &str) -> Res<()> {
    let ok = !name.is_empty() && name.len() <= 32 && name.chars().enumerate().all(|(i, c)| c.is_ascii_lowercase() || c.is_ascii_digit() || (i > 0 && (c == '_' || c == '-')));
    if !ok { refuse!("An account's name is 1 to 32 letters (a to z), digits, _ or -, starting with a letter or digit. \"{}\" is not one.", short(name)); }
    Ok(())
}
/// The folder of an account that is there: a folder of `~/.claude-accounts/` by its bare name (also one made by hand,
/// whatever its name, but never a path).
fn dir_of(name: &str) -> Res<PathBuf> {
    let plain = !name.is_empty() && !name.starts_with('.') && !name.contains(['/', '\\']) && !name.chars().any(char::is_control);
    let d = root().join(name);
    if !plain || !d.is_dir() { refuse!("There is no account \"{}\".", short(name)); }
    Ok(d)
}
impl Gateway {
    /// Refuses what could not be written into the sourced file safely, or what no gateway takes, in words for the form.
    pub fn check(&self) -> Res<()> {
        let u = &self.base_url;
        let host = u.strip_prefix("https://").or_else(|| u.strip_prefix("http://")).unwrap_or("");
        if !host.starts_with(|c: char| c.is_ascii_alphanumeric()) || u.len() > 300 || !u.chars().all(|c| c.is_ascii_alphanumeric() || "-._~:/?&=%+@,#".contains(c)) {
            refuse!("The base URL must start with http:// or https:// and hold no space, quote or shell character.");
        }
        if !ident(&self.key_var) || self.key_var.len() > 64 { refuse!("The key variable is the NAME of an environment variable (letters, digits and _, not starting with a digit), such as MY_GATEWAY_KEY. Never the key itself."); }
        if !self.model.is_empty() && !word(&self.model) { refuse!("The default model \"{}\" is not a model name: letters, digits and . _ : / @ + - only.", short(&self.model)); }
        if self.models.len() > 50 { refuse!("At most 50 models."); }
        if let Some(m) = self.models.iter().find(|m| !word(m)) { refuse!("The model \"{}\" is not a model name: letters, digits and . _ : / @ + - only, separated by spaces.", short(m)); }
        if self.context.is_some_and(|n| n == 0 || n > 100_000_000) { refuse!("The context tokens are a whole number from 1 to 100000000, or left empty."); }
        Ok(())
    }
}
/// `KEY=value` of a line (a leading `export` allowed); None for a comment, a blank or anything that is not an assignment.
fn assignment(line: &str) -> Option<(&str, &str)> {
    let l = line.trim_start();
    let (k, v) = l.strip_prefix("export ").unwrap_or(l).trim_start().split_once('=')?;
    ident(k).then_some((k, v))
}
/// A value as the shell would read it here: inside one pair of quotes, or up to a space or comment.
fn unquote(v: &str) -> String {
    let v = v.trim_start();
    match v.chars().next() {
        Some(q @ ('"' | '\'')) => v[1..].split(q).next().unwrap_or("").to_string(),
        _ => v.split(|c: char| c.is_whitespace() || c == '#').next().unwrap_or("").to_string(),
    }
}
fn parse(text: &str) -> Gateway {
    let mut g = Gateway::default();
    for (k, v) in text.lines().filter_map(assignment) {
        let v = unquote(v);
        match k {
            "ANTHROPIC_BASE_URL" => g.base_url = v,
            "ANTHROPIC_AUTH_TOKEN" => {
                let name = v.strip_prefix('$').map(|r| r.trim_start_matches('{').trim_end_matches('}')).filter(|n| ident(n));
                g.key_literal = name.is_none() && !v.is_empty();
                g.key_var = name.unwrap_or("").to_string();
            }
            "ANTHROPIC_MODEL" => g.model = v,
            "ACCOUNT_MODELS" => g.models = v.split_whitespace().map(String::from).collect(),
            CONTEXT => g.context = v.parse().ok(),
            _ => {}
        }
    }
    g
}
/// The text of `account.env` with the lines this owns set from `g`: a line that already says it stays as it is, one
/// that says otherwise is rewritten in place, a missing one is added at the end, one now empty is dropped. Every other
/// line, comments included, stays where it is.
fn apply(text: &str, g: &Gateway) -> String {
    let want: [(&str, Option<String>); 5] = [
        ("ANTHROPIC_BASE_URL", Some(g.base_url.clone())),
        ("ANTHROPIC_AUTH_TOKEN", Some(format!("${}", g.key_var))),
        ("ANTHROPIC_MODEL", Some(g.model.clone()).filter(|m| !m.is_empty())),
        ("ACCOUNT_MODELS", Some(g.models.join(" ")).filter(|m| !m.is_empty())),
        (CONTEXT, g.context.map(|n| n.to_string())),
    ];
    let mut lines: Vec<Option<String>> = text.lines().map(|l| Some(l.to_string())).collect();
    for (key, plain) in want {
        let at: Vec<usize> = lines.iter().enumerate().filter(|(_, l)| l.as_deref().and_then(assignment).is_some_and(|(k, _)| k == key)).map(|(i, _)| i).collect();
        let line = plain.as_ref().map(|p| if key == CONTEXT { format!("{key}={p}") } else { format!("{key}=\"{p}\"") });
        match at.first() {
            Some(&i) => {
                let same = lines[i].as_deref().and_then(assignment).map(|(_, v)| unquote(v)) == plain;
                if !same { lines[i] = line; }
                for &j in &at[1..] { lines[j] = None; } // the shell would have read the last one: one is left
            }
            None => lines.extend(line.map(Some)),
        }
    }
    lines.into_iter().flatten().map(|l| l + "\n").collect()
}
/// Written beside it and renamed over it, readable by its owner only.
fn write_env(dir: &Path, text: &str) -> Res<()> {
    let tmp = dir.join(format!(".account.env.{}.{}.tmp", std::process::id(), TMP.fetch_add(1, Ordering::Relaxed)));
    let mut f = std::fs::OpenOptions::new().write(true).create_new(true).mode(0o600).open(&tmp)?;
    f.write_all(text.as_bytes()).and_then(|_| f.sync_all()).and_then(|_| std::fs::rename(&tmp, dir.join(ENV))).inspect_err(|_| { let _ = std::fs::remove_file(&tmp); })?;
    Ok(())
}

/// Every account folder with what it is, and a gateway's fields, in name order.
pub fn list() -> Vec<Account> {
    choices().into_iter().skip(1).map(|name| {
        let kind = kind_of(&name);
        let gateway = (kind == "gateway").then(|| parse(&std::fs::read_to_string(root().join(&name).join(ENV)).unwrap_or_default()));
        Account { name, kind, gateway }
    }).collect()
}
/// A gateway's fields as its `account.env` says them.
pub fn read(name: &str) -> Res<Gateway> {
    let env = dir_of(name)?.join(ENV);
    if !env.is_file() { refuse!("The account \"{}\" is a Claude login, not a gateway.", short(name)); }
    Ok(parse(&std::fs::read_to_string(env)?))
}
/// A new account folder under `~/.claude-accounts/`: a Claude login (no `account.env`; log in once in a terminal) or,
/// with `gateway`, a gateway. A name already there is refused.
pub fn create(name: &str, gateway: Option<&Gateway>) -> Res<()> {
    check_name(name)?;
    if let Some(g) = gateway { g.check()?; }
    let dir = root().join(name);
    std::fs::DirBuilder::new().recursive(true).mode(0o700).create(root())?;
    match std::fs::DirBuilder::new().mode(0o700).create(&dir) {
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => refuse!("There is already an account \"{name}\"."),
        r => r?,
    }
    if let Some(g) = gateway { write_env(&dir, &apply(PREAMBLE, g)).inspect_err(|_| { let _ = std::fs::remove_dir(&dir); })?; }
    Ok(())
}
/// A gateway's fields saved: only the lines of its `account.env` that are these change. A login gets an `account.env`
/// and is a gateway from then on.
pub fn save_gateway(name: &str, g: &Gateway) -> Res<()> {
    g.check()?;
    let dir = dir_of(name)?;
    let text = if dir.join(ENV).is_file() { std::fs::read_to_string(dir.join(ENV))? } else { PREAMBLE.to_string() };
    write_env(&dir, &apply(&text, g))
}
/// A gateway made a Claude login again: its `account.env` removed, and with it every line in it, so only with `confirm`.
pub fn make_login(name: &str, confirm: bool) -> Res<()> {
    let env = dir_of(name)?.join(ENV);
    if !env.is_file() { return Ok(()); }
    if !confirm { refuse!("Making \"{}\" a Claude login deletes its account.env (its base URL, key variable, models and any other line in it). Confirm to go on.", short(name)); }
    std::fs::remove_file(env)?;
    Ok(())
}
