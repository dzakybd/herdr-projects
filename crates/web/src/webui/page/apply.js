// The sheet that sets one account for several repos at once: which account, for which roles, in which repos. The server
// (webui/apply.rs) sets each repo's own entry; what it refuses (no such repo) stays in the sheet.
import { h, icon } from "./dom.js";

const ROLES = [["worker", "Agent"], ["reviewer", "Reviewer"], ["coordinator", "Coordinator"]];
export function applyEl({ accounts, repos, onApply, onClose }) {
  const account = h("select.sw-set-pick", { "aria-label": "Account" }, accounts.map((a) => h("option", { value: a }, a)));
  const box = (value, label, on = true) => { const c = h("input", { type: "checkbox", value, checked: on }); return [c, h("label.sw-form-row.sw-form-check", c, h("span", label))]; };
  const roles = ROLES.map(([v, l]) => box(v, l));
  const claude = box("claude", "Put a role on Claude Code where it runs another program (an account is for Claude Code only)", true);
  const list = repos.map((r) => box(r.name, r.name, false));
  const all = h("button.btn.sw-act", { type: "button", onclick: () => { const on = list.some(([c]) => !c.checked); list.forEach(([c]) => (c.checked = on)); } }, "All / none");
  const err = h("p.sw-set-from.sw-form-err", { role: "alert" });
  const apply = h("button.btn.btn-primary.sw-act", { type: "button", onclick: async () => {
    apply.disabled = true;
    err.textContent = (await onApply({ account: account.value, roles: roles.filter(([c]) => c.checked).map(([c]) => c.value), repos: list.filter(([c]) => c.checked).map(([c]) => c.value), claude: claude[0].checked })) || "";
    apply.disabled = false;
  } }, "Apply");
  return h("div.sw-scrim", { onclick: (e) => { if (e.target === e.currentTarget) onClose(); } }, h("div.sw-sheet", { role: "dialog", "aria-modal": "true", "aria-label": "Set an account for several repos" },
    h("header.sw-menu-head", h("span.sw-menu-title", "An account for several repos"), h("button.icon-button", { type: "button", "aria-label": "Close", onclick: onClose }, icon("close"))),
    h("p.sw-set-means", "Each repo gets its own entry. A repo you set by hand later keeps what you set."),
    h("label.sw-form-row", h("span.sw-acct-label", "Account"), account),
    h("div.sw-form-row", h("span.sw-acct-label", "Roles"), roles.map(([, l]) => l)), claude[1],
    h("div.sw-form-row", h("span.sw-acct-label", "Repos", all), list.map(([, l]) => l)), err,
    h("div.sw-ask-keys", h("button.btn.sw-act", { type: "button", onclick: onClose }, "Cancel"), apply)));
}
