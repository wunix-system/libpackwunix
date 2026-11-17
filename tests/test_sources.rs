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
