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
