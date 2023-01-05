.PHONY: codecov

check:
	cargo +nightly fmt
	cargo clippy
	cargo test run

codecov:
	grcov . -s . --binary-path ./target/debug/ -t lcov --branch --ignore-not-existing -o ./target/debug/coverage/
