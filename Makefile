RUST_DIR         = /rust
RUSTUP_HOME    = ${RUST_DIR}/rustup
CARGO_HOME     = ${RUST_DIR}/cargo
CARGO          = PATH=${CARGO_HOME}/bin:${GNU_TOOLS}/bin/:${PATH} CARGO_HOME=${CARGO_HOME} RUSTUP_HOME=${RUSTUP_HOME} cargo

#a Help
.PHONY: help
help.rust:
	@echo "RISC-V rust builds"

