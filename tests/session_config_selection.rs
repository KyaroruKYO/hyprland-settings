use std::path::PathBuf;

use hyprland_settings::config_selection::{
    ConfigSelectionLifecycle, ConfigSelectionState, SourceFollowChoice,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn session_selected_config_state_can_be_represented_without_persistence() {
    let state = ConfigSelectionState::auto_detected("/tmp/active.conf")
        .preview_manual_config(
            "/tmp/session.conf",
            SourceFollowChoice::ReviewAllConnectedFiles,
        )
        .use_preview_for_session_read_only();
    let preview = state.preview();

    assert_eq!(state.lifecycle, ConfigSelectionLifecycle::SessionReadOnly);
    assert_eq!(
        preview.session_read_only_config,
        Some(PathBuf::from("/tmp/session.conf"))
    );
    assert!(preview.session_only);
    assert!(preview.preview_only);
    assert_eq!(
        state.write_target_path(),
        Some(&PathBuf::from("/tmp/active.conf"))
    );
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

#[test]
fn session_selected_config_is_cleared_by_cancel_and_never_becomes_write_target() {
    let state = ConfigSelectionState::auto_detected("/tmp/active.conf")
        .preview_manual_config("/tmp/session.conf", SourceFollowChoice::OnlySelectedFile)
        .use_preview_for_session_read_only()
        .cancel_preview();

    let preview = state.preview();
    assert_eq!(preview.session_read_only_config, None);
    assert_eq!(preview.selected_for_review, None);
    assert_eq!(
        state.write_target_path(),
        Some(&PathBuf::from("/tmp/active.conf"))
    );
}
