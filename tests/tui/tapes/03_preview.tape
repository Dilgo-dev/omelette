# tests/tui/tapes/03_preview.tape
#
# Demos the table preview panel against a seeded SQLite file with a
# `pets` table. Tabs through Connections -> Schema -> Preview, then
# scrolls horizontally and vertically.

Output tests/tui/tapes/output/03_preview.gif

Set Shell "bash"
Set FontSize 14
Set Width 1280
Set Height 720
Set Theme "Catppuccin Mocha"
Set TypingSpeed 60ms

Hide
Type "export HOME=$(mktemp -d) && mkdir -p $HOME/.config/omelette"
Enter
Type 'sqlite3 $HOME/seed.db "CREATE TABLE pets (id INTEGER PRIMARY KEY, name TEXT, age INTEGER); INSERT INTO pets (name, age) VALUES (\"milo\", 4), (\"luna\", 2), (\"pepper\", 7);"'
Enter
Type 'cat > $HOME/.config/omelette/connections.json <<JSON'
Enter
Type '{"connections":[{"id":"seed","label":"seeded pets","engine":"Sqlite","dsn":"sqlite://'$HOME'/seed.db"}]}'
Enter
Type "JSON"
Enter
Show

Type "./target/release/omelette"
Enter
Sleep 1500ms

Tab
Sleep 800ms
Tab
Sleep 1200ms
Type "j"
Sleep 400ms
Type "j"
Sleep 400ms
Type "k"
Sleep 400ms
Type "l"
Sleep 400ms
Type "h"
Sleep 600ms

Type "q"
Sleep 500ms
