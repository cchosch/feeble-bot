.DEFAULT_GOAL=dev

dev:
	RUSTFLAGS=-Awarnings cargo watch -x "run -- -m dev" -i .\*\out\*
