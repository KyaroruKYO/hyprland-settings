use hyprland_settings::runtime_preview_dead_man::dead_man_ui_state;
use hyprland_settings::runtime_preview_ui_projection::runtime_preview_ui_row_state;
use hyprland_settings::save_only_pending::is_save_only_editable;
use hyprland_settings::structured_family::StructuredFamilyKind;
use hyprland_settings::write_classification::{
    is_high_risk_gated_writable_setting, SAFE_WRITABLE_ROWS,
};

#[test]
fn save_transition_source_orders_persistence_before_mark_saved() {
    let controller = include_str!("../src/runtime_preview_ui_projection.rs");
    let method = controller
        .split("pub fn persist_and_mark_saved")
        .nth(1)
        .expect("durable Save helper should exist")
        .split("fn mark_saved")
        .next()
        .expect("durable Save helper should have a bounded body");
    let persist = method
        .find("let durable = persist()")
        .expect("persistence must run inside the helper");
    let mark = method
        .find("mark_saved_after_durable_receipt(&durable)")
        .expect("durable receipt must gate Saved state");
    assert!(
        persist < mark,
        "Saved state must follow durable persistence"
    );
    assert!(controller.contains("fn mark_saved(&mut self)"));
    assert!(!controller.contains("pub fn mark_saved(&mut self)"));
}

#[test]
fn every_ui_save_entry_uses_durable_paths_and_batch_state_preflight() {
    let window = include_str!("../src/ui/window.rs");
    assert!(window.contains("persist_and_mark_saved(||"));
    assert!(!window.contains(".mark_saved()"));
    assert!(window.contains("Save all atomically"));
    assert!(!window.contains("Saved {saved_count} of {total}"));

    let batch = window
        .split("save_split.connect_clicked")
        .nth(1)
        .expect("batch Save handler should exist")
        .split("(revealer, refresh)")
        .next()
        .expect("batch handler should have a bounded body");
    let state_preflight = batch
        .find("validate_mark_saved_transition")
        .expect("preview state must be preflighted");
    let write = batch
        .find("gated_safe_batch_save_live")
        .expect("one gated batch write should exist");
    let mark = batch
        .find("mark_saved_after_durable_receipt")
        .expect("durable batch receipt must precede Saved transition");
    assert!(state_preflight < write && write < mark);
    assert!(batch.contains("All changes remain pending"));

    let detail = window
        .split("fn append_write_controls")
        .nth(1)
        .expect("detail save-review handler should exist")
        .split("fn append_detail_line")
        .next()
        .expect("detail handler should have a bounded body");
    assert!(detail.contains("Stage reviewed change"));
    assert!(detail.contains("stage_save_only_change("));
    assert!(!detail.contains("gated_scalar_save_live("));

    let family = include_str!("../src/family_record_picker.rs");
    let family_transition = family
        .split("pub fn persist_and_mark_saved")
        .nth(1)
        .expect("structured preview durable transition should exist")
        .split("fn revert_internal")
        .next()
        .expect("structured transition should have a bounded body");
    let persist = family_transition
        .find("let receipt = persist()?")
        .expect("structured persistence must run first");
    let saved = family_transition
        .find("self.phase = RecordPickerPhase::Saved")
        .expect("structured phase should become Saved");
    assert!(persist < saved);

    for module in [
        include_str!("../src/ui/window.rs"),
        include_str!("../src/production_save.rs"),
        include_str!("../src/write_flow.rs"),
        include_str!("../src/safe_batch_write.rs"),
    ] {
        assert!(!module.contains("Command::new"));
        assert!(!module.contains(".arg(\"reload\")"));
        assert!(!module.contains("&[\"reload\"]"));
    }
}

