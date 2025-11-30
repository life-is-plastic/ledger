build-release:
	cargo build --release
	mv target/release/ledger ~/bin/l
