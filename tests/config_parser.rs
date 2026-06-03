use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use hyprland_settings::config_parser::{
    parse_hyprland_config_file, parse_hyprland_config_text, ParseStatus,
};

fn fixture_path(name: &str) -> Result<PathBuf> {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-parser-{name}-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir_all(&root)?;
    Ok(root.join("hyprland.conf"))
}

#[test]
fn parses_supported_scalar_assignments() {
    let path = Path::new("/tmp/hyprland-settings-fixture.conf");
    let parsed = parse_hyprland_config_text(
        path,
        r#"
general:gaps_in = 5
input:kb_layout = us
decoration:rounding = 10
animations:enabled = true
misc:disable_hyprland_logo = true
"#,
    );

    let scalars: Vec<_> = parsed.scalar_records().collect();
    assert_eq!(scalars.len(), 5);
    assert_eq!(
        scalars
            .iter()
            .map(|record| record.normalized_setting_id.as_deref())
            .collect::<Vec<_>>(),
        vec![
            Some("general.gaps_in"),
            Some("input.kb_layout"),
            Some("decoration.rounding"),
            Some("animations.enabled"),
            Some("misc.disable_hyprland_logo"),
        ]
    );
    assert_eq!(scalars[0].raw_value.as_deref(), Some("5"));
    assert_eq!(scalars[1].raw_value.as_deref(), Some("us"));
}

#[test]
fn preserves_comments_blank_lines_line_numbers_and_paths() {
    let path = Path::new("/tmp/line-preservation.conf");
    let parsed =
        parse_hyprland_config_text(path, "\n# comment\ngeneral:gaps_in = 5 # inline comment\n");

    assert_eq!(parsed.records[0].status, ParseStatus::Blank);
    assert_eq!(parsed.records[1].status, ParseStatus::Comment);
    assert_eq!(parsed.records[2].status, ParseStatus::Scalar);
    assert_eq!(parsed.records[2].line_number, 3);
    assert_eq!(
        parsed.records[2].raw_line,
        "general:gaps_in = 5 # inline comment"
    );
    assert_eq!(parsed.records[2].path, path);
    assert_eq!(parsed.records[2].raw_value.as_deref(), Some("5"));
}

#[test]
fn reports_duplicate_scalar_keys() {
    let parsed = parse_hyprland_config_text(
        "/tmp/duplicate.conf",
        "animations:enabled = true\nanimations:enabled = false\n",
    );

    assert_eq!(
        parsed
            .duplicate_scalar_keys
            .get("animations.enabled")
            .cloned(),
        Some(vec![1, 2])
    );
}

#[test]
fn preserves_structured_family_lines_as_raw_records() {
    let parsed = parse_hyprland_config_text(
        "/tmp/structured.conf",
        r#"
monitor = ,preferred,auto,1
bind = SUPER,Return,exec,kitty
windowrule = float,class:^(pavucontrol)$
animation = windows,1,7,default
bezier = snappy,0.2,0.8,0.2,1.0
gesture = 3, horizontal, workspace
permission = /usr/bin/app, screencopy, allow
"#,
    );

    let structured: Vec<_> = parsed
        .records
        .iter()
        .filter(|record| record.status == ParseStatus::StructuredRaw)
        .collect();

    assert_eq!(structured.len(), 7);
    assert_eq!(
        structured
            .iter()
            .map(|record| record.normalized_setting_id.as_deref())
            .collect::<Vec<_>>(),
        vec![
            Some("hl.monitor"),
            Some("hl.bind"),
            Some("hl.windowrule"),
            Some("hl.animation"),
            Some("hl.curve"),
            Some("hl.gesture"),
            Some("hl.permission"),
        ]
    );
    assert!(structured.iter().all(|record| record
        .warning
        .as_deref()
        .unwrap_or("")
        .contains("raw")));
}

#[test]
fn preserves_device_block_as_raw_structured_metadata() {
    let parsed = parse_hyprland_config_text(
        "/tmp/device.conf",
        "device {\n  name = test-device\n  sensitivity = -0.5\n}\n",
    );

    assert_eq!(parsed.records.len(), 4);
    assert!(parsed.records.iter().all(|record| {
        record.status == ParseStatus::StructuredRaw
            && record.normalized_setting_id.as_deref() == Some("hl.device")
    }));
}

#[test]
fn unsupported_lines_are_preserved_with_warning() {
    let parsed = parse_hyprland_config_text(
        "/tmp/unsupported.conf",
        "$terminal = kitty\nsource = ~/.config/hypr/local.conf\n",
    );

    assert_eq!(parsed.records[0].status, ParseStatus::Scalar);
    assert_eq!(parsed.records[0].normalized_setting_id, None);
    assert_eq!(
        parsed.records[0].warning.as_deref(),
        Some("scalar key could not be normalized")
    );
    assert_eq!(parsed.records[1].status, ParseStatus::Scalar);
    assert_eq!(parsed.records[1].normalized_setting_id, None);
}

#[test]
fn parses_config_file_from_fixture_path() -> Result<()> {
    let path = fixture_path("file")?;
    fs::write(&path, "general:gaps_in = 5\n")?;

    let parsed = parse_hyprland_config_file(&path)?;

    assert_eq!(parsed.path, path);
    assert_eq!(parsed.records.len(), 1);
    assert_eq!(
        parsed.records[0].normalized_setting_id.as_deref(),
        Some("general.gaps_in")
    );

    fs::remove_dir_all(path.parent().expect("fixture path should have parent"))?;
    Ok(())
}
