use std::collections::BTreeSet;
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use anyhow::Result;
use hyprland_settings::config_backup::BackupManager;
use hyprland_settings::config_parser::parse_hyprland_config_text;
use hyprland_settings::current_config::{CurrentConfigSnapshot, CurrentValueProjection};
use hyprland_settings::durable_fs::DurableFsError;
use hyprland_settings::pending_change::stage_pending_change;
use hyprland_settings::scalar_write::{apply_scalar_write_plan, ScalarWriteError};
use hyprland_settings::source_aware_current_config::current_source_graph_fingerprint;
use hyprland_settings::write_classification::{
    config_key_from_official_setting, finite_choice_options, is_high_risk_gated_writable_setting,
    safe_writable_official_setting, SafeWritableRow, ScalarWriteValueKind, SAFE_WRITABLE_ROWS,
};
use hyprland_settings::write_safety::{review_write_plan, WritePlan, WritePlanRequest};

fn temp_root(name: &str) -> Result<PathBuf> {
    let stamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-scalar-write-{name}-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir_all(&root)?;
    Ok(root)
}

fn known_ids() -> BTreeSet<String> {
    SAFE_WRITABLE_ROWS
        .iter()
        .map(|row| row.row_id.to_string())
        .collect()
}

fn current_value_for(path: &PathBuf, setting_id: &str, contents: &str) -> CurrentValueProjection {
    let parsed = parse_hyprland_config_text(path, contents);
    CurrentConfigSnapshot::from_parsed(parsed).value_for(setting_id)
}

fn reviewed_plan(
    root: &std::path::Path,
    source: &PathBuf,
    setting_id: &str,
    proposed_value: &str,
) -> Result<WritePlan> {
    let contents = fs::read_to_string(source)?;
    let official_setting =
        safe_writable_official_setting(setting_id).expect("fixture row should be safe-writable");
    let current = current_value_for(source, official_setting, &contents);
    let backup = BackupManager::new(root.join("backups")).create_backup(source)?;
    let pending = stage_pending_change(setting_id, &current, proposed_value);
    let review = review_write_plan(WritePlanRequest {
        known_setting_ids: known_ids(),
        detected_config_path: source.clone(),
        current_value: current,
        pending_change: pending,
        backup: Some(backup),
    });
    Ok(review.plan.expect("fixture write should pass review"))
}

fn valid_value_for(row: &SafeWritableRow) -> &'static str {
    if row.value_kind == ScalarWriteValueKind::SourceBacked {
        return match row.row_id {
            "input.kb_model" => "pc105",
            "input.kb_layout" => "us,de",
            "input.kb_variant" => "intl",
            "input.kb_options" => "grp:alt_shift_toggle,ctrl:nocaps",
            "input.kb_rules" => "evdev",
            _ => "us",
        };
    }
    if row.value_kind == ScalarWriteValueKind::MonitorName {
        return "";
    }
    if row.value_kind == ScalarWriteValueKind::Color {
        return "rgba(ffffffff)";
    }
    if row.value_kind == ScalarWriteValueKind::Gradient {
        return "rgba(ffffffff) rgba(000000ff) 45deg";
    }
    if row.value_kind == ScalarWriteValueKind::Vector2 {
        return "10 20";
    }
    if row.value_kind == ScalarWriteValueKind::NumericList {
        return "0.2 0.0 0.5 1 1.2 1.5";
    }
    if row.value_kind == ScalarWriteValueKind::CssGap {
        return "5 10 15 20";
    }
    if row.value_kind == ScalarWriteValueKind::AccelProfile {
        return "custom 0.2 0.0 0.5 1";
    }
    if row.value_kind == ScalarWriteValueKind::CommaSeparatedFloatList {
        return "0.333, 0.5, 0.667, 1.0";
    }
    if row.value_kind == ScalarWriteValueKind::LineSafeString {
        return "JetBrains Mono";
    }
    if row.value_kind == ScalarWriteValueKind::Path {
        return "~/.config/hypr/example.conf";
    }
    if row.value_kind == ScalarWriteValueKind::RegexString {
        return "^(Alacritty|kitty)$";
    }
    if row.value_kind == ScalarWriteValueKind::Boolean {
        return "true";
    }
    if row.value_kind == ScalarWriteValueKind::FiniteChoice {
        return finite_choice_options(row.row_id)
            .and_then(|options| options.get(1).or_else(|| options.first()))
            .map(|option| option.raw_value)
            .expect("finite choice row should have verified options");
    }
    if row.row_id == "input.pointer_sensitivity" {
        return "-0.25";
    }
    if matches!(
        row.row_id,
        "appearance.shadow.render_power"
            | "appearance.glow.render_power"
            | "group.groupbar.priority"
            | "misc.force_default_wallpaper"
            | "misc.initial_workspace_tracking"
            | "binds.workspace_center_on"
            | "binds.focus_preferred_method"
    ) {
        return "1";
    }
    if matches!(
        row.row_id,
        "input.touchdevice.transform" | "input.tablet.transform" | "misc.anr_missed_pings"
    ) {
        return "2";
    }
    if matches!(
        row.row_id,
        "appearance.rounding_power"
            | "group.groupbar.rounding_power"
            | "group.groupbar.gradient_rounding_power"
    ) {
        return "2.5";
    }
    if matches!(
        row.row_id,
        "appearance.blur.noise"
            | "dwindle.split_width_multiplier"
            | "dwindle.default_split_ratio"
            | "master.mfact"
            | "scrolling.column_width"
            | "scrolling.follow_min_visible"
    ) {
        return "0.5";
    }
    if row.value_kind == ScalarWriteValueKind::Percent {
        return "0.75";
    }
    if row.value_kind == ScalarWriteValueKind::Number {
        return "10";
    }
    "10"
}

