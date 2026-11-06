use libpackwunix::format::{PackageContainer, WspHeader, WSP_HEADER_SIZE, WSP_MAGIC};
use libpackwunix::manifest::{PackageManifest, PackageTier, PackageType};
use libpackwunix::version::Version;

#[test]
fn test_wsp_header_roundtrip() {
    let header = WspHeader {
        magic: WSP_MAGIC,
        format_version: 1,
        package_type: 0, // system
        flags: 0,
        sig_offset: 128,
        sig_len: 64,
        meta_offset: 192,
        meta_len: 250,
        payload_offset: 442,
        payload_len: 1024,
        payload_sha256: [0xabu8; 32],
    };

    let bytes = header.to_bytes();
    assert_eq!(bytes.len(), WSP_HEADER_SIZE);
    assert_eq!(&bytes[0..4], b"WSP1");

    let parsed = WspHeader::from_bytes(&bytes).expect("Failed to parse WspHeader");
    assert_eq!(parsed.magic, WSP_MAGIC);
    assert_eq!(parsed.format_version, 1);
    assert_eq!(parsed.package_type, 0);
    assert_eq!(parsed.meta_len, 250);
    assert_eq!(parsed.payload_len, 1024);
    assert_eq!(parsed.payload_sha256, [0xabu8; 32]);
}

#[test]
fn test_package_container_build_and_extract() {
    let mut manifest = PackageManifest::default();
    manifest.name = "nano".into();
    manifest.version = Version::new(1, 0, 0);
    manifest.arch = "x86_64".into();
    manifest.package_type = PackageType::User;
    manifest.tier = PackageTier::Optional;
    manifest.description = "Simple text editor".into();

    let payload_dummy = b"MOCK_TAR_GZ_PAYLOAD_FOR_NANO_BINARY";
    let fake_signature = [7u8; 64];

    // Сборка пакета в память
    let container_bytes = PackageContainer::build(&manifest, Some(&fake_signature), payload_dummy)
        .expect("Failed to build container");

    assert!(container_bytes.len() > WSP_HEADER_SIZE);

    // Чтение пакета из памяти
    let container = PackageContainer::parse(&container_bytes)
        .expect("Failed to parse built container");

    assert_eq!(container.header.format_version, 1);
    assert_eq!(container.header.package_type, 1); // user
    assert_eq!(container.manifest.name, "nano");
    assert_eq!(container.manifest.version, Version::new(1, 0, 0));
    assert_eq!(container.signature, Some(fake_signature));
    assert_eq!(container.payload, payload_dummy);
}
