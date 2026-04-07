# tests/tui/tapes/02_schema.tape
#
# Demos the schema browser: add a SQLite connection, Tab into the
# schema panel, refresh, Tab back to connections.

Output tests/tui/tapes/output/02_schema.gif

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
Sleep 1500ms

Type "a"
Sleep 800ms
Tab
Sleep 800ms
Type "R"
Sleep 800ms
Tab
Sleep 600ms

Type "q"
Sleep 500ms
