#![doc = include_str!("../README.md")]
#![no_std]
// Update in Cargo.toml as well.
#![doc(html_root_url = "https://docs.rs/embed_entitlements/0.1.0")]

/// Embed the `.entitlements` file at `$path` directly in the current binary.
///
/// On all targets, this will read the entitlements and add a
/// `__TEXT,__entitlements` section containing them. On simulator targets,
/// this will additionally convert the entitlements to DER, and embed them in
/// a `__TEXT,__ents_der` section (that's what the simulator requires).
///
/// See [the module level docs][crate] for details.
#[macro_export]
macro_rules! embed_entitlements {
    ($path:literal $(,)?) => {
        // The wildcard `_` prevents polluting the call site with identifiers.
        const _: () = {
            // Because `len` is a `const fn`, we can use it to turn `SLICE`
            // into an array that gets directly embedded. This is necessary
            // because the `__entitlements` section must contain the direct
            // data, not a reference to it.
            const SLICE: &[$crate::__core::primitive::u8] = $crate::__core::include_bytes!($path);

            // Prevents this from being optimized out of the binary.
            #[used] // TODO: Use `#[used(linker)]`
            // Places this data in the correct location.
            #[unsafe(link_section = "__TEXT,__entitlements")]
            // Prevents repeated use by creating a linker error.
            // SAFETY: The symbol shouldn't be referenced by anything.
            #[unsafe(no_mangle)]
            static _EMBED_ENTITLEMENT: [$crate::__core::primitive::u8; SLICE.len()] = *unsafe {
                $crate::__core::mem::transmute::<
                    *const $crate::__core::primitive::u8,
                    &[$crate::__core::primitive::u8; _],
                >(SLICE.as_ptr())
            };
        };

        // When compiling for the simulator, convert entitlements to DER and
        // embed it in `__TEXT,__ents`.
        #[cfg(target_env = "sim")]
        const _: () = {
            const SLICE: &[$crate::__core::primitive::u8] =
                $crate::__convert_entitlements_to_der!($path);

            #[used]
            #[unsafe(link_section = "__TEXT,__ents_der")]
            // Don't add no_mangle, we've already asserted uniqueness above.
            static _EMBED_ENT_DER: [$crate::__core::primitive::u8; SLICE.len()] = *unsafe {
                $crate::__core::mem::transmute::<
                    *const $crate::__core::primitive::u8,
                    &[$crate::__core::primitive::u8; _],
                >(SLICE.as_ptr())
            };
        };
    };
}

// Re-exports
#[doc(hidden)]
pub use core as __core;
#[cfg(target_env = "sim")]
#[doc(hidden)]
pub use embed_entitlements_macro::convert_entitlements_to_der as __convert_entitlements_to_der;
