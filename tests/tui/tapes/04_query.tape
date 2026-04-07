# tests/tui/tapes/04_query.tape
#
# Demos the query editor: enter editing mode, type a SQL query, run
# it with F5 against the seeded SQLite db, then leave editing mode.

Output tests/tui/tapes/output/04_query.gif

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

Type "e"
Sleep 600ms
Type "SELECT name, age FROM pets WHERE age > 3"
Sleep 600ms
F5
Sleep 1500ms
Escape
Sleep 600ms

Type "q"
Sleep 500ms
