//! A helper proc-macro for `embed_entitlements`.
use std::fmt::Display;
use std::path::Path;

use proc_macro::{Delimiter, Group, Ident, Literal, Punct, Spacing, Span, TokenStream, TokenTree};

/// Read the given entitlement file and convert it to DER / ASN.1.
///
/// Should be combined with a `include_bytes!` of the same path to make the
/// compiler properly track the path.
///
/// # Errors
///
/// Errors if the given entitlement file doesn't exist. May also error if the
/// file isn't a valid entitlement property list (though the quality of errors
/// here is allowed to change in future releases).
#[proc_macro]
pub fn convert_entitlements_to_der(input: TokenStream) -> TokenStream {
    let mut iter = input.into_iter();

    // Parse argument as a string literal
    let Some(path) = iter.next() else {
        return error("must provide a path", Span::call_site());
    };
    // Recurse into fake groups.
    let path = match path {
        TokenTree::Group(group) if group.delimiter() == Delimiter::None => {
            let mut iter = group.stream().into_iter();
            let Some(path) = iter.next() else {
                return error("must provide a path", Span::call_site());
            };
            if let Some(remainder) = iter.next() {
                return error("too many arguments", remainder.span());
            }
            path
        }
        path => path,
    };
    let span = path.span();
    let path = match litrs::StringLit::try_from(path) {
        Ok(string_lit) => string_lit,
        Err(e) => return error(format!("path was not a string: {e}"), span),
    };

    // Handle remaining tokens
    if let Some(remainder) = iter.next() {
        // Allow trailing comma.
        if matches!(&remainder, TokenTree::Punct(punct) if punct.as_char() == ',') {
            if let Some(remainder) = iter.next() {
                return error("too many arguments", remainder.span());
            }
        } else {
            return error("too many arguments", remainder.span());
        }
    }

    // Make path relative to the invocation location. The intent is to find
    // the same path as `include_bytes!`.
    //
    // See https://github.com/rust-lang/rfcs/pull/3200 for doing this better.
    let path = if let Some(file) = span.local_file() {
        file.parent().unwrap().join(path.value())
    } else {
        // `rust-analyzer`'s `proc-macro-srv` doesn't support `local_file`:
        // https://github.com/rust-lang/rust-analyzer/issues/15950
        //
        // We could handle this by emitting a `compile_error!`, but that would
        // cause the user's editor to show annoying red lines. Alternatively,
        // we might silently allow it and try to read a file path relative to
        // the root, or maybe just return an empty `b""`, but that might cause
        // problems if `local_file` actually fails (e.g. due to some weird
        // usage in a macro).
        //
        // Instead, we emit code that will successful compile, but will fail
        // linking. That way, we allow `rust-analyzer` to work, while still
        // catching actual errors.
        //
        // Specifically, we emit two equal `#[no_mangle]` statics.
        let error_symbol = [
            TokenTree::from(Punct::new('#', Spacing::Alone)),
            TokenTree::from(Group::new(
                Delimiter::Bracket,
                TokenStream::from_iter([
                    TokenTree::from(Ident::new("unsafe", span)),
                    TokenTree::from(Group::new(
                        Delimiter::Parenthesis,
                        TokenStream::from_iter([TokenTree::from(Ident::new("no_mangle", span))]),
                    )),
                ]),
            )),
            TokenTree::from(Ident::new("static", span)),
            TokenTree::from(Ident::new("COULD_NOT_FIND_LOCAL_FILE", span)),
            TokenTree::from(Punct::new(':', Spacing::Alone)),
            TokenTree::from(Group::new(Delimiter::Parenthesis, TokenStream::new())),
            TokenTree::from(Punct::new('=', Spacing::Alone)),
            TokenTree::from(Group::new(Delimiter::Parenthesis, TokenStream::new())),
            TokenTree::from(Punct::new(';', Spacing::Alone)),
        ];

        let mut res = TokenStream::new();
        res.extend(error_symbol.clone());
        res.extend(error_symbol);
        res.extend([TokenTree::from(Literal::byte_string(b""))]);
        return TokenTree::from(Group::new(Delimiter::Brace, res)).into();
    };

    // Read and convert the entitlements file at the given path.
    //
    // Wrt. change-tracking and making sure that rustc/Cargo reruns if the
    // file changes, for now it should be sufficient that we use
    // `include_bytes!` on the file in `embed_entitlements!` already, but in
    // the future we should do something smarter, see:
    // https://github.com/rust-lang/rust/issues/99515
    match convert(&path) {
        Ok(bytes) => TokenStream::from_iter([TokenTree::from(Literal::byte_string(&bytes))]),
        Err(e) => error(&e, span),
    }
}

// On host macOS, we rely on `/usr/bin/derq`, to avoid depending on a bunch
// of extra crates. This is the same tool Xcode invokes.
//
// ```sh
// derq query --raw -f xml -i $path -o -
// ```
#[cfg(all(target_os = "macos", not(feature = "pure")))]
fn convert(path: &Path) -> Result<Vec<u8>, String> {
    // `derq` segmentation faults on non-existent files, so let's do this
    // check to have better error messages.
    if !path.try_exists().unwrap_or(true) {
        return Err(format!("couldn't read `{}`: not found", path.display()));
    }

    let suggestion = "Maybe try enabling the `embed_entitlements/pure` feature?";

    let res = std::process::Command::new("derq")
        .arg("query")
        .arg("--raw")
        .arg("-f")
        .arg("xml")
        .arg("-i")
        .arg(path)
        .arg("-o")
        .arg("-")
        .output()
        .map_err(|e| format!("failed spawning `derq`: {e}. {suggestion}"))?;

    if !res.status.success() {
        let stderr = String::from_utf8_lossy(&res.stderr);
        let stderr = if stderr.is_empty() {
            stderr.to_string()
        } else {
            format!("\n{stderr}")
        };
        return Err(format!(
            "failed running `derq`: {}. {suggestion}{}",
            res.status, stderr,
        ));
    }

    Ok(res.stdout)
}

// On other platforms, we use crates to do the conversion ourselves.
#[cfg(not(all(target_os = "macos", not(feature = "pure"))))]
fn convert(path: &Path) -> Result<Vec<u8>, String> {
    let contents = plist::Value::from_file(path)
        .map_err(|e| format!("couldn't read `{}`: {e}", path.display()))?;

    // TODO: Work with upstream to split this into a smaller crate.
    apple_codesign::plist_der::der_encode_plist(&contents)
        .map_err(|e| format!("couldn't convert data to DER: {e}"))
}

/// Emit `compile_error!($s)` with a span pointing to `span`.
///
/// TODO: <https://github.com/rust-lang/rust/issues/54140>.
fn error(s: impl Display, span: Span) -> TokenStream {
    TokenStream::from_iter([
        TokenTree::from(Ident::new("compile_error", span)),
        TokenTree::from(Punct::new('!', Spacing::Alone)),
        TokenTree::from({
            let mut group = Group::new(
                Delimiter::Parenthesis,
                TokenTree::from({
                    let mut literal = Literal::string(&s.to_string());
                    literal.set_span(span);
                    literal
                })
                .into(),
            );
            group.set_span(span);
            group
        }),
    ])
}
