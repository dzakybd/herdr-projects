# Claude Code accounts

**Give each role its own Claude Code account, and its own model, without a second board.** `worker.account`,
`reviewer.account` and `coordinator.account` name the account a role's Claude Code runs under; `worker.model`,
`reviewer.model` and `coordinator.model` name the model. Both are settings like the others ([settings](settings.md)):
for one task, for a repo, or for everything of yours.

```
herdr-projects set worker.account work --repo my-app          every task of my-app runs under ~/.claude-accounts/work
herdr-projects set reviewer.account personal                  every reviewer, unless a repo or a task says otherwise
herdr-projects set worker.model opus --task 12                one task's agent on opus
herdr-projects unset worker.account --repo my-app             back to what it was
```

## An account

An account is a Claude Code config folder, the one `CLAUDE_CONFIG_DIR` names. The value is:

- a **name**: a folder of `~/.claude-accounts/` (`work` is `~/.claude-accounts/work`), or
- a **path** (`~/accounts/work`, or an absolute one), or
- `none` (the default): the account herdr itself runs under.

Before the agent's program starts, its pane's shell is given `CLAUDE_CONFIG_DIR` and `CLAUDE_ACCOUNT_LABEL` (the
folder's name). An account whose folder is not there is refused when the agent starts, with the folder it looked for
(`Could not start: the account "work" has no folder …`), because a typo would otherwise start a new, logged-out
account. Only `claude` has accounts of this kind; for another program the setting does nothing.

Log in to an account once, as you would: `CLAUDE_CONFIG_DIR=~/.claude-accounts/work claude`.

## A gateway account

An account folder may hold an **`account.env`**. The pane's own shell sources it after giving the folder, so a provider
account (a base URL, a model, a key) needs nothing else:

```sh
# ~/.claude-accounts/gateway/account.env
ANTHROPIC_BASE_URL="https://gateway.example.com"
ANTHROPIC_AUTH_TOKEN="$GATEWAY_API_KEY"     # read from the shell's own variable: the board never holds a key
ANTHROPIC_MODEL="model-one"
ACCOUNT_MODELS="model-one model-two"        # the models the web board offers for this account
```

The shell is the pane's, an interactive one, so a key exported in `~/.zshrc` is there. A key written in the file would
work too, but then the file is a secret.

## A model

`<role>.model` is any name the account takes: `opus` or `sonnet` for a login, or a gateway's own (`model-two`). It is
passed to Claude Code as `--model`, and Claude Code, not herdr, checks it. It wins over a model written after the
slash in `<role>.agent` ([models and effort](models.md)); `none` leaves that, or the account's own `ANTHROPIC_MODEL`.

## In the web board

Open a heading (the machine's row for everything of yours, a project's row for one repo) or a task, and the
**Accounts** card stands above **All settings**: a role each, its account as a list of the folders there are, what the
account is (a login, a gateway, or no folder), and its model as a list of what the account offers (Claude's short
names for a login, `ACCOUNT_MODELS` for a gateway, typed where there is no list). A change is saved at once, for that
heading or task only; **Put back** takes it away. It lays out for a phone: full-width lists and 44 px targets.

The terminal board's settings page lists the same settings, typed.

## Managing accounts in the web board

**Manage accounts**, in the Accounts card's heading, opens a sheet that makes and changes accounts, so none has to be
made by hand in a terminal. It is for a device that may drive the board; one that only watches does not see the button,
and the calls behind it (`accounts`, `account-save`) refuse it.

- **The list** shows every folder of `~/.claude-accounts/` and what it is, a Claude login or a gateway. Tap one to change it.
- **New account**: a name (a to z, digits, `_` and `-`, at most 32, starting with a letter or digit), and a type.
  The folder is made under `~/.claude-accounts/` only, readable by you alone; a name that is already there is refused,
  and a name with a slash or `..` in it is no name.
  - **Claude login**: the sheet then shows the command to run once in a terminal, with a Copy button:
    `CLAUDE_CONFIG_DIR=~/.claude-accounts/<name> claude`. The login happens there; the board never sees it.
  - **Gateway**: writes the account's `account.env` from the form.
- **A gateway's fields**: the base URL (`http://` or `https://`), the **name** of the variable that holds its key, the
  default model (`ANTHROPIC_MODEL`), the models the board offers (`ACCOUNT_MODELS`, one per line or separated by spaces)
  and, optionally, the context tokens (`CLAUDE_CODE_MAX_CONTEXT_TOKENS`). The key variable must be exported by the shell
  the agent's pane starts (for example `export MY_GATEWAY_KEY=…` in `~/.zshrc`): the sheet never takes a key, and the board
  stores none. A file that holds a key written out is told so, the key is not shown, and saving replaces it with the variable.
- **Saving rewrites only those lines** of `account.env` (the file is written beside itself and renamed over it, mode
  `0600`). Comments and every other line, such as `CLAUDE_CODE_AUTO_COMPACT_WINDOW`, stay as they are; an empty default
  model, model list or context size drops its line. A value that would be unsafe in a file the shell sources (a quote, a
  backtick, `$( )`, a newline, a space in a URL or model name) is refused, with the reason shown in the sheet.
- **Login and gateway, both ways**: a login becomes a gateway by giving it the fields (an `account.env` is made). A gateway
  becomes a login again by deleting its `account.env`, with every line in it, so the sheet asks for a tick first.
- Deleting a whole account folder is not offered here: remove it in a terminal.

The Accounts card's lists read the folders and `ACCOUNT_MODELS` at every look, so a saved account and its models are
offered at once, with no restart.

## Trust

Claude Code asks "do you trust this folder?" the first time it starts in one, and remembers the answer in the account's
own `.claude.json`. `trust.folders` says whether the board answers it for you:

- `board` (the default): the board marks as trusted the folders it makes or owns for a task or a role, and only those:
  the task's folder, a reviewer's or picker's folder, and a Coordinator's chat folder. It writes them into the
  `.claude.json` of the account the role runs under, and does it under any access, so a role on `ask` is no longer
  asked once per task or per account. Any other folder is never trusted by the board.
- `ask`: the board marks a folder as trusted only for a role on access `full` or `auto`, as it used to. A role on
  `ask` is asked for each folder.

This is only the folder dialog. What an agent may do without asking, `<role>.access`, is not changed: on `ask`, it
still asks before each action. Set it like any setting: `herdr-projects set trust.folders ask --repo my-app`.
