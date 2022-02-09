THEME_FILE = cli/src/res/one-dark.tmTheme
GRAMMAR_FILE = cli/src/res/roost.sublime-syntax

release: pull
	cargo test --release -- tests::all
	cargo build --release

install: pull
	cargo build --release
	sudo cp target/release/roost-cli /usr/local/bin/roost

pull:
	mkdir -p cli/src/res/
	[ -f $(THEME_FILE) ] || touch $(THEME_FILE)
	[ -f $(GRAMMAR_FILE) ] || touch $(GRAMMAR_FILE)
	cargo test --release -- tests::fetch::fetch

test:
	cargo test -- tests::all
