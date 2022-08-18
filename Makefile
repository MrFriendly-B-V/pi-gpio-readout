all: upload
.PHONY: build upload

RUST_SRC := $(shell find . -type f -name \*.rs)

build: target/aarch64-unknown-linux-gnu/release/pi-gpio-readout
target/aarch64-unknown-linux-gnu/release/pi-gpio-readout: $(RUST_SRC) Cargo.toml
	cargo build --release --target aarch64-unknown-linux-gnu

upload: build
	rsync -e 'ssh -p 25565' -ah --progress target/aarch64-unknown-linux-gnu/release/pi-gpio-readout tobias@83.87.88.123:~