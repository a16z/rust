build-toolchain:
	CARGO_TARGET_RISCV32I_JOLT_ZKVM_ELF_RUSTFLAGS="-Cpasses=loweratomic" ./x build
	CARGO_TARGET_RISCV32I_JOLT_ZKVM_ELF_RUSTFLAGS="-Cpasses=loweratomic" ./x build --stage 2

install-toolchain:
	rustup toolchain link riscv32i-jolt-zkvm-elf build/host/stage2

build-install-toolchain:
	make build-toolchain
	make install-toolchain

archive:
	tar -czvf toolchain.tar.gz build/host/stage2
