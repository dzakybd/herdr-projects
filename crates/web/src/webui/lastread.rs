//! What the web board keeps of its reads for a moment, for every page: the last full read (a page that opens starts from
//! it, and reads again on a thread of its own, where it used to wait for a whole read of git and herdr), and the closer
//! look at a task (a tap on it again within a few seconds keeps what was read). A test answers for herdr in its own
//! thread, where a read is never kept: it must see what it just did.
use crate::board::data::Data;
use crate::json::*;
use std::collections::HashMap;
use std::sync::Mutex;

/// How old a full read may be for a page that opens to start from it.
const FRESH: i64 = 60_000;
/// How long a task's closer look is kept.
const SNAP: i64 = 5_000;
static LAST: Mutex<Option<(i64, Data)>> = Mutex::new(None);
static SNAPS: Mutex<Option<HashMap<String, (i64, Value)>>> = Mutex::new(None);

/// A full read, kept.
pub fn keep(d: &Data) { *LAST.lock().unwrap() = Some((crate::clock::now(), d.clone())); }

/// The last full read for a page that opens, without what belongs to another page's own asks (its coordinator's look,
/// its settings page, its skills): None when it is old, or when a test is running.
pub fn recent() -> Option<Data> {
    if crate::herdr::cli::fake().is_some() { return None; }
    let g = LAST.lock().unwrap();
    let (at, d) = g.as_ref()?;
    if crate::clock::now() - *at > FRESH { return None; }
    let mut d = d.clone();
    (d.sub, d.mine, d.skills) = (None, None, None);
    Some(d)
}

/// The closer look at a task: the one kept a moment ago, else `read`.
pub fn snap_of(id: &str, read: impl FnOnce() -> Value) -> Value {
    if crate::herdr::cli::fake().is_some() { return read(); }
    let now = crate::clock::now();
    if let Some((at, v)) = SNAPS.lock().unwrap().as_ref().and_then(|m| m.get(id)) { if now - *at < SNAP { return v.clone(); } }
    let v = read();
    let mut g = SNAPS.lock().unwrap();
    let m = g.get_or_insert_with(HashMap::new);
    m.retain(|_, (at, _)| now - *at < SNAP);
    m.insert(id.to_string(), (now, v.clone()));
    v
}
