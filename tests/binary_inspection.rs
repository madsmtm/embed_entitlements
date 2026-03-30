use object::{Object, ObjectSection};

embed_entitlements::embed_entitlements!("../keychain.entitlements");

/// Read this test binary, and verify that it contains the entitlements in the
/// expected sections.
#[test]
fn binary_contains_entitlements() {
    let file = std::fs::read(std::env::current_exe().unwrap()).unwrap();
    let file = object::File::parse(&*file).unwrap();

    let section = file
        .section_by_name("__entitlements")
        .expect("did not contain section");
    assert_eq!(section.segment_name().unwrap().unwrap(), "__TEXT");
    assert_eq!(
        section.data().unwrap().to_vec(),
        include_bytes!("../keychain.entitlements")
    );

    if cfg!(target_env = "sim") {
        let section = file
            .section_by_name("__ents_der")
            .expect("did not contain section");
        assert_eq!(section.segment_name().unwrap().unwrap(), "__TEXT");
        assert_eq!(
            section.data().unwrap().to_vec(),
            b"\x70\x3c\x02\x01\x01\xb0\x37\x30\x35\x0c\x16keychain-access-groups\x30\x1b\x0c\x19com.somecompany.testgroup"
        );
    }
}
