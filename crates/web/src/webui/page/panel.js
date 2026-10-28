// One task, whole: its title, the buttons for what can be done to it now, and the task panel's parts (status, what
// it needs from you, description, reviews, evidence, what the agent said, settings, session). The
// parts and the buttons come from the board (webui/panel-model.js, webui/side-model.js); this file lays them out.
// Under it, All settings: every work setting of the task (or automation) with its plain label, what it means, its
// value and the place it comes from, each one changed right there (webui/settings-model.js).
import { h, icon, ink } from "./dom.js";

const cap = (s) => (s ? s[0].toUpperCase() + s.slice(1) : s);
// Buttons that end something are not the ones to hit by mistake.
const QUIET = ["stop", "archive", "back"];

function item(x, onGo) {
  if (x.k === "gap") return h("div.sw-gap");
  // A firing of an automation that made a task: the line goes to that task.
  if (x.k === "text" && x.go) return h("button.sw-text.sw-go", { type: "button", title: `Open ${x.go}`, onclick: () => onGo?.(x.go) }, ink(x.value));
  // A line the board set in under another (a finding's further lines under its first) stays set in when it wraps.
  if (x.k === "text") { const lead = /^ */.exec(x.value[0]?.t || "")[0].length; return h("p.sw-text", lead ? { style: `padding-left:${lead * 0.5}em` } : null, ink(lead ? [{ ...x.value[0], t: x.value[0].t.slice(lead) }, ...x.value.slice(1)] : x.value)); }
  return h("div.sw-fact", h("dt.sw-label", x.label), h("dd.sw-value",
    x.bar === undefined ? null : h("span.sw-bar.sw-bar-wide", { role: "progressbar", "aria-valuenow": x.bar, "aria-valuemin": 0, "aria-valuemax": 100 }, h("span.sw-bar-fill", { style: `width:${x.bar}%` })),
    x.lines.map((l) => h("span.sw-vline", ink(l)))));
}
export function actionsEl(snap, { onPress, onMenu }) {
  const buttons = (snap.actions || []).map((b, n) => h(`button.btn${n === 0 && !QUIET.includes(b.id) ? ".btn-primary" : ""}.sw-act`, { type: "button", onclick: () => onPress(b.id) }, cap(b.label)));
  // Into its agent: the open key, as the hint line has it where it says so (enter only folds and sends now).
  if (snap.canOpen && !(snap.actions || []).some((b) => b.id === "open")) buttons.push(h("button.btn.sw-act", { type: "button", onclick: () => onPress("open") }, "Open this task's agent"));
  buttons.push(h("button.btn.btn-ghost.sw-act", { type: "button", "aria-haspopup": "dialog", onclick: onMenu }, "More…"));
  return h("div.sw-actions", buttons);
}
export function panelEl(snap, handlers) {
  const p = snap.panel;
  if (!p) return null;
  return h("article.sw-panel", { "aria-label": p.automation ? "Automation" : "Task" },
    h("header.sw-panel-head",
      // One back arrow at a time: on a phone this one stands for the Board's own while a task is open (board.css); beside
      // the list on a wide screen, where the list never left, it is a close mark instead.
      h("button.icon-button.sw-panel-back", { type: "button", "aria-label": "Back to the list", title: "Back to the list", onclick: handlers.onClose }, h("span.sw-narrow", icon("back")), h("span.sw-wide", icon("close"))),
      h("h3.sw-panel-title", p.tag ? h("span.sw-id", p.tag + " ") : null, p.title || "(no title)")),
    actionsEl(snap, handlers),
    // A coordinator keeps what it started with: after its settings changed, a new one is what runs on them.
    p.stale ? h("div.sw-acct-warn.sw-stale", icon("alert"), h("span", `It runs on its old settings. A new one runs on ${p.stale}.`), h("button.btn.sw-act", { type: "button", onclick: () => handlers.onNew?.(p.to || "") }, "Start a new one")) : null,
    h("div.sw-parts", p.parts.map((part) => h("section.sw-part",
      h(`h4.sw-part-head${part.color ? `.sw-c-${part.color}` : ""}`, part.head),
      h("dl.sw-facts", part.items.map((x) => item(x, handlers.onGo)))))));
}

