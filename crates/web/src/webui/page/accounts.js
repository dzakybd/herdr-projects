// The Accounts sheet (opened from the Accounts card's "Manage accounts"): the Claude Code accounts there are, a new one (a
// Claude login or a gateway), and a gateway's base URL, the name of its key variable, its models and its context size.
// The server checks every value and says why not (webui/accounts.rs); a key is never typed here, only the name of the
// variable the shell exports it as. docs/accounts.md.
/* global document, navigator */
import { h, icon, post } from "./dom.js";

const KINDS = { gateway: ["cloud", "Gateway"], login: ["user", "Claude login"] };
const loginCommand = (name) => `CLAUDE_CONFIG_DIR=~/.claude-accounts/${name || "<name>"} claude`;
const words = (text) => text.split(/\s+/).filter(Boolean);

// Copied where the browser lets a page write the clipboard, else through a box that is selected and copied.
async function copy(text, button) {
  try { await navigator.clipboard.writeText(text); } catch {
    const box = h("textarea", { value: text }); document.body.append(box); box.select();
    try { document.execCommand("copy"); } catch {}
    box.remove();
  }
  button.textContent = "Copied"; setTimeout(() => { button.textContent = "Copy"; }, 2000);
}
// What to run in a terminal to log in to a login account, once. `name()` is read when it is copied: the form may still be typed into.
function loginEl(name) {
  const cmd = h("code.sw-acc-cmd", loginCommand(name()));
  const btn = h("button.btn.sw-act", { type: "button", "aria-label": "Copy the login command", onclick: () => copy(loginCommand(name()), btn) }, "Copy");
  return { cmd, el: h("div.sw-acc-login", h("p.sw-set-means", "Log in once, in a terminal on this machine, with this command:"), h("div.sw-acc-cmdrow", cmd, btn)) };
}
const row = (label, el, hint) => h("label.sw-form-row", h("span.sw-acct-label", label), el, hint ? h("span.sw-set-from", hint) : null);

