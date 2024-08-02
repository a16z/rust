ifdef VERBOSE
Q :=
BOOTSTRAP_ARGS := -v
else
Q := @
BOOTSTRAP_ARGS :=
endif

# Pass `-jN` to the bootstrap if it is specified.
ifdef MAKEFLAGS
  ifneq (,$(findstring -j, $(MAKEFLAGS)))
    BOOTSTRAP_ARGS += $(filter -j%, $(MAKEFLAGS))
  endif
endif

BOOTSTRAP := /Applications/Xcode.app/Contents/Developer/usr/bin/python3 /Users/mzhu/code/rust/src/bootstrap/bootstrap.py

all:
	$(Q)$(BOOTSTRAP) build --stage 2 $(BOOTSTRAP_ARGS)
	$(Q)$(BOOTSTRAP) doc --stage 2 $(BOOTSTRAP_ARGS)

help:
	$(Q)echo 'Welcome to bootstrap, the Rust build system!'
	$(Q)echo
	$(Q)echo This makefile is a thin veneer over the ./x.py script located
	$(Q)echo in this directory. To get the full power of the build system
	$(Q)echo you can run x.py directly.
	$(Q)echo
	$(Q)echo To learn more run \`./x.py --help\`

clean:
	$(Q)$(BOOTSTRAP) clean $(BOOTSTRAP_ARGS)

rustc-stage1:
	$(Q)$(BOOTSTRAP) build --stage 1 library/test $(BOOTSTRAP_ARGS)
rustc-stage2:
	$(Q)$(BOOTSTRAP) build --stage 2 library/test $(BOOTSTRAP_ARGS)

docs: doc
doc:
	$(Q)$(BOOTSTRAP) doc --stage 2 $(BOOTSTRAP_ARGS)
nomicon:
	$(Q)$(BOOTSTRAP) doc --stage 2 src/doc/nomicon $(BOOTSTRAP_ARGS)
book:
	$(Q)$(BOOTSTRAP) doc --stage 2 src/doc/book $(BOOTSTRAP_ARGS)
standalone-docs:
	$(Q)$(BOOTSTRAP) doc --stage 2 src/doc $(BOOTSTRAP_ARGS)
check:
	$(Q)$(BOOTSTRAP) test --stage 2 $(BOOTSTRAP_ARGS)
check-aux:
	$(Q)$(BOOTSTRAP) test --stage 2 \
		src/tools/cargo \
		src/tools/cargotest \
		src/etc/test-float-parse \
		$(BOOTSTRAP_ARGS)
	# Run standard library tests in Miri.
	$(Q)BOOTSTRAP_SKIP_TARGET_SANITY=1 \
		$(BOOTSTRAP) miri --stage 2 \
		library/core \
		library/alloc \
		--no-doc
	# Some doctests use file system operations to demonstrate dealing with `Result`.
	$(Q)MIRIFLAGS="-Zmiri-disable-isolation" \
		$(BOOTSTRAP) miri --stage 2 \
		library/core \
		library/alloc \
		--doc
	# In `std` we cannot test everything, so we skip some modules.
	$(Q)MIRIFLAGS="-Zmiri-disable-isolation" \
		$(BOOTSTRAP) miri --stage 2 library/std \
		--no-doc -- \
		--skip fs:: --skip net:: --skip process:: --skip sys::pal::
	$(Q)MIRIFLAGS="-Zmiri-disable-isolation" \
		$(BOOTSTRAP) miri --stage 2 library/std \
		--doc -- \
		--skip fs:: --skip net:: --skip process:: --skip sys::pal::
	# Also test some very target-specific modules on other targets
	# (making sure to cover an i686 target as well).
	$(Q)MIRIFLAGS="-Zmiri-disable-isolation" BOOTSTRAP_SKIP_TARGET_SANITY=1 \
		$(BOOTSTRAP) miri --stage 2 library/std \
		--target aarch64-apple-darwin,i686-pc-windows-msvc \
		--no-doc -- \
		time:: sync:: thread:: env::
dist:
	$(Q)$(BOOTSTRAP) dist $(BOOTSTRAP_ARGS)
distcheck:
	$(Q)$(BOOTSTRAP) dist $(BOOTSTRAP_ARGS)
	$(Q)$(BOOTSTRAP) test --stage 2 distcheck $(BOOTSTRAP_ARGS)
install:
	$(Q)$(BOOTSTRAP) install $(BOOTSTRAP_ARGS)
tidy:
	$(Q)$(BOOTSTRAP) test --stage 2 src/tools/tidy $(BOOTSTRAP_ARGS)
prepare:
	$(Q)$(BOOTSTRAP) build --stage 2 nonexistent/path/to/trigger/cargo/metadata

## MSVC native builders

# this intentionally doesn't use `$(BOOTSTRAP)` so we can test the shebang on Windows
ci-msvc-py:
	$(Q)/Users/mzhu/code/rust//x.py test --stage 2 tidy
ci-msvc-ps1:
	$(Q)/Users/mzhu/code/rust//x.ps1 test --stage 2 --skip tidy
ci-msvc: ci-msvc-py ci-msvc-ps1

## MingW native builders

# test both x and bootstrap entrypoints
ci-mingw-x:
	$(Q)/Users/mzhu/code/rust//x test --stage 2 tidy
ci-mingw-bootstrap:
	$(Q)$(BOOTSTRAP) test --stage 2 --skip tidy
ci-mingw: ci-mingw-x ci-mingw-bootstrap

.PHONY: dist

build-toolchain:
	GITHUB_ACTIONS=false CARGO_TARGET_RISCV32I_JOLT_ZKVM_ELF_RUSTFLAGS="-Cpasses=loweratomic" ./x build
	GITHUB_ACTIONS=false CARGO_TARGET_RISCV32I_JOLT_ZKVM_ELF_RUSTFLAGS="-Cpasses=loweratomic" ./x build --stage 2

install-toolchain:
	rustup toolchain link riscv32i-jolt-zkvm-elf build/host/stage2

build-install-toolchain:
	make build-toolchain
	make install-toolchain

archive:
	tar -czvf toolchain.tar.gz build/host/stage2