// ---- All settings ------------------------------------------------------------------------------------------------------
// Folded until asked for; it stays as you left it while the page is kept fresh.
let unfolded = false, shown = 0;
// One setting's control: one of a few words is picked, anything else is typed (saved when you press enter or leave the box,
// and only when it says something new).
function control(x, canSet, onSet) {
  return x.words ? h("select.sw-set-pick", { "aria-label": x.label, disabled: !canSet, onchange: (e) => onSet(x.name, e.target.value) },
    x.words.map((w) => h("option", { value: w.value, selected: w.now }, w.say)))
    : h("input.sw-set-box", { type: "text", value: x.value, "aria-label": x.label, title: x.hint, placeholder: x.hint, disabled: !canSet, autocomplete: "off", spellcheck: false,
      onchange: (e) => { const text = e.target.value.trim(); if (text && text !== x.value) onSet(x.name, text); else e.target.value = x.value; } });
}
const putBack = (x, onSet) => h("button.sw-set-back", { type: "button", title: "Back to what it would be without its own answer", onclick: () => onSet(x.name, null) }, icon("undo"), "Put back");
function setRow(x, whose, canSet, onSet) {
  return h(`div.sw-set${x.own ? ".is-own" : ""}`,
    h("div.sw-set-top", h("span.sw-set-label", x.label), control(x, canSet, onSet)),
    x.means ? h("p.sw-set-means", x.words ? `On: ${x.means}.` : `Means: ${x.means}.`) : null, // a typed value has no "on": what it is
    h("p.sw-set-from", [x.own ? `* ${x.whose || whose} only` : "", x.from].filter(Boolean).join(" · "), h("span.sw-set-name", `${x.own || x.from ? " · " : ""}${x.name}`),
      x.own && canSet ? putBack(x, onSet) : null));
}
// Which Claude account runs each role: stood out at the top, where a phone does not scroll past eighty rows to find it.
const ROLES = { worker: ["Agent", "user"], reviewer: ["Reviewer", "eye"], coordinator: ["Coordinator", "board"] };
const KINDS = { default: ["key", "The account herdr itself runs under"], login: ["user", "A Claude login"], gateway: ["cloud", "Gateway: own base URL and model"], missing: ["close", "No such account folder: its agent will not start"] };
// A changed one has its label in bold; the heading says for whom.
function accountsEl(rows, models, whose, canSet, onSet, onApply) {
  // Its model: one of the account's, or typed where the account names none. Its own is put back like the account's.
  const model = (acct) => {
    const m = models.find((y) => y.name.split(".")[0] === acct.name.split(".")[0]);
    return m ? h(`div.sw-acct-model${m.own ? ".is-own" : ""}`, h("span.sw-acct-label", icon("cpu"), "Model"), control(m, canSet, onSet), m.own && canSet ? putBack(m, onSet) : null) : null;
  };
  const row = (x) => {
    const [label, ic] = ROLES[x.name.split(".")[0]] || [x.label, "user"];
    const now = x.words?.find((w) => w.now)?.value ?? x.value, [kic, say] = KINDS[x.tags?.[now]] || KINDS.login;
    return h(`div.sw-acct${x.own ? ".is-own" : ""}`,
      h("div.sw-acct-top", h("span.sw-acct-label", icon(ic), label), control(x, canSet, onSet)),
      h("p.sw-set-from.sw-acct-tag", h("span.sw-acct-say", icon(kic), h("span", say)), x.own && canSet ? putBack(x, onSet) : null),
      x.program && x.program !== "claude" ? h("p.sw-set-from.sw-acct-warn", icon("alert"), h("span", `Runs on ${x.program}: an account is for Claude Code only.`),
        canSet ? h("button.sw-set-back", { type: "button", onclick: () => onSet(`${x.name.split(".")[0]}.agent`, "claude") }, "Use Claude") : null) : model(x));
  };
  return h("section.sw-accounts", h("h4.sw-accounts-head", icon("user"), "Accounts", h("span.sw-settings-note", `which Claude account runs each role, for ${whose}`), onApply && canSet ? h("button.sw-set-back", { type: "button", onclick: onApply }, icon("plus"), "Several repos") : null), rows.map(row));
}
export function settingsEl(snap, { onSet, onApply }) {
  const s = snap.settings;
  if (!s || !s.rows.length) return null;
  if (s.show !== shown) { if (s.show > shown && shown >= 0) unfolded = true; shown = s.show; }
  const set = (name, value) => onSet(s.id, name, value);
  const accounts = s.rows.filter((x) => x.name.endsWith(".account")), models = s.rows.filter((x) => x.name.endsWith(".model"));
  const rest = s.rows.filter((x) => !x.name.endsWith(".account") && !x.name.endsWith(".model"));
  const el = h("details.sw-settings", { open: unfolded, ontoggle: (e) => { unfolded = e.target.open; } },
    h("summary.sw-settings-head", "All settings", h("span.sw-settings-note", `${s.rows.filter((x) => x.own).length} of ${s.whose.endsWith("s") ? s.whose + "'" : s.whose + "'s"} own`)),
    h("p.sw-set-means", s.canSet ? `Every setting. A change here is for ${s.whose} only, and marked *.` : "Read-only here."),
    rest.map((x) => setRow(x, s.whose, s.canSet, set)));
  return h("div.sw-settings-wrap", accounts.length ? accountsEl(accounts, models, s.whose, s.canSet, set, onApply) : null, el);
}
