# herdr-projects (a fork for per-project Claude accounts)

Read AGENTS.md first: it is the repo's own contract (file size limit, where things go, the check). This file is what is
specific to this fork.

## Project context

### Stack
- Language: Rust, one workspace (`crates/core`, `flow`, `board`, `web`, command in `src/`); the web board's page is plain
  ES modules and CSS in `crates/web/src/webui/page/`, embedded in the binary (a new file is also listed in `page.rs`).
- Runs as a herdr plugin (linked from this folder: `herdr plugin list`), served in the web UI Ovis.

### Commands
- Build for tests: `cargo build --profile quick`
- Test (one area): `cargo test --profile quick --test main -- <name>`
- Test (all) and the repo's rules: `npm run check` (25+ minutes: run it at the end, not while iterating)
- Reload what runs on this machine: `sh scripts/reload.sh` (release build, helpers restarted, herdr untouched)
- Regenerate made docs: `./target/quick/herdr-projects docs`

### Layout
- Source in `crates/*/src`; tests in `tests/` (modules of `tests/main.rs`, which is at its 300-line limit: add tests to an
  existing file, not a `mod` line); docs in `docs/` (never edit the made tables).
- Do not modify: `docs/settings.md` between the `made:` lines, `docs/where-things-live.md` (not in git).

### Conventions specific to this fork
- Accounts: `<role>.account` and `<role>.model` (docs/accounts.md). An account is a folder of `~/.claude-accounts/`; a
  gateway account has an `account.env` sourced by the pane's shell. Never store an API key in it: name the variable.
- Web calls that change something go in `CHANGES` in `crates/web/src/webui/door.rs` (else the front door answers 404).
- A new setting is one entry in `crates/core/src/store/settings_list.rs`; a role setting goes in `role()`.

### Forbidden
- `herdr-projects update` and `herdr plugin update herdr-projects`: they fetch upstream's release and replace this fork's
  behaviour. To take upstream changes: `git fetch upstream && git merge upstream/main`, then `sh scripts/reload.sh`.
- Leaving `target/release/herdr-projects` older than a source file: `bin/run.sh` then runs a downloaded upstream
  release instead of this fork. Run `sh scripts/reload.sh` after changing source.
- Restarting the herdr server for a reload: it closes every open pane.

## Project Learnings
- The web board's repo follows the page's pane (`follow` call), not the terminal board's last folder.
- Claude Code keeps folder trust in `<CLAUDE_CONFIG_DIR>/.claude.json`, so the board trusts there for a role on an account.
