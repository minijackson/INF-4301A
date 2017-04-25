release: ./target/release/inf_4301a

debug: ./target/debug/inf_4301a

run: release
	./target/release/inf_4301a

./target/release/inf_4301a:
	cargo build --release

./target/debug/inf_4301a:
	cargo build

check:
	cargo test

cargo:
	./rust_installer.sh

.PHONY: release debug check cargo
