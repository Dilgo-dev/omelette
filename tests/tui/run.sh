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

# ────────────────────────────────────────────────────────────────────
# 01 / connections list: add, rename, delete
# ────────────────────────────────────────────────────────────────────
test_connections_crud() {
  local s
  s=$(om_start connections) || return 1
  assert_contains "$s" "connections" || return 1
  assert_contains "$s" "no connection yet" || return 1

  om_send "$s" a
  assert_contains "$s" "New SQLite 1" || return 1
  assert_contains "$s" "SQL" || return 1

  om_send "$s" r
  for _ in $(seq 1 12); do tmux send-keys -t "$s" BSpace; done
  sleep 0.2
  om_type "$s" "local pgsql"
  om_send "$s" Enter
  assert_contains "$s" "local pgsql" || return 1
  assert_contains "$s" "renamed" || return 1

  om_send "$s" d
  assert_contains "$s" "confirm delete" || return 1
  om_send "$s" y
  assert_contains "$s" "no connection yet" || return 1
  assert_contains "$s" "deleted" || return 1
  om_stop "$s"
}
run_test "connections / add rename delete round trip" test_connections_crud

# ────────────────────────────────────────────────────────────────────
# 02 / schema browser: focus toggle, empty state, refresh
# ────────────────────────────────────────────────────────────────────
test_schema_panel() {
  local s
  s=$(om_start schema) || return 1
  assert_contains "$s" "schema" || return 1

  om_send "$s" a
  assert_contains "$s" "New SQLite 1" || return 1

  om_send "$s" Tab
  assert_contains "$s" "no tables" || return 1
  assert_contains "$s" "Tab: focus" || return 1
  assert_contains "$s" "R: refresh" || return 1

  om_send "$s" R
  assert_contains "$s" "loaded 0 table" || return 1

  om_send "$s" Tab
  om_send "$s" Tab
  assert_contains "$s" "a: add" || return 1
  om_stop "$s"
}
run_test "schema / panel toggles focus and refreshes" test_schema_panel

# ────────────────────────────────────────────────────────────────────
# 03 / preview: seeded SQLite renders rows in the preview panel
# ────────────────────────────────────────────────────────────────────
test_preview_seeded() {
  local s
  s=$(om_start_seeded preview) || return 1
  assert_contains "$s" "seeded preview" || return 1

  om_send "$s" Tab
  assert_contains "$s" "pets" || return 1

  om_send "$s" Tab
  assert_contains "$s" "preview" || return 1
  assert_contains "$s" "name" || return 1
  assert_contains "$s" "age" || return 1
  assert_contains "$s" "milo" || return 1
  assert_contains "$s" "luna" || return 1
  assert_contains "$s" "pepper" || return 1
  assert_contains "$s" "hjkl: scroll" || return 1
  om_stop "$s"
}
run_test "preview / seeded sqlite shows rows" test_preview_seeded

summary
