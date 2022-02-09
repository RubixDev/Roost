THEME_FILE = src/res/one-dark.tmTheme
GRAMMAR_FILE = src/res/roost.sublime-syntax

release: pull test_release
	cargo build --release

install: release
	sudo cp target/release/roost /usr/local/bin/roost

pull:
	mkdir -p src/res/
	[ -f $(THEME_FILE) ] || touch $(THEME_FILE)
	[ -f $(GRAMMAR_FILE) ] || touch $(GRAMMAR_FILE)
	cargo test --release --package roost --lib -- tests::fetch::fetch --exact --nocapture

test_release:
	cargo test --release -- tests::all

test:
	cargo test -- tests::all
