#!/bin/sh
# Builds the release program this checkout runs (bin/run.sh uses target/release when no source is newer) and starts the
# board's helpers (keeper, web front door) again on it, without restarting herdr. Open terminal boards keep the old build
# until they are opened again.
set -e
here=$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)
cd "$here"
cargo build --release
pkill -f "$here/target/release/herdr-projects (keeper|keeper-work|serve|door)" 2>/dev/null || true
sleep 3 # the hook starts nothing while a helper is still ending
sh bin/run.sh hook
sleep 6
"$HOME/.herdr-projects/data/bin/herdr-projects" web-ui | sed -n 1,2p
