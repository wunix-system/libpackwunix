use libpackwunix::sources::{get_wsp_system_repo, parse_repo_sources};

const SOURCES_SAMPLE: &str = r#"
# WPM Sources Configuration

[core-apps]
Server = https://apps.wunix.org/stable/x86_64
Server = https://mirror.wunix.org/apps/x86_64

[community]
Server = https://community.wunix.dev/repo/x86_64
"#;

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
