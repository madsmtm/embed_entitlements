# Embed entitlements

[![Latest version](https://badgen.net/crates/v/embed_entitlements)](https://crates.io/crates/embed_entitlements)
[![License](https://badgen.net/badge/license/Zlib%20OR%20Apache-2.0%20OR%20MIT/blue)](./README.md#license)
[![Documentation](https://docs.rs/embed_entitlements/badge.svg)](https://docs.rs/embed_entitlements/)
[![CI](https://github.com/madsmtm/embed_entitlements/actions/workflows/ci.yml/badge.svg)](https://github.com/madsmtm/embed_entitlements/actions/workflows/ci.yml)

Embed `.entitlements` directly in your executable binary.


## Motivation

On Apple platforms (macOS, iOS, tvOS, watchOS and visionOS), applications are
sandboxed (optionally on macOS, required on the others). Granting your
application additional capabilities to functionality outside the sandbox is
controlled using "entitlements", see
[Apple's documentation](https://developer.apple.com/documentation/bundleresources/entitlements).

Usually, these are applied when code signing, such as by resigning a binary in
the following manner:
```sh
codesign --force --sign $IDENTITY --entitlements foo.entitlements target/debug/foo
```

For binaries that run on the simulator (Simulator.app / `xcrun simctl`),
things are a bit different though; the binary runs on the host macOS XNU
kernel, which means that the binary's entitlements are applied in a macOS
context. So you don't want to sign it with iOS entitlements (at a maximum, you
want to sign it with the `com.apple.security.get-task-allow` entitlement).

Instead, the entitlements that the simulator should apply is embedded inside
the binary in the `__TEXT,__entitlements` section.

Xcode [does this automatically](https://github.com/swiftlang/swift-build/blob/swift-6.3-RELEASE/Sources/SWBApplePlatform/Specs/Embedded-Simulator.xcspec)
by invoking the linker with `-sectcreate`, but if you want to run your binary
on the simulator without building it in Xcode, you can use this crate to do
the embedding.

This crate works well with the [`embed_plist`](https://docs.rs/embed_plist/)
crate for embedding an application's `Info.plist`.


## Simulator usage

Add the `embed_entitlements` crate when your project is compiled for a
simulator target:

```sh
cargo add embed_entitlements --target 'cfg(target_env = "sim")'
```

Write the desired entitlements to a file (in this example
[`keychain.entitlements`](./keychain.entitlements)):

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
	<key>keychain-access-groups</key>
	<array>
		<string>com.somecompany.testgroup</string>
	</array>
</dict>
</plist>
```

And tell `embed_entitlements` to embed that file in your binary:

```rust
#[cfg(target_env = "sim")]
embed_entitlements::embed_entitlements!("keychain.entitlements");
```


## General usage

Annoyingly, this separation between entitlements on real devices and in the
simulator means that you have to mention the entitlement file in multiple
places.

Ideally, you would just be able to write:
```rust
#[cfg(target_os = "ios")]
embed_entitlements::embed_entitlements!("keychain.entitlements");
```

And have that work regardless of whether you're compiling for the simulator or
a real device.

Projects that bundle and sign applications _can_ make this work though! All
they have to do is something like the following:

```rust,ignore
let file = object::File::parse(&*file).unwrap();

if let Some(section) = file.section_by_name("__entitlements")
    && target.env != "sim"
{
    let entitlements = section.data().unwrap().to_vec();
    // invoke `codesign` with `--entitlements=$entitlements` flag.
} else {
    // invoke `codesign` without entitlements.
}
```

Current the following projects support this usage:
- None.

See [#2](https://github.com/madsmtm/embed_entitlements/issues/2) for tracking
doing this in the Rust ecosystem.


## DER

On newer OS versions, the simulator actually reads from `__TEXT,__ents_der`,
which contains the entitlement re-encoded in DER / ASN.1 format, see
[this tech note](https://developer.apple.com/documentation/technotes/tn3125-inside-code-signing-provisioning-profiles#The-future-is-DER)
for details.

On `target_env = "sim"`, this crate automatically converts the entitlements
file to DER and also embeds that in the `__TEXT,__ents_der` section. This
requires a procedural macro.


## Multi-use protection

Only one entitlement file can exist in a binary, and accidentally embedding
it multiple times would break in weird ways. This library makes reuse a
link-time error, which means the error happens even if the macro is used
across different crates and modules.

```compile_fail
embed_entitlements::embed_entitlements!("keychain.entitlements");
embed_entitlements::embed_entitlements!("keychain.entitlements");
```

This example produces an error like this:

```text
error: symbol `_EMBED_ENTITLEMENT` is already defined
 --> srcmain.rs
 |
 | embed_entitlements::embed_entitlements!("keychain.entitlements");
 | ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^
```


## Minimum Supported Rust Version

This library targets **1.91** as its minimum supported Rust version (MSRV),
because it uses `target_env = "sim"`. This could possibly be lowered if need
be, feel free to open an issue if you need it.


## License

This project is trio-licensed under the [Zlib], [Apache-2.0] or [MIT] license,
at your option.

[MIT]: ./LICENSE-MIT.txt
[Zlib]: ./LICENSE-ZLIB.txt
[Apache-2.0]: ./LICENSE-APACHE.txt
