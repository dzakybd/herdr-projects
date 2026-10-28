//! Which repos have a workspace open in herdr, and which repo one pane is in: the web board follows the workspace the
//! web UI shows (web.rs: the `follow` call), and lists the repos with one open (model/list.rs).
use crate::json::*;
use std::sync::Mutex;

/// The repos with a workspace open in herdr: the root of each pane's folder, read from herdr at most every few seconds.
pub fn open_repos() -> Vec<std::path::PathBuf> {
    static SEEN: Mutex<(i64, Vec<std::path::PathBuf>)> = Mutex::new((0, Vec::new()));
    let now = crate::clock::now();
    let mut g = SEEN.lock().unwrap();
    if now - g.0 < 5000 && g.0 != 0 { return g.1.clone(); }
    let panes = crate::herdr::Herdr::new(None).panes(None).unwrap_or_default();
    g.1 = panes.iter().filter_map(|p| {
        let at = if p.s("foreground_cwd").is_empty() { p.t("cwd") } else { p.t("foreground_cwd") };
        if at.is_empty() { None } else { std::fs::canonicalize(crate::store::repos::primary_root(std::path::Path::new(&at))).ok() }
    }).collect();
    g.0 = now;
    g.1.clone()
}
/// The folder of a repo the board knows that a herdr pane is in (its folder now, or that of what runs there); None for
/// a pane in no git repo, one that is gone, or a name that is no pane's. A repo not seen before is named by this.
pub fn repo_of_pane(pane: &str) -> Option<String> {
    if pane.is_empty() || pane.len() > 40 || !pane.chars().all(|ch| ch.is_ascii_alphanumeric() || ":-_".contains(ch)) { return None; }
    let p = crate::herdr::Herdr::new(None).pane_get(pane)?;
    let at = if p.s("foreground_cwd").is_empty() { p.t("cwd") } else { p.t("foreground_cwd") };
    if at.is_empty() { return None; }
    let root = std::fs::canonicalize(crate::store::repos::primary_root(std::path::Path::new(&at))).ok()?;
    if let Some(known) = crate::store::repos::known_repos().values().map(text).find(|d| std::fs::canonicalize(d).ok().as_deref() == Some(root.as_path())) { return Some(known); }
    // A git repo not seen before is named now, as the first task added in it would (never the home folder).
    if root == std::fs::canonicalize(crate::store::files::user_home()).ok()? || !crate::store::repos::is_repo(&root) { return None; }
    crate::store::repos::repo_of(&root).ok().map(|(_, dir)| dir.display().to_string())
}
