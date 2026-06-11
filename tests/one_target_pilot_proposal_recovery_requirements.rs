use hyprland_settings::one_target_pilot_gate_flip_proposal_review::one_target_pilot_backup_verification_recovery_review;
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

#[test]
fn proposal_requires_backup_verification_recovery_and_no_runtime_reload() {
    let review = one_target_pilot_backup_verification_recovery_review();

    assert!(review.exact_backup_before_write_required);
    assert!(review.backup_byte_equality_proof_required);
    assert!(review.write_only_after_backup_proof_required);
    assert!(review.reread_target_after_write_required);
    assert!(review.expected_value_verification_required);
    assert!(review.restore_backup_on_write_failure_after_backup_required);
    assert!(review.restore_backup_on_verification_failure_required);
    assert!(review.reread_restored_file_required);
    assert!(review.restored_bytes_and_value_verification_required);
    assert!(review.report_recovery_failure_without_hiding_backup_required);
    assert!(!review.automatic_hyprland_reload_allowed);
    assert!(!review.mutating_hyprctl_allowed);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}
