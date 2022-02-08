THEME_FILE = src/res/one-dark.tmTheme
GRAMMAR_FILE = src/res/roost.sublime-syntax

release: pull
	cargo build --release

install: release
	sudo cp target/release/roost /usr/local/bin/roost

pull:
	mkdir -p src/res/
	[ -f $(THEME_FILE) ] || touch $(THEME_FILE)
	[ -f $(GRAMMAR_FILE) ] || touch $(GRAMMAR_FILE)
	cargo test --package roost --lib -- tests::fetch --exact --nocapture
