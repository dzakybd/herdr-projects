// Browser notifications for the web board: a bell in its header that, once you turn it on, shows a notification when a
// task newly needs you, is ready to approve or failed (the events of the `board.notify` setting, which the board works
// out and puts on each row as `event`). Each snapshot is compared with the one before; a task is told once for each
// event it comes into. Nothing leaves this browser: it is the page's own Notification API, so it works while the board
// is open (a tab, or the installed web app), not after it is closed.
/* global window, document, localStorage, Notification, navigator, setTimeout, clearTimeout */
import { h, icon } from "./dom.js";

const KEY = "sw-notify", WORDS = { "needs you": "needs you", "ready to approve": "is ready to approve", failed: "failed" };
let wanted = false, seen = null, view = "", pick = () => {}, button = null, note = null, hide = 0;

const have = () => typeof window.Notification === "function";
const stored = () => { try { return localStorage.getItem(KEY) === "1"; } catch { return false; } };
const store = (on) => { try { localStorage.setItem(KEY, on ? "1" : "0"); } catch {} };
const granted = () => have() && Notification.permission === "granted";
// On: asked for, and the browser lets it.
const active = () => wanted && granted();

// Said in a small note under the header (it goes by itself, or on a tap): where notifications cannot work, and why.
function say(text) {
  clearTimeout(hide);
  note.textContent = text; note.hidden = !text;
  if (text) hide = setTimeout(() => say(""), 12000);
}
function paint() {
  const on = active();
  button.classList.toggle("is-on", on);
  button.setAttribute("aria-pressed", String(on));
  const says = on ? "Notifications are on: tap to turn them off" : "Tell me when a task needs me";
  button.title = says; button.setAttribute("aria-label", says);
  button.replaceChildren(icon(on ? "bell" : "bellOff"));
}
async function ask() {
  try { const p = Notification.requestPermission(); return await p; } catch { return Notification.permission; }
}
async function toggle() {
  if (active()) { wanted = false; store(false); seen = null; paint(); return say(""); }
  if (!have()) return say("This browser has no notifications here. On an iPhone they work only in the web app added to the Home Screen (iOS 16.4 or later), not in a Safari tab.");
  if (!window.isSecureContext) return say("Browsers show notifications only on a page served over https (or localhost); this one is not.");
  const p = Notification.permission === "default" ? await ask() : Notification.permission;
  wanted = p === "granted"; store(wanted); seen = null; paint();
  say(wanted ? "" : p === "denied" ? "Notifications are blocked for this site: allow them in the browser's site settings, then tap the bell again." : "Notifications were not allowed.");
}
// The header's bell, with its note: `onPick(id)` selects a task (the notification's tap).
export function notifyEl(onPick) {
  pick = onPick; wanted = stored();
  button = h("button.icon-button.sw-notify-btn", { type: "button", "aria-pressed": "false", onclick: toggle });
  note = h("p.sw-notify-note", { role: "status", hidden: true, onclick: () => say("") });
  paint();
  return h("span.sw-notify", button, note);
}
// Whether the board should keep asking while its page is hidden: a hidden tab is exactly where a notification is for.
export const watching = () => active();

function show(r) {
  const label = r.tag || r.label || "Task", title = `${label} ${WORDS[r.event]}`, opts = { body: r.title || "", tag: `${r.id}:${r.event}` };
  try {
    const n = new Notification(title, opts);
    n.onclick = () => { try { window.focus(); n.close(); } catch {} pick(r.id); };
  } catch {
    // Where the constructor is not allowed (a phone's Chrome): through the page's service worker, if it has one. That
    // one cannot answer a tap, so it only shows.
    try { navigator.serviceWorker?.getRegistration().then((reg) => reg?.showNotification(title, opts)).catch(() => {}); } catch {}
  }
}
// Called with each new snapshot. The first one only says what is so already, and so does the first after the view moved
// (another machine or filter, a group folded or unfolded): what comes into view is not news.
export function watch(snap) {
  if (!active()) { seen = null; return; }
  const rows = snap.list.groups.flatMap((g) => g.rows).filter((r) => r.id);
  const f = snap.flags, key = JSON.stringify([f.archive, f.byRepo, f.byMachine, f.allDone, f.scope, f.filter, snap.list.groups.map((g) => [g.id, g.folded])]);
  const was = seen, same = key === view;
  seen = new Map(rows.map((r) => [r.id, r.event || ""])); view = key;
  if (!was || !same) return;
  for (const r of rows) if (r.event && was.get(r.id) !== r.event) show(r);
}
