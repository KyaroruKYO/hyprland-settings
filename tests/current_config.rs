use hyprland_settings::config_parser::parse_hyprland_config_text;
use hyprland_settings::current_config::CurrentConfigSnapshot;

#[test]
fn structured_family_counts_are_grouped_read_only() {
    let parsed = parse_hyprland_config_text(
        "/tmp/structured-current.conf",
        r#"
monitor = ,preferred,auto,1
bind = SUPER,Return,exec,kitty
animation = windows,1,7,default
bezier = snappy,0.2,0.8,0.2,1.0
gesture = 3, horizontal, workspace
device {
  name = test-device
}
permission = /usr/bin/app, screencopy, allow
"#,
    );
    let snapshot = CurrentConfigSnapshot::from_parsed(parsed);
    let counts = snapshot.structured_family_counts();

    assert_eq!(counts.get("hl.monitor"), Some(&1));
    assert_eq!(counts.get("hl.bind"), Some(&1));
    assert_eq!(counts.get("hl.animation"), Some(&1));
    assert_eq!(counts.get("hl.curve"), Some(&1));
    assert_eq!(counts.get("hl.gesture"), Some(&1));
    assert_eq!(counts.get("hl.device"), Some(&3));
    assert_eq!(counts.get("hl.permission"), Some(&1));
    assert!(snapshot
        .structured_summary()
        .contains("Structured config entries preserved read-only"));
}

#[test]
fn structured_summary_reports_empty_state() {
    let parsed = parse_hyprland_config_text(
        "/tmp/no-structured-current.conf",
        "animations:enabled = true\n",
    );
    let snapshot = CurrentConfigSnapshot::from_parsed(parsed);

    assert_eq!(
        snapshot.structured_summary(),
        "Structured config entries: none parsed."
    );
}
