use libpackwunix::manifest::{PackageManifest, PackageTier, PackageType};
use libpackwunix::version::{CompareOp, Version};

const SAMPLE_MANIFEST: &str = r#"
[package]
name = "wunix-kernel"
version = "0.4.2"
arch = "x86_64"
type = "system"
tier = "critical"
description = "Monolithic Wunix Kernel with preemptive scheduler"
maintainer = "Wunix Core Team <dev@wunix.org>"

[dependencies]
required = [
    "libwunix >= 0.2.0",
    "init"
]
conflicts = [
    "old-kernel"
]
replaces = [
    "wunix-kernel-legacy"
]

[triggers]
post_install = "update-bootloader"
post_remove = "clean-old-kernels"
"#;

#[test]
fn test_parse_valid_manifest() {
    let manifest = PackageManifest::parse(SAMPLE_MANIFEST).expect("Failed to parse valid manifest");

    assert_eq!(manifest.name, "wunix-kernel");
    assert_eq!(manifest.version, Version::new(0, 4, 2));
    assert_eq!(manifest.arch, "x86_64");
    assert_eq!(manifest.package_type, PackageType::System);
    assert_eq!(manifest.tier, PackageTier::Critical);
    assert_eq!(manifest.description, "Monolithic Wunix Kernel with preemptive scheduler");
    assert_eq!(manifest.maintainer, "Wunix Core Team <dev@wunix.org>");

    assert_eq!(manifest.required_deps.len(), 2);
    assert_eq!(manifest.required_deps[0].name, "libwunix");
    assert_eq!(manifest.required_deps[0].req.op, CompareOp::GtEq);
    assert_eq!(manifest.required_deps[0].req.version, Version::new(0, 2, 0));
    assert_eq!(manifest.required_deps[1].name, "init");
    assert_eq!(manifest.required_deps[1].req.op, CompareOp::Any);

    assert_eq!(manifest.conflicts, vec!["old-kernel"]);
    assert_eq!(manifest.replaces, vec!["wunix-kernel-legacy"]);

    assert_eq!(manifest.post_install.as_deref(), Some("update-bootloader"));
    assert_eq!(manifest.post_remove.as_deref(), Some("clean-old-kernels"));
}

#[test]
fn test_manifest_serialize_roundtrip() {
    let manifest = PackageManifest::parse(SAMPLE_MANIFEST).expect("Parse initial");
    let serialized = manifest.to_toml_string();

    let manifest2 = PackageManifest::parse(&serialized).expect("Parse serialized");
    assert_eq!(manifest.name, manifest2.name);
    assert_eq!(manifest.version, manifest2.version);
    assert_eq!(manifest.package_type, manifest2.package_type);
    assert_eq!(manifest.tier, manifest2.tier);
    assert_eq!(manifest.required_deps, manifest2.required_deps);
    assert_eq!(manifest.post_install, manifest2.post_install);
}
