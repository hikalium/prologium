default:
	cargo build

commit:
	cargo fmt
	cargo clippy
	cargo test
	git commit
