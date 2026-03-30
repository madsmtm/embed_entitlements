use embed_entitlements_macro::convert_entitlements_to_der;

fn main() {
    let _ = include_bytes!("bad_syntax.entitlements"); // rerun if changed
    let _ = convert_entitlements_to_der!("bad_syntax.entitlements");
}