fn existing_value_for(row: &SafeWritableRow) -> &'static str {
    if row.value_kind == ScalarWriteValueKind::SourceBacked {
        return match row.row_id {
            "input.kb_model" => "pc104",
            "input.kb_layout" => "us",
            "input.kb_variant" => "",
            "input.kb_options" => "ctrl:nocaps",
            "input.kb_rules" => "base",
            _ => "us",
        };
    }
    if row.value_kind == ScalarWriteValueKind::MonitorName {
        return "";
    }
    if row.value_kind == ScalarWriteValueKind::Color {
        return "rgba(000000ff)";
    }
    if row.value_kind == ScalarWriteValueKind::Gradient {
        return "rgba(000000ff)";
    }
    if row.value_kind == ScalarWriteValueKind::Vector2 {
        return "0 0";
    }
    if row.value_kind == ScalarWriteValueKind::NumericList {
        return "0.2 0.5 1";
    }
    if row.value_kind == ScalarWriteValueKind::CssGap {
        return "5";
    }
    if row.value_kind == ScalarWriteValueKind::AccelProfile {
        return "flat";
    }
    if row.value_kind == ScalarWriteValueKind::CommaSeparatedFloatList {
        return "0.25,0.75";
    }
    if row.value_kind == ScalarWriteValueKind::LineSafeString {
        return "Sans";
    }
    if row.value_kind == ScalarWriteValueKind::Path {
        return "./old";
    }
    if row.value_kind == ScalarWriteValueKind::RegexString {
        return "firefox";
    }
    if row.value_kind == ScalarWriteValueKind::Boolean {
        return "false";
    }
    if row.value_kind == ScalarWriteValueKind::FiniteChoice {
        return finite_choice_options(row.row_id)
            .and_then(|options| options.first())
            .map(|option| option.raw_value)
            .expect("finite choice row should have verified options");
    }
    if row.row_id == "input.pointer_sensitivity" {
        return "0";
    }
    if row.value_kind == ScalarWriteValueKind::Percent {
        return "0.5";
    }
    if row.value_kind == ScalarWriteValueKind::Number {
        return "5";
    }
    "5"
}

