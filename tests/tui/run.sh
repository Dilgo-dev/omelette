#!/usr/bin/env bash
# End-to-end TUI tests for omelette, driven through tmux.
#
# Each test spawns omelette in a detached tmux session, sends a sequence
# of keys, captures the pane as plain text, and asserts the presence (or
# absence) of expected strings. Tests run with an isolated $HOME so they
# never touch the user's real connections.
#
# Run with:
#   make tui-test
# or directly:
#   nix-shell -p tmux --run ./tests/tui/run.sh

cd "$(dirname "$0")/../.."

# shellcheck source=lib.sh
. tests/tui/lib.sh

# ────────────────────────────────────────────────────────────────────
# 00 / smoke: app boots, shows title, quits cleanly with q
# ────────────────────────────────────────────────────────────────────
test_smoke_boot() {
  local s
  s=$(om_start smoke) || return 1
  assert_contains "$s" "omel" || return 1
  assert_contains "$s" "ette" || return 1
  assert_contains "$s" "crack open your databases" || return 1
  assert_contains "$s" "q: quit" || return 1
  om_stop "$s"
}
run_test "smoke / boots and shows title" test_smoke_boot

summary
