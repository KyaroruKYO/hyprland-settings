use std::path::Path;

use anyhow::Result;
use hyprland_settings::config_discovery::{ConfigDiscovery, ConfigDiscoveryStatus};
use hyprland_settings::current_config::CurrentConfigSnapshot;
use hyprland_settings::export::ExportBundle;
use hyprland_settings::metadata::resolve_metadata_path_with_env;
use hyprland_settings::ui::model::{RowDetailProjection, UiProjection};
use hyprland_settings::validation::validate_bundle;

fn load_projection() -> Result<UiProjection> {
    let resolution = resolve_metadata_path_with_env(None, None)?;
    let bundle = ExportBundle::load(Path::new(&resolution.export_dir))?;
    let summary = validate_bundle(&bundle)?;
    Ok(UiProjection::from_bundle(
        &bundle,
        &summary,
        ConfigDiscovery {
            status: ConfigDiscoveryStatus::Missing,
            attempted_paths: Vec::new(),
        },
        CurrentConfigSnapshot::read_unavailable("test fixture has no live config"),
    ))
}

fn detail_for(row_id: &str) -> Result<RowDetailProjection> {
    let projection = load_projection()?;
    Ok(projection
        .detail_for_row(row_id)
        .unwrap_or_else(|| panic!("missing detail for {row_id}")))
}

#[test]
fn row_detail_projection_includes_required_metadata() -> Result<()> {
    let detail = detail_for("animations.enabled")?;

    assert_eq!(detail.row_id, "animations.enabled");
    assert_eq!(detail.official_setting, "animations.enabled");
    assert_eq!(detail.tab_label, "Animations");
    assert!(!detail.label.is_empty());
    assert!(!detail.description.is_empty());
    assert!(!detail.read_support.is_empty());
    assert!(!detail.preview_status.is_empty());
    assert!(!detail.risk_class.is_empty());
    assert_eq!(detail.non_read_status, None);
    assert!(detail
        .safety_notes
        .iter()
        .any(|note| note.contains("read-only text when available")));

    Ok(())
}

#[test]
fn row_detail_projection_has_no_live_values_or_command_strings() -> Result<()> {
    let detail = detail_for("appearance.blur.enabled")?;
    let combined = [
        detail.label,
        detail.row_id,
        detail.official_setting,
        detail.description,
        detail.read_support,
        detail.write_support,
        detail.preview_status,
        detail.risk_class,
        detail.write_candidate_status,
    ]
    .join(" ");
    let forbidden = ["hypr", "ctl"].concat();

    assert!(!combined.contains(forbidden.as_str()));
    assert!(!combined.contains("current value:"));
    assert!(!combined.contains("write command"));

    Ok(())
}

#[test]
fn write_candidate_detail_remains_disabled_metadata() -> Result<()> {
    let detail = detail_for("windows.snap.enabled")?;

    assert_eq!(
        detail.write_candidate_status,
        "active write candidate gated by backup and review"
    );
    assert_eq!(
        detail.write_candidate_target_mode.as_deref(),
        Some("pending-change-only")
    );
    assert_eq!(detail.write_candidate_executable, Some(false));
    assert_eq!(
        detail.write_candidate_command_generation_allowed,
        Some(false)
    );
    assert!(detail.edit.editable);
    assert_eq!(detail.edit.proposed_value.as_deref(), Some("true"));
    let pending = detail.edit.pending.expect("pending projection expected");
    assert_eq!(pending.setting_id, "windows.snap.enabled");
    assert_eq!(pending.validation_label, "valid");
    assert!(!pending.can_review);
    assert!(pending
        .review_summary
        .iter()
        .any(|line| line.contains("current config could not be read")));

    Ok(())
}

#[test]
fn validator_backed_detail_is_editable() -> Result<()> {
    let detail = detail_for("appearance.blur.size")?;

    assert!(detail.edit.editable);
    assert_eq!(detail.edit.proposed_value.as_deref(), Some("0"));
    let pending = detail.edit.pending.expect("pending projection expected");
    assert_eq!(pending.setting_id, "appearance.blur.size");
    assert_eq!(pending.validation_label, "valid");
    assert!(!pending.can_review);

    Ok(())
}

#[test]
fn parser_backed_color_detail_is_editable() -> Result<()> {
    let detail = detail_for("misc.background_color")?;

    assert!(detail.edit.editable);
    assert_eq!(
        detail.edit.proposed_value.as_deref(),
        Some("rgba(ffffffff)")
    );
    let pending = detail.edit.pending.expect("pending projection expected");
    assert_eq!(pending.setting_id, "misc.background_color");
    assert_eq!(pending.validation_label, "valid");
    assert!(!pending.can_review);

    Ok(())
}

