//! UI tests.
//!
//! Bless with:
//! ```sh
//! TRYBUILD=overwrite cargo test --package embed_entitlements_macro --test ui
//! ```

#[cfg(target_os = "macos")] // Only run UI tests on host macOS.
#[test]
fn ui() {
    if cfg!(feature = "pure") {
        return;
    }

    let t = trybuild::TestCases::new();
    t.compile_fail("tests/ui/*.rs");
}
