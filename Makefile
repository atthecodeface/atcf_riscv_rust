RUST_DIR       = /rust
GNU_TOOLS      = /Users/gstark/Git/rfpc-sw/install/
GNU_TOOLS      = /riscv/tools/bin/riscv32-unknown-elf-
RISCV_CROSS_PREFIX = ${GNU_TOOLS}/riscv32-elf-
RUSTC_LINKER   = ${RISCV_CROSS_PREFIX}ld
RUSTUP_HOME    = ${RUST_DIR}/rustup
CARGO_HOME     = ${RUST_DIR}/cargo
RUST_ENV       = PATH=${CARGO_HOME}/bin:${GNU_TOOLS}/bin/:${PATH} CARGO_HOME=${CARGO_HOME} RUSTUP_HOME=${RUSTUP_HOME} GNU_TOOLS=${GNU_TOOLS} RUSTC_LINKER=${RUSTC_LINKER}
CARGO          = ${RUST_ENV} cargo
RUSTUP         = $(RUST_ENV) rustup

#a Top level
.PHONY: all
all: build

#a Help
.PHONY: help
help.rust:
	@echo "RISC-V rust builds"


#a Update rust
update_rust:
	${RUSTUP} target add riscv32imc-unknown-none-elf

#a Build
build:
	${CARGO} build --release

build_debug:
	${CARGO} build

clean:
	${CARGO} clean
