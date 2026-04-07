VERSION ?= dev

.PHONY: build install clean fmt lint check tui-test tui-record tui-record-one

build:
	cargo build --release

install: build
	cp target/release/omelette ~/.local/bin/omelette

clean:
	cargo clean

fmt:
	cargo fmt

lint:
	cargo clippy -- -D warnings

check: fmt lint build

tui-test: build
	nix-shell -p tmux --run ./tests/tui/run.sh

tui-record: build
	mkdir -p tests/tui/tapes/output
	nix-shell -p vhs ttyd ffmpeg --run 'for tape in tests/tui/tapes/*.tape; do echo "▶ $$tape"; vhs "$$tape" || exit 1; done'

tui-record-one: build
	mkdir -p tests/tui/tapes/output
	nix-shell -p vhs ttyd ffmpeg --run 'vhs tests/tui/tapes/$(TAPE).tape'
