release: cargo ./target/release/inf_4301a

debug: cargo ./target/debug/inf_4301a

run: release
	./target/release/inf_4301a

./target/release/inf_4301a:
	cargo build --release

./target/debug/inf_4301a:
	cargo build

check: cargo
	cargo test

doc: cargo
	cargo doc

cargo:
	./rust_installer.sh

.PHONY: release debug check doc cargo
