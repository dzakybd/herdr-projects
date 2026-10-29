#!/bin/sh
# Builds the release program this checkout runs (bin/run.sh uses target/release when no source is newer) and starts the
# board's helpers (keeper, web front door) again on it, without restarting herdr. Open terminal boards keep the old build
# until they are opened again.
set -e
here=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$here"
cargo build --release
# Also the downloaded program's helpers (bin/herdr-projects-<os>-<arch>): started while a source file was newer than the build, they would stay and take the wrapper over.
pkill -f "$here/(target/release/herdr-projects|bin/herdr-projects-[a-z]+-[a-z0-9_]+) (keeper|keeper-work|serve|door)" 2>/dev/null || true
sleep 3 # the hook starts nothing while a helper is still ending
# The hook starts nothing while it still sees a helper's lock fresh, so it is asked again until the front door is up.
for _ in 1 2 3 4; do
  sh bin/run.sh hook
  sleep 8
  out=$("$HOME/.herdr-projects/data/bin/herdr-projects" web-ui) && case "$out" in *"Front door: up"*) break ;; esac
done
printf '%s\n' "$out" | sed -n 1,2p
