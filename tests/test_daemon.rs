use libpackwunix::daemon::{DaemonConfig, NotifyMode};

const DAEMON_CONF_SAMPLE: &str = r#"
[daemon]
check_interval = "24h"
check_on_login = true
critical_only = false
notify_mode = "terminal"
preload_packages = false

[network]
timeout_seconds = 15
"#;

#[test]
fn test_daemon_config_parse() {
    let conf = DaemonConfig::parse(DAEMON_CONF_SAMPLE).expect("Failed to parse daemon config");

    assert_eq!(conf.check_interval_seconds, 86400); // 24 hours
    assert!(conf.check_on_login);
    assert!(!conf.critical_only);
    assert_eq!(conf.notify_mode, NotifyMode::Terminal);
    assert!(!conf.preload_packages);
    assert_eq!(conf.timeout_seconds, 15);
}

#[test]
fn test_daemon_duration_parsing() {
    assert_eq!(libpackwunix::daemon::parse_duration_to_seconds("10s").unwrap(), 10);
    assert_eq!(libpackwunix::daemon::parse_duration_to_seconds("5m").unwrap(), 300);
    assert_eq!(libpackwunix::daemon::parse_duration_to_seconds("2h").unwrap(), 7200);
    assert_eq!(libpackwunix::daemon::parse_duration_to_seconds("1d").unwrap(), 86400);
    assert_eq!(libpackwunix::daemon::parse_duration_to_seconds("3600").unwrap(), 3600);
}