#[test]
fn generic_scalar_writer_replaces_each_safe_writable_row() -> Result<()> {
    for row in SAFE_WRITABLE_ROWS
        .iter()
        .filter(|row| !is_high_risk_gated_writable_setting(row.row_id))
    {
        let root = temp_root(row.row_id)?;
        let source = root.join("hyprland.conf");
        let config_key = config_key_from_official_setting(row.official_setting);
        let proposed = valid_value_for(row);
        fs::write(
            &source,
            format!("{config_key} = {} # keep\n", existing_value_for(row)),
        )?;
        let contents = fs::read_to_string(&source)?;
        let backup = BackupManager::new(root.join("backups")).create_backup(&source)?;
        let current = current_value_for(&source, row.official_setting, &contents);
        let pending = stage_pending_change(row.row_id, &current, proposed);
        let review = review_write_plan(WritePlanRequest {
            known_setting_ids: known_ids(),
            detected_config_path: source.clone(),
            current_value: current,
            pending_change: pending,
            backup: Some(backup),
        });

        let result = apply_scalar_write_plan(&review.plan.expect("safe toggle should plan"))?;

        assert_eq!(result.verified_value.as_deref(), Some(proposed));
        assert_eq!(
            fs::read_to_string(&source)?,
            format!("{config_key} = {proposed} # keep\n")
        );
        fs::remove_dir_all(root)?;
    }
    Ok(())
}

#[test]
fn generic_scalar_writer_appends_missing_safe_writable_row() -> Result<()> {
    for row in SAFE_WRITABLE_ROWS
        .iter()
        .filter(|row| !is_high_risk_gated_writable_setting(row.row_id))
    {
        let root = temp_root(&format!("{}-append", row.row_id))?;
        let source = root.join("hyprland.conf");
        let config_key = config_key_from_official_setting(row.official_setting);
        let proposed = valid_value_for(row);
        fs::write(&source, "# insertion fixture\n")?;
        let backup = BackupManager::new(root.join("backups")).create_backup(&source)?;
        let current = CurrentValueProjection::not_configured();
        let pending = stage_pending_change(row.row_id, &current, proposed);
        let review = review_write_plan(WritePlanRequest {
            known_setting_ids: known_ids(),
            detected_config_path: source.clone(),
            current_value: current,
            pending_change: pending,
            backup: Some(backup),
        });

        let result = apply_scalar_write_plan(&review.plan.expect("safe toggle should plan"))?;

        assert_eq!(result.verified_value.as_deref(), Some(proposed));
        assert!(fs::read_to_string(&source)?.contains(&format!("{config_key} = {proposed}")));
        fs::remove_dir_all(root)?;
    }
    Ok(())
}

#[test]
fn finite_choice_writer_roundtrips_every_verified_choice() -> Result<()> {
    for row in SAFE_WRITABLE_ROWS.iter().filter(|row| {
        row.value_kind == ScalarWriteValueKind::FiniteChoice
            && !is_high_risk_gated_writable_setting(row.row_id)
    }) {
        let row_id = row.row_id;
        let config_key = config_key_from_official_setting(row.official_setting);
        let options = finite_choice_options(row_id).expect("finite choices should exist");

        for option in options {
            let root = temp_root(&format!("{row_id}-{}", option.raw_value))?;
            let source = root.join("hyprland.conf");
            fs::write(&source, format!("{config_key} = 0\n"))?;
            let contents = fs::read_to_string(&source)?;
            let backup = BackupManager::new(root.join("backups")).create_backup(&source)?;
            let current = current_value_for(&source, row.official_setting, &contents);
            let pending = stage_pending_change(row.row_id, &current, option.raw_value);
            let review = review_write_plan(WritePlanRequest {
                known_setting_ids: known_ids(),
                detected_config_path: source.clone(),
                current_value: current,
                pending_change: pending,
                backup: Some(backup),
            });

            let result = apply_scalar_write_plan(
                &review
                    .plan
                    .expect("verified finite choice should pass write safety"),
            )?;

            assert_eq!(result.verified_value.as_deref(), Some(option.raw_value));
            assert_eq!(
                fs::read_to_string(&source)?,
                format!("{config_key} = {}\n", option.raw_value),
                "{row_id} should roundtrip {} ({})",
                option.raw_value,
                option.label
            );
            fs::remove_dir_all(root)?;
        }
    }
    Ok(())
}

#[test]
fn scalar_replacement_rejects_same_setting_and_unrelated_external_drift() -> Result<()> {
    for external_bytes in [
        b"general:snap:enabled = true\n".as_slice(),
        b"# unrelated external edit\ngeneral:snap:enabled = false\n".as_slice(),
    ] {
        let root = temp_root("replacement-drift")?;
        let source = root.join("hyprland.conf");
        fs::write(&source, "general:snap:enabled = false\n")?;
        let plan = reviewed_plan(&root, &source, "windows.snap.enabled", "true")?;

        fs::write(&source, external_bytes)?;
        let error = apply_scalar_write_plan(&plan)
            .expect_err("any target-file byte drift must abort the replacement");

        assert!(matches!(
            error,
            ScalarWriteError::Drift(DurableFsError::OnDiskDriftDetected(_))
        ));
        assert_eq!(fs::read(&source)?, external_bytes);
        fs::remove_dir_all(root)?;
    }
    Ok(())
}