// `onSaved(out)`: the board's own answer to a save, to draw from (the Accounts card's lists offer the account and its models at once).
export function accountsEl({ onClose, onSaved }) {
  const body = h("div.sw-acc-body");
  let accounts = null, edit = null, said = "", failed = "";
  const draw = () => { body.replaceChildren(edit ? formEl() : listEl()); body.querySelector("input[aria-label=Name], button.sw-acc-item")?.focus({ preventScroll: true }); };
  const load = async () => { const out = await post("accounts"); accounts = out.ok ? out.accounts : []; failed = out.ok ? "" : out.msg || "The accounts could not be read."; draw(); };

  function listEl() {
    return h("div",
      h("p.sw-set-means", "An account is a folder of ~/.claude-accounts. Each role picks one in the Accounts card: a Claude login, or a gateway with its own base URL and models."),
      said ? h("p.sw-set-from.sw-acc-said", { role: "status" }, said) : null, failed ? h("p.sw-set-from.sw-form-err", { role: "alert" }, failed) : null,
      accounts === null ? h("p.sw-set-from", "Reading the accounts…") : accounts.length ? h("div.sw-acc-list", accounts.map((a) => { const [ic, say] = KINDS[a.kind] || KINDS.login;
        return h("button.sw-acc-item", { type: "button", onclick: () => { said = ""; edit = { account: a }; draw(); } }, icon(ic), h("span.sw-acc-name", a.name), h("span.sw-set-from", say), icon("right")); })) : h("p.sw-set-from", "No account yet."),
      h("div.sw-ask-keys", h("button.btn.sw-act", { type: "button", onclick: onClose }, "Close"), h("button.btn.btn-primary.sw-act", { type: "button", onclick: () => { said = ""; edit = { account: null }; draw(); } }, icon("plus"), "New account")));
  }
  function formEl() {
    if (edit.done) return h("div", h("p.sw-set-from.sw-acc-said", { role: "status" }, said), loginEl(() => edit.account?.name || "").el, h("div.sw-ask-keys", h("button.btn.btn-primary.sw-act", { type: "button", onclick: () => { edit = null; draw(); } }, "Done")));
    const a = edit.account, fresh = !a, g = a?.gateway || {};
    const name = h("input.sw-set-box", { type: "text", value: a?.name || "", placeholder: "work", "aria-label": "Name", autocomplete: "off", autocapitalize: "none", spellcheck: false, maxLength: 32 });
    const type = h("select.sw-set-pick", { "aria-label": "Type" }, [["login", "Claude login"], ["gateway", "Gateway"]].map(([v, t]) => h("option", { value: v, selected: (a?.kind || "login") === v }, t)));
    const url = h("input.sw-set-box", { type: "url", value: g.baseUrl || "", placeholder: "https://gateway.example.com", "aria-label": "Base URL", autocomplete: "off", spellcheck: false });
    const keyVar = h("input.sw-set-box", { type: "text", value: g.keyVar || "", placeholder: "MY_GATEWAY_KEY", "aria-label": "Key variable name", autocomplete: "off", autocapitalize: "none", spellcheck: false });
    const model = h("input.sw-set-box", { type: "text", value: g.model || "", list: "sw-acc-models", "aria-label": "Default model", autocomplete: "off", autocapitalize: "none", spellcheck: false });
    const models = h("textarea.sw-set-box.sw-form-prompt", { rows: 3, value: (g.models || []).join("\n"), placeholder: "one model per line", "aria-label": "Models", autocapitalize: "none", spellcheck: false });
    const chips = h("div.sw-acc-chips", { "aria-hidden": "true" }), options = h("datalist", { id: "sw-acc-models" });
    const context = h("input.sw-set-box", { type: "text", inputMode: "numeric", value: g.context ?? "", placeholder: "350000", "aria-label": "Context tokens", autocomplete: "off" });
    const confirm = h("input", { type: "checkbox" });
    const err = h("p.sw-set-from.sw-form-err", { role: "alert" });
    const login = loginEl(() => (fresh ? name.value.trim() : a.name));
    const gateway = h("div.sw-acc-gw", row("Base URL", url), row("Key variable name", keyVar, "The NAME of an environment variable, never the key. Export it in your shell (for example `export MY_GATEWAY_KEY=…` in ~/.zshrc): the agent's terminal reads it from there, and the board stores no key."),
      g.keyLiteral ? h("p.sw-set-from.sw-acct-warn", icon("alert"), h("span", "This file holds a key written out. It is not shown; saving replaces it with the variable named above.")) : null,
      row("Default model", model), row("Models the board offers", models, "Separated by spaces or lines."), chips, options, row("Context tokens (optional)", context, "Written as CLAUDE_CODE_MAX_CONTEXT_TOKENS. Every other line of the file is kept as it is."));
    const toLogin = h("label.sw-form-row.sw-form-check", { hidden: true }, confirm, h("span", "Delete its account.env (base URL, key variable, models and any other line in it)"));
    const sync = () => {
      const list = words(models.value); chips.replaceChildren(...list.map((m) => h("span.sw-acc-chip", m))); options.replaceChildren(...list.map((m) => h("option", { value: m })));
      const gw = type.value === "gateway"; gateway.hidden = !gw; login.el.hidden = gw; login.cmd.textContent = loginCommand(fresh ? name.value.trim() : a.name);
      toLogin.hidden = gw || a?.kind !== "gateway";
    };
    for (const el of [name, type, models]) el.addEventListener("input", sync);
    sync();
    const save = h("button.btn.btn-primary.sw-act", { type: "button", onclick: async () => {
      save.disabled = true; err.textContent = "";
      const out = await post("account-save", { new: fresh, name: fresh ? name.value.trim() : a.name, type: type.value, baseUrl: url.value, keyVar: keyVar.value, model: model.value, models: models.value, context: context.value, confirm: confirm.checked });
      save.disabled = false;
      if (!out.ok) { err.textContent = out.msg || "That did not work."; return; }
      accounts = out.accounts; said = out.said || "Saved."; onSaved(out);
      // A new login is not logged in yet: its command stays up, to copy.
      if (fresh && type.value === "login") edit = { account: accounts.find((x) => x.name === name.value.trim()), done: true }; else edit = null;
      draw();
    } }, fresh ? "Make the account" : "Save");
    return h("div",
      fresh ? row("Name", name, "Lowercase letters, digits, _ and -.") : h("p.sw-acc-title", icon((KINDS[a.kind] || KINDS.login)[0]), a.name),
      row("Type", type), login.el, gateway, toLogin, err,
      h("div.sw-ask-keys", h("button.btn.sw-act", { type: "button", onclick: () => { edit = null; draw(); } }, "Back"), save));
  }
  load();
  return h("div.sw-scrim", { onclick: (e) => { if (e.target === e.currentTarget) onClose(); } }, h("div.sw-sheet", { role: "dialog", "aria-modal": "true", "aria-label": "Accounts" },
    h("header.sw-menu-head", h("span.sw-menu-title", "Accounts"), h("button.icon-button", { type: "button", "aria-label": "Close", onclick: onClose }, icon("close"))), body));
}
