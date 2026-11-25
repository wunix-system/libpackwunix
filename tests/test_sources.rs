use libpackwunix::sources::{get_wsp_system_repo, parse_repo_index, parse_repo_sources};

const SOURCES_SAMPLE: &str = r#"
# WPM Sources Configuration

[core-apps]
Server = https://apps.wunix.org/stable/x86_64
Server = https://mirror.wunix.org/apps/x86_64

[community]
Server = https://community.wunix.dev/repo/x86_64
"#;

const PACKAGES_JSON_SAMPLE: &str = r#"{
  "repository": "apps",
  "arch": "x86_64",
  "packages_count": 2,
  "packages": [
    {
      "name": "curl",
      "version": "1.0.0",
      "arch": "x86_64",
      "tier": "optional",
      "type": "user",
      "description": "Сетевой клиент HTTP/HTTPS",
      "filename": "curl-1.0.0.wp",
      "size": 2415523,
      "sha256": "5a925a4c7efb2f586e30f64da0026d51ea0899bee1288197daec3baca3983c62"
    },
    {
      "name": "git",
      "version": "2.43.0",
      "arch": "x86_64",
      "tier": "optional",
      "type": "user",
      "description": "Система контроля версий Git",
      "filename": "git-2.43.0.wp",
      "size": 1925203,
      "sha256": "26818a878c233c8017938d59d38d4c73a9487de474834f09fe6a60c6221ed503"
    }
  ]
}"#;

#[test]
fn test_wsp_system_repo_is_constant() {
    let repo_url = get_wsp_system_repo();
    assert!(repo_url.starts_with("http"));
    assert!(repo_url.contains("x86_64"));
}

#[test]
fn test_repo_sources_parse() {
    let repos = parse_repo_sources(SOURCES_SAMPLE).expect("Failed to parse sources");

    assert_eq!(repos.len(), 2);

    assert_eq!(repos[0].name, "core-apps");
    assert_eq!(repos[0].servers.len(), 2);
    assert_eq!(repos[0].servers[0], "https://apps.wunix.org/stable/x86_64");
    assert_eq!(repos[0].servers[1], "https://mirror.wunix.org/apps/x86_64");

    assert_eq!(repos[1].name, "community");
    assert_eq!(repos[1].servers.len(), 1);
    assert_eq!(repos[1].servers[0], "https://community.wunix.dev/repo/x86_64");
}

#[test]
fn test_parse_repo_index_contains_git() {
    let pkgs = parse_repo_index(PACKAGES_JSON_SAMPLE);
    assert_eq!(pkgs.len(), 2);

    let git_pkg = pkgs.iter().find(|p| p.name == "git").expect("git package not found in index");
    assert_eq!(git_pkg.version.major, 2);
    assert_eq!(git_pkg.version.minor, 43);
    assert_eq!(git_pkg.filename, "git-2.43.0.wp");
    assert_eq!(git_pkg.size, 1925203);
    assert_eq!(git_pkg.sha256, "26818a878c233c8017938d59d38d4c73a9487de474834f09fe6a60c6221ed503");
}

#[test]
fn test_repo_sources_ignores_system_section() {
    let raw = r#"
[system]
Server = http://evil.example.com/system

[SYSTEM]
Server = http://evil2.example.com/system

[apps]
Server = http://wunix.rusos.site/repo/apps/x86_64
"#;
    let repos = parse_repo_sources(raw).expect("Failed to parse sources");
    assert_eq!(repos.len(), 1);
    assert_eq!(repos[0].name, "apps");
    assert_eq!(repos[0].servers[0], "http://wunix.rusos.site/repo/apps/x86_64");
}

#[test]
fn test_parse_repo_index_with_meta_and_dependencies() {
    let raw = r#"{
  "repository": "system",
  "arch": "x86_64",
  "packages": [
    {
      "name": "base",
      "version": "0.1-1234abc",
      "arch": "x86_64",
      "tier": "base",
      "type": "system",
      "meta": true,
      "dependencies": [
        "wunix-base",
        "coreutils"
      ],
      "description": "Metapackage",
      "filename": "base-0.1-1234abc.wp",
      "size": 512,
      "sha256": "deadbeef"
    },
    {
      "name": "curl",
      "version": "0.1-1234abc",
      "arch": "x86_64",
      "tier": "optional",
      "type": "user",
      "meta": false,
      "dependencies": ["libcwunix >= 0.1", "openssl >= 3.4.0"],
      "description": "HTTP client",
      "filename": "curl-0.1-1234abc.wp",
      "size": 1024,
      "sha256": "cafebabe"
    }
  ]
}"#;
    let pkgs = parse_repo_index(raw);
    assert_eq!(pkgs.len(), 2);

    assert_eq!(pkgs[0].name, "base");
    assert!(pkgs[0].is_meta);
    assert_eq!(pkgs[0].dependencies, vec!["wunix-base", "coreutils"]);

    assert_eq!(pkgs[1].name, "curl");
    assert!(!pkgs[1].is_meta);
    assert_eq!(pkgs[1].dependencies, vec!["libcwunix >= 0.1", "openssl >= 3.4.0"]);
}
