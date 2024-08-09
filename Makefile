build-toolchain:
	GITHUB_ACTIONS=false CARGO_TARGET_RISCV32IM_JOLT_ZKVM_ELF_RUSTFLAGS="-Cpasses=lower-atomic" ./x build
	GITHUB_ACTIONS=false CARGO_TARGET_RISCV32IM_JOLT_ZKVM_ELF_RUSTFLAGS="-Cpasses=lower-atomic" ./x build --stage 2

install-toolchain:
	rustup toolchain link riscv32im-jolt-zkvm-elf build/host/stage2

build-install-toolchain:
	make build-toolchain
	make install-toolchain

archive:
	tar -czvf toolchain.tar.gz build/host/stage2
