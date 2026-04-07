# tests/tui/tapes/01_connections.tape
#
# Demos the connections list panel: add a placeholder, rename it,
# then delete it through the confirm popup.

Output tests/tui/tapes/output/01_connections.gif

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
Type "a"
Sleep 800ms
Type "a"
Sleep 800ms

Type "k"
Sleep 400ms
Type "r"
Sleep 400ms
Backspace 14
Type "prod postgres"
Sleep 400ms
Enter
Sleep 800ms

Type "j"
Sleep 400ms
Type "d"
Sleep 600ms
Type "y"
Sleep 800ms

Type "q"
Sleep 500ms
