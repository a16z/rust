// ignore-tidy-linelength
// revisions: ENABLE_SVE DISABLE_SVE DISABLE_NEON ENABLE_NEON
// compile-flags: --crate-type=rlib --target=aarch64-unknown-linux-gnu
// needs-llvm-components: aarch64

// [ENABLE_SVE] compile-flags: -C target-feature=+sve
// ENABLE_SVE: attributes #0 = { {{.*}} "target-features"="+outline-atomics,+sve,+neon,+v8a" }

// [DISABLE_SVE] compile-flags: -C target-feature=-sve
// DISABLE_SVE: attributes #0 = { {{.*}} "target-features"="+outline-atomics,-sve,+v8a" }

// [DISABLE_NEON] compile-flags: -C target-feature=-neon
// DISABLE_NEON: attributes #0 = { {{.*}} "target-features"="+outline-atomics,-neon,-fp-armv8,+v8a" }

// [ENABLE_NEON] compile-flags: -C target-feature=+neon
// ENABLE_NEON: attributes #0 = { {{.*}} "target-features"="+outline-atomics,+neon,+fp-armv8,+v8a" }


#![feature(no_core, lang_items)]
#![no_core]

#[lang = "sized"]
trait Sized {}

pub fn test() {}
