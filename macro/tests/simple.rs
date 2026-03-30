use embed_entitlements_macro::convert_entitlements_to_der;

#[test]
fn convert() {
    let _ = include_bytes!("simple.entitlements"); // rerun if changed
    let ents_der: &[u8] = convert_entitlements_to_der!("simple.entitlements");
    assert_eq!(ents_der, include_bytes!("simple.entitlements.der"));
}