#[test]
fn active_write_sources_require_exact_preconditions_and_hardened_commit() {
    let scalar = include_str!("../src/scalar_write.rs");
    let batch = include_str!("../src/safe_batch_write.rs");
    let family = include_str!("../src/structured_family_gated_persistence.rs");
    let durable = include_str!("../src/durable_fs.rs");
    let backup = include_str!("../src/config_backup.rs");

    for source in [scalar, batch, family] {
        assert!(source.contains("hardened_atomic_replace"));
    }
    assert!(scalar.contains("validate_source_graph_precondition(plan)?"));
    assert!(scalar.contains("expected_occurrence_count"));
    assert!(batch.contains("multi-file batch rejected before writing"));
    assert!(batch.contains("plan.target_files.len() != 1"));
    assert!(family.contains("source_graph_fingerprint"));
    assert!(
        durable
            .matches("verify_file_precondition(expected)?")
            .count()
            >= 2
    );
    assert!(durable.contains("tempfile_in(parent)"));
    assert!(durable.contains("sync_directory(parent)"));
    assert!(backup.contains("create_new(true)"));
    assert!(backup.contains("Permissions::from_mode(0o600)"));
    assert!(backup.contains("Permissions::from_mode(0o700)"));
    assert!(!backup.contains("std::env::temp_dir"));
}

#[test]
fn real_machine_audits_and_report_regeneration_are_explicitly_gated() {
    let support = include_str!("support/safe_batch_harness.rs");
    assert!(support.contains("HYPRLAND_SETTINGS_RUN_REAL_CONFIG_AUDIT"));
    assert!(support.contains("HYPRLAND_SETTINGS_REGENERATE_REPORTS"));

    let report_writer = support
        .split("pub fn write_report")
        .nth(1)
        .expect("report helper should exist")
        .split("pub fn temp_root")
        .next()
        .expect("report helper should have a bounded body");
    let regeneration_gate = report_writer
        .find("HYPRLAND_SETTINGS_REGENERATE_REPORTS")
        .expect("tracked report writes need an explicit gate");
    let tracked_write = report_writer
        .find("fs::write(&path")
        .expect("explicit regeneration branch should write expected reports");
    let temporary_write = report_writer
        .find("tempfile()")
        .expect("normal tests should serialize to a test-owned tempfile");
    assert!(regeneration_gate < tracked_write && tracked_write < temporary_write);

    for source in [
        include_str!("safe_batch_real_config_hardening.rs"),
        include_str!("source_aware_and_high_risk_hardening.rs"),
        include_str!("structured_family_active_config_pilot.rs"),
    ] {
        assert!(source.contains("#[ignore = \"read-only real-config"));
        assert!(source.contains("HYPRLAND_SETTINGS_RUN_REAL_CONFIG_AUDIT"));
    }
}

#[test]
fn stabilization_does_not_promote_rows_or_structured_families() {
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
    let live = SAFE_WRITABLE_ROWS
        .iter()
        .filter(|row| {
            runtime_preview_ui_row_state(row.row_id).is_some_and(|state| state.preview_enabled)
        })
        .count();
    let dead_man = SAFE_WRITABLE_ROWS
        .iter()
        .filter(|row| dead_man_ui_state(row.row_id).is_some_and(|state| state.arm_enabled))
        .count();
    let save_only = SAFE_WRITABLE_ROWS
        .iter()
        .filter(|row| is_save_only_editable(row.row_id))
        .count();
    let blocked = SAFE_WRITABLE_ROWS
        .iter()
        .filter(|row| {
            is_high_risk_gated_writable_setting(row.row_id)
                && !dead_man_ui_state(row.row_id).is_some_and(|state| state.arm_enabled)
        })
        .count();
    assert_eq!((live, dead_man, save_only, blocked), (135, 38, 117, 51));
    assert_eq!(live + dead_man + save_only, 290);

    assert_eq!(StructuredFamilyKind::ALL.len(), 7);
    let picker = include_str!("../src/family_record_picker.rs");
    let picked_family = picker
        .split("pub enum PickedFamily")
        .nth(1)
        .expect("production picker family enum should exist")
        .split("impl PickedFamily")
        .next()
        .expect("picker enum should have a bounded body");
    assert!(picked_family.contains("Animation"));
    assert!(picked_family.contains("Curve"));
    for blocked_family in ["Monitor", "Bind", "Gesture", "Device", "Permission"] {
        assert!(!picked_family.contains(blocked_family));
    }
}
