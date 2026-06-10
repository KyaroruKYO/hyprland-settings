use std::path::PathBuf;

use hyprland_settings::config_selection::{
    ConfigSelectionLifecycle, ConfigSelectionSource, ConfigSelectionState, SourceFollowChoice,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn auto_detected_config_state_can_be_represented() {
    let detected = PathBuf::from("/tmp/hyprland.conf");
    let state = ConfigSelectionState::auto_detected(&detected);
    let preview = state.preview();

    assert_eq!(preview.detected_config, Some(detected.clone()));
    assert_eq!(preview.selected_for_review, None);
    assert_eq!(
        preview.selection_source,
        ConfigSelectionSource::AutoDetected
    );
    assert_eq!(
        preview.source_follow_choice,
        SourceFollowChoice::ReviewAllConnectedFiles
    );
    assert!(!preview.preview_only);
    assert!(!preview.confirmed);
    assert!(!preview.cancelled);
    assert_eq!(state.write_target_path(), Some(&detected));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn manual_preview_state_is_preview_only_and_not_write_target() {
    let detected = PathBuf::from("/tmp/root.conf");
    let manual = PathBuf::from("/tmp/other.conf");
    let state = ConfigSelectionState::auto_detected(&detected)
        .preview_manual_config(&manual, SourceFollowChoice::OnlySelectedFile);
    let preview = state.preview();

    assert_eq!(preview.detected_config, Some(detected.clone()));
    assert_eq!(preview.selected_for_review, Some(manual));
    assert_eq!(
        preview.selection_source,
        ConfigSelectionSource::ManualPreview
    );
    assert_eq!(
        preview.source_follow_choice,
        SourceFollowChoice::OnlySelectedFile
    );
    assert!(preview.preview_only);
    assert!(!preview.confirmed);
    assert_eq!(state.write_target_path(), Some(&detected));
}

#[test]
fn source_follow_cancel_and_confirm_states_are_represented_without_persistence() {
    let confirmed = ConfigSelectionState::auto_detected("/tmp/root.conf")
        .preview_manual_config(
            "/tmp/manual.conf",
            SourceFollowChoice::ReviewAllConnectedFiles,
        )
        .confirm_preview();
    let confirmed_preview = confirmed.preview();
    assert_eq!(
        confirmed.lifecycle,
        ConfigSelectionLifecycle::ConfirmedForFutureReview
    );
    assert!(confirmed_preview.preview_only);
    assert!(confirmed_preview.confirmed);
    assert_eq!(
        confirmed.write_target_path(),
        Some(&PathBuf::from("/tmp/root.conf"))
    );

    let cancelled = confirmed.cancel_preview();
    let cancelled_preview = cancelled.preview();
    assert_eq!(cancelled.lifecycle, ConfigSelectionLifecycle::Cancelled);
    assert!(cancelled_preview.cancelled);
    assert_eq!(cancelled_preview.selected_for_review, None);
    assert_eq!(
        cancelled.write_target_path(),
        Some(&PathBuf::from("/tmp/root.conf"))
    );

    let no_config = ConfigSelectionState::no_detected_config();
    assert_eq!(
        no_config.preview().selection_source,
        ConfigSelectionSource::None
    );
    assert_eq!(no_config.write_target_path(), None);
}

#[test]
fn all_source_follow_choices_are_available() {
    let choices = [
        SourceFollowChoice::ReviewAllConnectedFiles,
        SourceFollowChoice::OnlySelectedFile,
        SourceFollowChoice::Cancel,
    ];

    assert_eq!(choices.len(), 3);
}
