# tests/tui/tapes/00_smoke.tape
#
# Reference VHS tape: boots omelette, shows the splash, and quits.
# Canonical demo used by the autorunner Discord upload. Copy this file
# as a starting point when adding a new tape for a new feature.

Output tests/tui/tapes/output/00_smoke.gif

Set Shell "bash"
Set FontSize 14
Set Width 1280
Set Height 720
Set Theme "Catppuccin Mocha"
Set TypingSpeed 60ms

Hide
Type "export HOME=$(mktemp -d) && mkdir -p $HOME/.config/omelette"
Enter
Show

Type "./target/release/omelette"
Enter
Sleep 2000ms

Type "q"
Sleep 500ms