#[test]
fn gradient_color_list_detail_is_editable() -> Result<()> {
    let detail = detail_for("general.col.active_border")?;

    assert!(detail.edit.editable);
    assert_eq!(
        detail.edit.proposed_value.as_deref(),
        Some("rgba(ffffffff) rgba(000000ff) 45deg")
    );
    let pending = detail.edit.pending.expect("pending projection expected");
    assert_eq!(pending.setting_id, "general.col.active_border");
    assert_eq!(pending.validation_label, "valid");
    assert!(!pending.can_review);

    Ok(())
}

#[test]
fn vector_tuple_detail_is_editable() -> Result<()> {
    let detail = detail_for("decoration.shadow.offset")?;

    assert!(detail.edit.editable);
    assert_eq!(detail.edit.proposed_value.as_deref(), Some("0 0"));
    let pending = detail.edit.pending.expect("pending projection expected");
    assert_eq!(pending.setting_id, "decoration.shadow.offset");
    assert_eq!(pending.validation_label, "valid");
    assert!(!pending.can_review);

    Ok(())
}

#[test]
fn numeric_list_detail_is_editable() -> Result<()> {
    let detail = detail_for("input.scroll_points")?;

    assert!(detail.edit.editable);
    assert_eq!(
        detail.edit.proposed_value.as_deref(),
        Some("0.2 0.0 0.5 1 1.2 1.5")
    );
    let pending = detail.edit.pending.expect("pending projection expected");
    assert_eq!(pending.setting_id, "input.scroll_points");
    assert_eq!(pending.validation_label, "valid");
    assert!(!pending.can_review);

    Ok(())
}

#[test]
fn enum_custom_string_detail_is_editable() -> Result<()> {
    let detail = detail_for("misc.font_family")?;

    assert!(detail.edit.editable);
    assert_eq!(detail.edit.proposed_value.as_deref(), Some("Sans"));
    let pending = detail.edit.pending.expect("pending projection expected");
    assert_eq!(pending.setting_id, "misc.font_family");
    assert_eq!(pending.validation_label, "valid");
    assert!(!pending.can_review);

    Ok(())
}

#[test]
fn sanitized_path_detail_is_editable() -> Result<()> {
    let detail = detail_for("decoration.screen_shader")?;

    assert!(detail.edit.editable);
    assert_eq!(
        detail.edit.proposed_value.as_deref(),
        Some("~/.config/hypr/example.conf")
    );
    let pending = detail.edit.pending.expect("pending projection expected");
    assert_eq!(pending.setting_id, "decoration.screen_shader");
    assert_eq!(pending.validation_label, "valid");
    assert!(!pending.can_review);

    Ok(())
}

#[test]
fn regex_string_detail_is_editable() -> Result<()> {
    let detail = detail_for("misc.swallow_regex")?;

    assert!(detail.edit.editable);
    assert_eq!(
        detail.edit.proposed_value.as_deref(),
        Some("^(Alacritty|kitty)$")
    );
    let pending = detail.edit.pending.expect("pending projection expected");
    assert_eq!(pending.setting_id, "misc.swallow_regex");
    assert_eq!(pending.validation_label, "valid");
    assert!(!pending.can_review);

    Ok(())
}

#[test]
fn non_allowlisted_detail_explains_disabled_edit_state() -> Result<()> {
    let detail = detail_for("xwayland.enabled")?;

    assert!(!detail.edit.editable);
    assert_eq!(
        detail.edit.disabled_reason.as_deref(),
        Some("not write-allowlisted")
    );
    assert!(detail.edit.pending.is_none());

    Ok(())
}

#[test]
fn row_detail_projection_has_no_live_config_paths() -> Result<()> {
    let projection = load_projection()?;
    let user_config_prefix = ["/home", "kyo", ".config"].join("/");

    for setting in &projection.settings {
        let detail = projection
            .detail_for_row(&setting.row_id)
            .unwrap_or_else(|| panic!("missing detail for {}", setting.row_id));
        let combined = [
            detail.label,
            detail.row_id,
            detail.official_setting,
            detail.tab_label,
            detail.subsection,
            detail.description,
            detail.read_support,
            detail.write_support,
            detail.preview_status,
            detail.risk_class,
            detail.write_candidate_status,
        ]
        .join(" ");

        assert!(!combined.contains(&user_config_prefix));
        assert!(!combined.contains("raw user config"));
        assert!(!combined.contains("included path"));
    }

    Ok(())
}