#[test]
fn scalar_insertion_rejects_external_setting_appearance_and_duplicates() -> Result<()> {
    let root = temp_root("insertion-drift")?;
    let source = root.join("hyprland.conf");
    fs::write(&source, "# setting initially absent\n")?;
    let plan = reviewed_plan(&root, &source, "windows.snap.enabled", "true")?;

    let externally_added = b"# setting initially absent\ngeneral:snap:enabled = false\n";
    fs::write(&source, externally_added)?;
    let error = apply_scalar_write_plan(&plan)
        .expect_err("appearance of a formerly absent setting must abort insertion");
    assert!(matches!(
        error,
        ScalarWriteError::Drift(DurableFsError::OnDiskDriftDetected(_))
    ));
    assert_eq!(fs::read(&source)?, externally_added);

    let duplicate_root = temp_root("duplicate-drift")?;
    let duplicate_source = duplicate_root.join("hyprland.conf");
    fs::write(&duplicate_source, "general:snap:enabled = false\n")?;
    let duplicate_plan = reviewed_plan(
        &duplicate_root,
        &duplicate_source,
        "windows.snap.enabled",
        "true",
    )?;
    let duplicate_bytes = b"general:snap:enabled = false\ngeneral:snap:enabled = true\n";
    fs::write(&duplicate_source, duplicate_bytes)?;
    assert!(matches!(
        apply_scalar_write_plan(&duplicate_plan),
        Err(ScalarWriteError::Drift(
            DurableFsError::OnDiskDriftDetected(_)
        ))
    ));
    assert_eq!(fs::read(&duplicate_source)?, duplicate_bytes);

    fs::remove_dir_all(root)?;
    fs::remove_dir_all(duplicate_root)?;
    Ok(())
}

#[test]
fn scalar_replacement_rejects_same_bytes_with_replaced_inode() -> Result<()> {
    let root = temp_root("inode-drift")?;
    let source = root.join("hyprland.conf");
    let original = b"general:snap:enabled = false\n";
    fs::write(&source, original)?;
    let plan = reviewed_plan(&root, &source, "windows.snap.enabled", "true")?;

    fs::rename(&source, root.join("replaced-original.conf"))?;
    fs::write(&source, original)?;
    let error = apply_scalar_write_plan(&plan)
        .expect_err("same bytes in a replacement inode must not satisfy the plan");

    assert!(matches!(
        error,
        ScalarWriteError::Drift(DurableFsError::TargetIdentityChanged(_))
    ));
    assert_eq!(fs::read(&source)?, original);
    fs::remove_dir_all(root)?;
    Ok(())
}

#[test]
fn scalar_replacement_rejects_source_include_mapping_drift() -> Result<()> {
    let root = temp_root("source-graph-drift")?;
    let graph_root = root.join("hyprland.conf");
    let source_a = root.join("settings-a.conf");
    let source_b = root.join("settings-b.conf");
    fs::write(&graph_root, "source = settings-a.conf\n")?;
    fs::write(&source_a, "general:snap:enabled = false\n")?;
    fs::write(&source_b, "general:snap:enabled = false\n")?;
    let mut plan = reviewed_plan(&root, &source_a, "windows.snap.enabled", "true")?;
    plan.source_graph_root = Some(graph_root.clone());
    plan.source_graph_fingerprint = current_source_graph_fingerprint(&graph_root);
    assert!(plan.source_graph_fingerprint.is_some());

    fs::write(&graph_root, "source = settings-b.conf\n")?;
    let error = apply_scalar_write_plan(&plan)
        .expect_err("source/include target remapping must abort before target write");

    assert!(matches!(error, ScalarWriteError::SourceGraphChanged(_)));
    assert_eq!(
        fs::read_to_string(&source_a)?,
        "general:snap:enabled = false\n"
    );
    fs::remove_dir_all(root)?;
    Ok(())
}
