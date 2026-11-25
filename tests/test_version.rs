use libpackwunix::version::{CompareOp, DependencyReq, Version, VersionReq};

#[test]
fn test_version_parse_and_compare() {
    let v1 = Version::parse("0.4.1").expect("Failed to parse 0.4.1");
    let v2 = Version::parse("0.4.2").expect("Failed to parse 0.4.2");
    let v3 = Version::parse("1.0.0").expect("Failed to parse 1.0.0");

    assert_eq!(v1.major, 0);
    assert_eq!(v1.minor, 4);
    assert_eq!(v1.patch, 1);

    assert!(v2 > v1);
    assert!(v3 > v2);
    assert_eq!(v1, Version::new(0, 4, 1));
}

#[test]
fn test_version_req_matches() {
    let v = Version::new(0, 4, 2);

    let req_gte = VersionReq {
        op: CompareOp::GtEq,
        version: Version::new(0, 4, 0),
    };
    assert!(req_gte.matches(&v));

    let req_strict_gt = VersionReq {
        op: CompareOp::Gt,
        version: Version::new(0, 4, 2),
    };
    assert!(!req_strict_gt.matches(&v));

    let req_exact = VersionReq {
        op: CompareOp::Eq,
        version: Version::new(0, 4, 2),
    };
    assert!(req_exact.matches(&v));
}

#[test]
fn test_dependency_req_parse() {
    let dep1 = DependencyReq::parse("libwunix >= 0.2.0").expect("parse dep with >=");
    assert_eq!(dep1.name, "libwunix");
    assert_eq!(dep1.req.op, CompareOp::GtEq);
    assert_eq!(dep1.req.version, Version::new(0, 2, 0));

    let dep2 = DependencyReq::parse("nano").expect("parse dep without version");
    assert_eq!(dep2.name, "nano");
    assert_eq!(dep2.req.op, CompareOp::Any);
}

#[test]
fn test_commit_version_parse_and_compare() {
    let v_commit1 = Version::parse("0.1-070b090").expect("Failed to parse 0.1-070b090");
    let v_commit2 = Version::parse("0.1-b5ee4ba").expect("Failed to parse 0.1-b5ee4ba");
    let v_base = Version::parse("0.1").expect("Failed to parse 0.1");

    assert_eq!(v_commit1.major, 0);
    assert_eq!(v_commit1.minor, 1);
    assert_eq!(v_commit1.patch, 0);
    assert_eq!(format!("{}", v_commit1), "0.1-070b090");
    assert_ne!(v_commit1, v_commit2);
    assert!(v_commit1 > v_base);

    let dep = DependencyReq::parse("libwunix >= 0.1").expect("parse >= 0.1");
    assert!(dep.req.matches(&v_commit1));
}

#[test]
fn test_monotonic_commit_count_and_no_false_upgrades() {
    let v14 = Version::parse("0.1.14-3ba645b").unwrap();
    let v15_lex_smaller_tag = Version::parse("0.1.15-0000001").unwrap();
    let v13_lex_larger_tag = Version::parse("0.1.13-fffffff").unwrap();
    let v14_other_tag = Version::parse("0.1.14-fffffff").unwrap();
    let v14_dirty = Version::parse("0.1.14-3ba645b-dirty").unwrap();

    assert_eq!(format!("{}", v14), "0.1.14-3ba645b");
    assert!(v15_lex_smaller_tag.is_newer_than(&v14));
    assert!(!v13_lex_larger_tag.is_newer_than(&v14));
    assert!(!v14_other_tag.is_newer_than(&v14));
    assert!(v14_dirty.is_newer_than(&v14));
    assert!(!v14.is_newer_than(&v14_dirty));
    assert!(v15_lex_smaller_tag.is_newer_than(&v14_dirty));
}

#[test]
fn test_transition_from_legacy_and_upstream_semver() {
    let old_commit_only = Version::parse("0.1-baaa495").unwrap();
    let old_base = Version::parse("0.1.0").unwrap();
    let legacy_1_0 = Version::parse("1.0.0").unwrap();
    let new_scoped = Version::parse("0.1.6-76f2ea9").unwrap();

    assert!(new_scoped.is_newer_than(&old_commit_only));
    assert!(new_scoped.is_newer_than(&old_base));
    assert!(new_scoped.is_newer_than(&legacy_1_0));

    let v3_3 = Version::parse("3.3.0").unwrap();
    let v3_4 = Version::parse("3.4.0").unwrap();
    let v3_5 = Version::parse("3.5.0").unwrap();
    assert!(!v3_3.is_newer_than(&v3_4));
    assert!(!v3_4.is_newer_than(&v3_4));
    assert!(v3_5.is_newer_than(&v3_4));
}

#[test]
fn test_content_timestamp_patch_version() {
    let old_app = Version::parse("0.1.2611181501-4f8a21c").unwrap();
    let new_app = Version::parse("0.1.2611251345-0001111").unwrap();
    let legacy_app = Version::parse("0.1-baaa495").unwrap();

    assert_eq!(old_app.patch, 2611181501);
    assert_eq!(format!("{}", old_app), "0.1.2611181501-4f8a21c");
    assert!(new_app.is_newer_than(&old_app));
    assert!(!old_app.is_newer_than(&new_app));
    assert!(old_app.is_newer_than(&legacy_app));
}
