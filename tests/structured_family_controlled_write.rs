use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use hyprland_settings::config_parser::parse_hyprland_config_file;
use hyprland_settings::current_config::CurrentConfigSnapshot;
use hyprland_settings::structured_family::{
    accept_structured_family_draft_rendered_record_confirmation,
    prove_structured_family_draft_rendered_record_render_reread,
    reject_structured_family_draft_rendered_record_confirmation,
    structured_family_draft_rendered_record_approval_draft,
    structured_family_draft_rendered_record_confirmation_request,
    structured_family_draft_rendered_record_diff_review_summary,
    structured_family_draft_rendered_record_plans,
    structured_family_draft_rendered_record_staged_apply_plan, structured_family_record_drafts,
    structured_family_record_editor_forms, StructuredFamilyDraftRenderedRecordStagedApplyPlan,
    StructuredFamilyKind,
};
use hyprland_settings::structured_family_controlled_write::{
    approve_structured_family_controlled_write, build_structured_family_controlled_write_plan,
    execute_structured_family_controlled_write,
    execute_structured_family_controlled_write_round_trip,
    restore_structured_family_controlled_write, structured_family_controlled_write_audit_record,
    verify_structured_family_controlled_write_approval, StructuredFamilyControlledWriteError,
};
use hyprland_settings::structured_family_write_target::{
    classify_structured_family_write_target, StructuredFamilyControlledWriteTarget,
    StructuredFamilyControlledWriteTargetKind, StructuredFamilyControlledWriteTargetRejection,
};

const FIXTURE_DIR: &str = "tests/fixtures/structured_families";

fn fixture_path(family: StructuredFamilyKind) -> PathBuf {
    let name = match family {
        StructuredFamilyKind::Monitor => "hl_monitor.conf",
        StructuredFamilyKind::Bind => "hl_bind.conf",
        StructuredFamilyKind::Animation => "hl_animation.conf",
        StructuredFamilyKind::Curve => "hl_curve.conf",
        StructuredFamilyKind::Gesture => "hl_gesture.conf",
        StructuredFamilyKind::Device => "hl_device.conf",
        StructuredFamilyKind::Permission => "hl_permission.conf",
    };
    Path::new(FIXTURE_DIR).join(name)
}

fn unique_temp_root(label: &str) -> PathBuf {
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock should be after the epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-controlled-write-{label}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("temp root should be creatable");
    root
}

fn copy_fixture_to_root(family: StructuredFamilyKind, root: &Path) -> PathBuf {
    let target = root.join("copied-hyprland.conf");
    fs::copy(fixture_path(family), &target).expect("fixture should copy into controlled root");
    target
}

fn family_records_on_disk(path: &Path, family: StructuredFamilyKind) -> Vec<String> {
    let snapshot = CurrentConfigSnapshot::from_parsed(
        parse_hyprland_config_file(path).expect("target should parse"),
    );
    snapshot
        .structured_family_projections()
        .into_iter()
        .find(|projection| projection.family == family)
        .expect("projection should exist")
        .records
        .iter()
        .map(|record| record.raw_line.clone())
        .collect()
}

fn accepted_staged_apply_plan(
    family: StructuredFamilyKind,
    scratch_root: &Path,
) -> StructuredFamilyDraftRenderedRecordStagedApplyPlan {
    let snapshot = CurrentConfigSnapshot::from_parsed(
        parse_hyprland_config_file(fixture_path(family)).expect("fixture should parse"),
    );
    let projection = snapshot
        .structured_family_projections()
        .into_iter()
        .find(|projection| projection.family == family)
        .expect("projection should exist");
    let forms = structured_family_record_editor_forms(&projection);
    let drafts = structured_family_record_drafts(&forms);
    let plans = structured_family_draft_rendered_record_plans(&drafts);
    let proof_path = scratch_root.join(format!("{}-render-reread.conf", family.family_id()));
    let proof = prove_structured_family_draft_rendered_record_render_reread(&plans, &proof_path);
    let summary = structured_family_draft_rendered_record_diff_review_summary(&plans, &proof);
    let approval = structured_family_draft_rendered_record_approval_draft(&summary);
    let request = structured_family_draft_rendered_record_confirmation_request(&approval);
    let accepted = accept_structured_family_draft_rendered_record_confirmation(&approval, &request);
    structured_family_draft_rendered_record_staged_apply_plan(&accepted, &summary)
}

fn temp_target(root: &Path, target: &Path) -> StructuredFamilyControlledWriteTarget {
    StructuredFamilyControlledWriteTarget::new(
        StructuredFamilyControlledWriteTargetKind::TemporaryConfigTarget,
        target,
        root,
    )
}

#[test]
fn controlled_executor_round_trips_every_family_on_copied_temp_targets() {
    for family in StructuredFamilyKind::ALL {
        let root = unique_temp_root("round-trip");
        let target_path = copy_fixture_to_root(family, &root);
        let original_bytes = fs::read(&target_path).expect("target should read");
        let original_records = family_records_on_disk(&target_path, family);
        assert!(
            !original_records.is_empty(),
            "{} fixture should contain records",
            family.family_id()
        );

        // Intended change: keep every record and append a duplicate of the
        // first record, so the written family section provably differs.
        let mut rendered_records = original_records.clone();
        rendered_records.push(original_records[0].clone());

        let staged_apply = accepted_staged_apply_plan(family, &root);
        let plan = build_structured_family_controlled_write_plan(
            &staged_apply,
            temp_target(&root, &target_path),
            rendered_records.clone(),
        )
        .expect("controlled write plan should build from accepted staged apply plan");
        let approval = approve_structured_family_controlled_write();

        let receipt = execute_structured_family_controlled_write(&plan, &approval)
            .expect("controlled write should execute against the copied temp target");

        assert!(receipt.executed);
        assert!(receipt.written);
        assert_eq!(receipt.family, family);
        assert_eq!(
            receipt.target_kind,
            StructuredFamilyControlledWriteTargetKind::TemporaryConfigTarget
        );
        assert!(receipt.backup.created);
        assert!(receipt.backup.byte_exact);
        assert!(receipt.backup.backup_path.starts_with(&root));
        assert_eq!(
            fs::read(&receipt.backup.backup_path).expect("backup should read"),
            original_bytes,
            "backup must preserve the original bytes"
        );
        assert!(receipt.post_write_verification.reread_completed);
        assert!(receipt.post_write_verification.intended_records_present);
        assert_eq!(
            receipt.post_write_verification.reread_record_count,
            rendered_records.len()
        );
        assert!(!receipt.active_real_config_touched);
        assert!(!receipt.hyprctl_reload_run);
        assert!(!receipt.runtime_mutated);

        let written_records = family_records_on_disk(&target_path, family);
        assert_eq!(
            written_records.len(),
            rendered_records.len(),
            "{} written target should contain the intended records",
            family.family_id()
        );
        assert_ne!(
            fs::read(&target_path).expect("target should read"),
            original_bytes,
            "write must actually change the controlled target"
        );

        let rollback = restore_structured_family_controlled_write(&plan, &receipt)
            .expect("controlled restore should execute");
        assert!(rollback.restore_executed);
        assert!(rollback.restored_byte_exact);
        assert!(rollback.post_restore_verification.reread_completed);
        assert!(rollback.post_restore_verification.intended_records_present);
        assert_eq!(
            fs::read(&target_path).expect("target should read"),
            original_bytes,
            "restore must return the controlled target to its original bytes"
        );
        assert_eq!(
            family_records_on_disk(&target_path, family),
            original_records
        );

        let audit = structured_family_controlled_write_audit_record(&receipt);
        assert!(audit.executed);
        assert!(audit.backup_created);
        assert!(audit.post_write_verified);
        assert!(!audit.active_real_config_touched);
        assert!(!audit.hyprctl_reload_run);
        assert!(!audit.runtime_mutated);
        assert!(audit.summary.contains(family.family_id()));

        fs::remove_dir_all(&root).expect("temp root should clean up");
    }
}

#[test]
fn controlled_executor_round_trip_helper_preserves_non_family_lines_and_emits_full_audit() {
    let family = StructuredFamilyKind::Monitor;
    let root = unique_temp_root("full-round-trip");
    let target_path = copy_fixture_to_root(family, &root);
    let original_bytes = fs::read(&target_path).expect("target should read");
    let original_records = family_records_on_disk(&target_path, family);

    let mut rendered_records = original_records.clone();
    rendered_records[0] = "monitor = eDP-1, 1920x1080@75, 0x0, 1".to_string();

    let staged_apply = accepted_staged_apply_plan(family, &root);
    let plan = build_structured_family_controlled_write_plan(
        &staged_apply,
        temp_target(&root, &target_path),
        rendered_records.clone(),
    )
    .expect("plan should build");
    let approval = approve_structured_family_controlled_write();

    let receipt = execute_structured_family_controlled_write(&plan, &approval)
        .expect("controlled write should execute");
    let written_text = fs::read_to_string(&target_path).expect("target should read");
    assert!(
        written_text.contains("# monitor fixture"),
        "non-family comment lines must be preserved by the write"
    );
    assert!(written_text.contains("monitor = eDP-1, 1920x1080@75, 0x0, 1"));
    assert!(!written_text.contains("monitor = eDP-1, 1920x1080@60, 0x0, 1"));
    let rollback = restore_structured_family_controlled_write(&plan, &receipt)
        .expect("restore should execute");
    assert!(rollback.restore_executed);
    assert_eq!(
        fs::read(&target_path).expect("target should read"),
        original_bytes
    );

    // The single-call round trip proves the same chain and carries rollback
    // proof inside the receipt.
    let round_trip_receipt =
        execute_structured_family_controlled_write_round_trip(&plan, &approval)
            .expect("round trip should execute");
    assert!(round_trip_receipt.executed);
    let rollback = round_trip_receipt
        .rollback
        .as_ref()
        .expect("round trip receipt should carry rollback proof");
    assert!(rollback.restore_executed);
    assert!(rollback.restored_byte_exact);
    assert!(rollback.post_restore_verification.intended_records_present);
    assert_eq!(
        fs::read(&target_path).expect("target should read"),
        original_bytes
    );

    let audit = structured_family_controlled_write_audit_record(&round_trip_receipt);
    assert!(audit.restore_executed);
    assert!(audit.post_restore_verified);

    fs::remove_dir_all(&root).expect("temp root should clean up");
}

#[test]
fn controlled_executor_refuses_active_real_hyprland_config() {
    let family = StructuredFamilyKind::Monitor;
    let root = unique_temp_root("active-refusal");
    let home = std::env::var("HOME").expect("HOME should be set");
    let active_config = Path::new(&home).join(".config/hypr/hyprland.conf");

    // Even when the declared kind lies about being temporary, the classifier
    // resolves the active real config and rejects it.
    let target = StructuredFamilyControlledWriteTarget::new(
        StructuredFamilyControlledWriteTargetKind::TemporaryConfigTarget,
        &active_config,
        Path::new(&home).join(".config/hypr"),
    );
    let policy = classify_structured_family_write_target(&target);
    assert_eq!(
        policy.resolved_kind,
        StructuredFamilyControlledWriteTargetKind::ActiveRealConfigTarget
    );
    assert!(!policy.writable);
    assert!(!policy.active_real_config_writable);
    assert!(policy
        .rejection_reasons
        .contains(&StructuredFamilyControlledWriteTargetRejection::ActiveRealConfigTargetRejected));

    let staged_apply = accepted_staged_apply_plan(family, &root);
    let plan = build_structured_family_controlled_write_plan(
        &staged_apply,
        target,
        vec!["monitor = eDP-1, 1920x1080@60, 0x0, 1".to_string()],
    )
    .expect("plan builds; target policy is enforced at execution time");
    let error = execute_structured_family_controlled_write(
        &plan,
        &approve_structured_family_controlled_write(),
    )
    .expect_err("active real config must be rejected");
    assert_eq!(
        error,
        StructuredFamilyControlledWriteError::TargetRejected(
            StructuredFamilyControlledWriteTargetRejection::ActiveRealConfigTargetRejected
        )
    );

    // A target under a controlled temp root that points at the active config
    // is also rejected: the active-config check wins over the root check.
    let sneaky = StructuredFamilyControlledWriteTarget::new(
        StructuredFamilyControlledWriteTargetKind::TemporaryConfigTarget,
        &active_config,
        &root,
    );
    let sneaky_policy = classify_structured_family_write_target(&sneaky);
    assert!(!sneaky_policy.writable);
    assert!(sneaky_policy
        .rejection_reasons
        .contains(&StructuredFamilyControlledWriteTargetRejection::ActiveRealConfigTargetRejected));

    fs::remove_dir_all(&root).expect("temp root should clean up");
}

#[test]
fn controlled_executor_refuses_unknown_targets_path_escapes_and_symlink_escapes() {
    let family = StructuredFamilyKind::Monitor;
    let root = unique_temp_root("escape-refusal");
    let target_path = copy_fixture_to_root(family, &root);
    let staged_apply = accepted_staged_apply_plan(family, &root);
    let approval = approve_structured_family_controlled_write();
    let rendered = vec!["monitor = eDP-1, 1920x1080@60, 0x0, 1".to_string()];

    // Unknown declared kind is never writable.
    let unknown = StructuredFamilyControlledWriteTarget::new(
        StructuredFamilyControlledWriteTargetKind::UnknownTarget,
        &target_path,
        &root,
    );
    let unknown_policy = classify_structured_family_write_target(&unknown);
    assert!(!unknown_policy.writable);
    assert!(unknown_policy
        .rejection_reasons
        .contains(&StructuredFamilyControlledWriteTargetRejection::UnknownTargetRejected));
    let plan =
        build_structured_family_controlled_write_plan(&staged_apply, unknown, rendered.clone())
            .expect("plan builds; policy is enforced at execution time");
    let error = execute_structured_family_controlled_write(&plan, &approval)
        .expect_err("unknown target must be rejected");
    assert!(matches!(
        error,
        StructuredFamilyControlledWriteError::TargetRejected(_)
    ));

    // Declared active kind is never writable either.
    let declared_active = StructuredFamilyControlledWriteTarget::new(
        StructuredFamilyControlledWriteTargetKind::ActiveRealConfigTarget,
        &target_path,
        &root,
    );
    assert!(!classify_structured_family_write_target(&declared_active).writable);

    // Parent traversal escapes are rejected.
    let escape = StructuredFamilyControlledWriteTarget::new(
        StructuredFamilyControlledWriteTargetKind::TemporaryConfigTarget,
        root.join("../escaped-target.conf"),
        &root,
    );
    let escape_policy = classify_structured_family_write_target(&escape);
    assert!(!escape_policy.writable);
    assert!(escape_policy
        .rejection_reasons
        .contains(&StructuredFamilyControlledWriteTargetRejection::PathEscapeRejected));

    // Targets outside the controlled root are rejected.
    let outside_root = unique_temp_root("outside-root");
    let outside = StructuredFamilyControlledWriteTarget::new(
        StructuredFamilyControlledWriteTargetKind::TemporaryConfigTarget,
        outside_root.join("outside.conf"),
        &root,
    );
    let outside_policy = classify_structured_family_write_target(&outside);
    assert!(!outside_policy.writable);
    assert!(outside_policy
        .rejection_reasons
        .contains(&StructuredFamilyControlledWriteTargetRejection::TargetOutsideControlledRoot));

    // Symlinked parents that resolve outside the controlled root are rejected.
    let escape_destination = unique_temp_root("symlink-destination");
    let link_dir = root.join("linked");
    std::os::unix::fs::symlink(&escape_destination, &link_dir)
        .expect("symlink should be creatable");
    let symlinked = StructuredFamilyControlledWriteTarget::new(
        StructuredFamilyControlledWriteTargetKind::TemporaryConfigTarget,
        link_dir.join("target.conf"),
        &root,
    );
    let symlink_policy = classify_structured_family_write_target(&symlinked);
    assert!(!symlink_policy.writable);
    assert!(symlink_policy
        .rejection_reasons
        .contains(&StructuredFamilyControlledWriteTargetRejection::SymlinkEscapeRejected));
    let plan = build_structured_family_controlled_write_plan(&staged_apply, symlinked, rendered)
        .expect("plan builds; policy is enforced at execution time");
    let error = execute_structured_family_controlled_write(&plan, &approval)
        .expect_err("symlink escape must be rejected");
    assert!(matches!(
        error,
        StructuredFamilyControlledWriteError::TargetRejected(
            StructuredFamilyControlledWriteTargetRejection::SymlinkEscapeRejected
        )
    ));

    // Disallowed controlled roots are rejected.
    let disallowed = StructuredFamilyControlledWriteTarget::new(
        StructuredFamilyControlledWriteTargetKind::TemporaryConfigTarget,
        PathBuf::from("/var/lib/hyprland-settings-not-a-test-root/target.conf"),
        PathBuf::from("/var/lib/hyprland-settings-not-a-test-root"),
    );
    let disallowed_policy = classify_structured_family_write_target(&disallowed);
    assert!(!disallowed_policy.writable);
    assert!(disallowed_policy
        .rejection_reasons
        .contains(&StructuredFamilyControlledWriteTargetRejection::ControlledRootNotAllowed));

    fs::remove_dir_all(&root).ok();
    fs::remove_dir_all(&outside_root).ok();
    fs::remove_dir_all(&escape_destination).ok();
}

#[test]
fn controlled_executor_fails_closed_on_missing_approval_and_missing_plans() {
    let family = StructuredFamilyKind::Monitor;
    let root = unique_temp_root("fail-closed");
    let target_path = copy_fixture_to_root(family, &root);
    let original_bytes = fs::read(&target_path).expect("target should read");
    let staged_apply = accepted_staged_apply_plan(family, &root);
    let rendered = vec!["monitor = eDP-1, 1920x1080@60, 0x0, 1".to_string()];
    let plan = build_structured_family_controlled_write_plan(
        &staged_apply,
        temp_target(&root, &target_path),
        rendered,
    )
    .expect("plan should build");

    // Missing approval.
    let mut unapproved = approve_structured_family_controlled_write();
    unapproved.controlled_write_approved = false;
    assert_eq!(
        execute_structured_family_controlled_write(&plan, &unapproved),
        Err(StructuredFamilyControlledWriteError::MissingApproval)
    );

    // Attempting to approve the active real config is itself an error.
    let mut forbidden = approve_structured_family_controlled_write();
    forbidden.active_real_config_write_approved = true;
    assert_eq!(
        verify_structured_family_controlled_write_approval(&forbidden),
        Err(StructuredFamilyControlledWriteError::ActiveRealConfigApprovalForbidden)
    );
    assert_eq!(
        execute_structured_family_controlled_write(&plan, &forbidden),
        Err(StructuredFamilyControlledWriteError::ActiveRealConfigApprovalForbidden)
    );

    // Missing backup, restore, and verification plans each fail closed.
    let approval = approve_structured_family_controlled_write();
    let mut missing_backup = plan.clone();
    missing_backup.backup_plan = None;
    assert_eq!(
        execute_structured_family_controlled_write(&missing_backup, &approval),
        Err(StructuredFamilyControlledWriteError::MissingBackupPlan)
    );
    let mut missing_restore = plan.clone();
    missing_restore.restore_plan = None;
    assert_eq!(
        execute_structured_family_controlled_write(&missing_restore, &approval),
        Err(StructuredFamilyControlledWriteError::MissingRestorePlan)
    );
    let mut missing_verification = plan.clone();
    missing_verification.verification_plan = None;
    assert_eq!(
        execute_structured_family_controlled_write(&missing_verification, &approval),
        Err(StructuredFamilyControlledWriteError::MissingVerificationPlan)
    );

    // Backup paths outside the controlled root fail closed.
    let mut foreign_backup = plan.clone();
    if let Some(backup_plan) = foreign_backup.backup_plan.as_mut() {
        backup_plan.backup_path = std::env::temp_dir().join("outside-root-backup.conf");
    }
    assert_eq!(
        execute_structured_family_controlled_write(&foreign_backup, &approval),
        Err(StructuredFamilyControlledWriteError::BackupPathOutsideControlledRoot)
    );

    // Nothing above may have touched the target.
    assert_eq!(
        fs::read(&target_path).expect("target should read"),
        original_bytes
    );

    fs::remove_dir_all(&root).expect("temp root should clean up");
}

#[test]
fn controlled_executor_fails_closed_on_unsafe_staged_apply_plans() {
    let family = StructuredFamilyKind::Monitor;
    let root = unique_temp_root("unsafe-plan");
    let target_path = copy_fixture_to_root(family, &root);
    let rendered = vec!["monitor = eDP-1, 1920x1080@60, 0x0, 1".to_string()];

    // A rejected confirmation produces a blocked staged apply plan, which the
    // plan builder refuses.
    let snapshot = CurrentConfigSnapshot::from_parsed(
        parse_hyprland_config_file(fixture_path(family)).expect("fixture should parse"),
    );
    let projection = snapshot
        .structured_family_projections()
        .into_iter()
        .find(|projection| projection.family == family)
        .expect("projection should exist");
    let forms = structured_family_record_editor_forms(&projection);
    let drafts = structured_family_record_drafts(&forms);
    let plans = structured_family_draft_rendered_record_plans(&drafts);
    let proof_path = root.join("unsafe-render-reread.conf");
    let proof = prove_structured_family_draft_rendered_record_render_reread(&plans, &proof_path);
    let summary = structured_family_draft_rendered_record_diff_review_summary(&plans, &proof);
    let approval_draft = structured_family_draft_rendered_record_approval_draft(&summary);
    let request = structured_family_draft_rendered_record_confirmation_request(&approval_draft);
    let rejected =
        reject_structured_family_draft_rendered_record_confirmation(&approval_draft, &request);
    let blocked_staged_apply =
        structured_family_draft_rendered_record_staged_apply_plan(&rejected, &summary);
    assert!(!blocked_staged_apply.blockers.is_empty());
    let error = build_structured_family_controlled_write_plan(
        &blocked_staged_apply,
        temp_target(&root, &target_path),
        rendered.clone(),
    )
    .expect_err("blocked staged apply plan must be refused");
    assert!(matches!(
        error,
        StructuredFamilyControlledWriteError::UnsafeStagedApplyPlan(_)
    ));

    // Empty rendered records are refused.
    let accepted = accepted_staged_apply_plan(family, &root);
    assert_eq!(
        build_structured_family_controlled_write_plan(
            &accepted,
            temp_target(&root, &target_path),
            Vec::new(),
        ),
        Err(StructuredFamilyControlledWriteError::EmptyRenderedRecords)
    );

    // A plan whose staged apply linkage was tampered away fails closed.
    let mut tampered = build_structured_family_controlled_write_plan(
        &accepted,
        temp_target(&root, &target_path),
        rendered,
    )
    .expect("plan should build");
    tampered.staged_apply_linked = false;
    assert!(matches!(
        execute_structured_family_controlled_write(
            &tampered,
            &approve_structured_family_controlled_write()
        ),
        Err(StructuredFamilyControlledWriteError::UnsafeStagedApplyPlan(
            _
        ))
    ));

    fs::remove_dir_all(&root).expect("temp root should clean up");
}

#[test]
fn controlled_executor_restores_original_bytes_when_post_write_verification_fails() {
    let family = StructuredFamilyKind::Monitor;
    let root = unique_temp_root("verification-failure");
    let target_path = copy_fixture_to_root(family, &root);
    let original_bytes = fs::read(&target_path).expect("target should read");
    let staged_apply = accepted_staged_apply_plan(family, &root);

    // A rendered record that does not parse as a family record cannot pass
    // post-write reread verification; the executor must restore the target.
    let plan = build_structured_family_controlled_write_plan(
        &staged_apply,
        temp_target(&root, &target_path),
        vec!["this line does not belong to any structured family".to_string()],
    )
    .expect("plan should build");
    let error = execute_structured_family_controlled_write(
        &plan,
        &approve_structured_family_controlled_write(),
    )
    .expect_err("verification must fail");
    assert_eq!(
        error,
        StructuredFamilyControlledWriteError::PostWriteVerificationFailed
    );
    assert_eq!(
        fs::read(&target_path).expect("target should read"),
        original_bytes,
        "failed verification must leave the original bytes restored"
    );

    fs::remove_dir_all(&root).expect("temp root should clean up");
}

#[test]
fn controlled_write_sources_have_no_reload_runtime_or_ui_paths() {
    for module in [
        "src/structured_family_write_target.rs",
        "src/structured_family_controlled_write.rs",
    ] {
        let source = fs::read_to_string(module).expect("module source should read");
        for forbidden in [
            "hyprctl reload",
            "hyprctl ",
            "\"hyprctl\"",
            "Command::",
            "std::process",
            "process::Command",
            "spawn(",
            "gtk::",
            "gtk4::",
            "adw::",
            "Button::",
            "connect_clicked",
            "/home/kyo/.config",
            "hyprland.conf\"",
            "write_flow::",
            "crate::write_flow",
            "apply_setting_change(",
            "execute_safe_write_scaffold",
        ] {
            assert!(
                !source.contains(forbidden),
                "{module} must not contain {forbidden}"
            );
        }
    }
}

#[test]
fn controlled_executor_is_unreachable_from_live_ui_and_scalar_write_paths() {
    for path in [
        "src/main.rs",
        "src/write_flow.rs",
        "src/ui/mod.rs",
        "src/ui/app.rs",
        "src/ui/model.rs",
        "src/ui/window.rs",
    ] {
        let source = fs::read_to_string(path).expect("source should read");
        for forbidden in [
            "structured_family_controlled_write",
            "structured_family_write_target",
            "execute_structured_family_controlled_write",
            "restore_structured_family_controlled_write",
        ] {
            assert!(
                !source.contains(forbidden),
                "{path} must not reference {forbidden}: the controlled executor has no live UI or scalar write-flow reachability"
            );
        }
    }
}

#[test]
fn controlled_write_default_state_cannot_target_active_config() {
    // The default approval cannot approve the active config.
    let approval = approve_structured_family_controlled_write();
    assert!(approval.controlled_write_approved);
    assert!(!approval.active_real_config_write_approved);
    assert!(verify_structured_family_controlled_write_approval(&approval).is_ok());

    // No target kind resolving to the active config is ever writable, and the
    // policy's active-config switch is a constant false.
    let home = std::env::var("HOME").expect("HOME should be set");
    for declared in [
        StructuredFamilyControlledWriteTargetKind::TestOwnedFixtureTarget,
        StructuredFamilyControlledWriteTargetKind::CopiedConfigTreeTarget,
        StructuredFamilyControlledWriteTargetKind::TemporaryConfigTarget,
        StructuredFamilyControlledWriteTargetKind::ActiveRealConfigTarget,
        StructuredFamilyControlledWriteTargetKind::UnknownTarget,
    ] {
        let target = StructuredFamilyControlledWriteTarget::new(
            declared,
            Path::new(&home).join(".config/hypr/hyprland.conf"),
            Path::new(&home).join(".config/hypr"),
        );
        let policy = classify_structured_family_write_target(&target);
        assert!(
            !policy.writable,
            "{} must not be writable",
            declared.as_str()
        );
        assert!(!policy.active_real_config_writable);
        assert_eq!(
            policy.resolved_kind,
            StructuredFamilyControlledWriteTargetKind::ActiveRealConfigTarget
        );
    }
}

#[test]
fn controlled_real_write_implementation_report_preserves_active_config_boundaries() {
    let report: serde_json::Value = serde_json::from_str(
        &fs::read_to_string(
            "data/reports/structured-family-controlled-real-write-implementation.v0.55.2.json",
        )
        .expect("controlled real-write report should read"),
    )
    .expect("controlled real-write report should be valid JSON");

    assert_eq!(
        report["artifactKind"],
        "structured-family-controlled-real-write-implementation"
    );
    for key in [
        "controlledRealWriteImplementationApproved",
        "actualExecutorWiringScaffoldApproved",
        "internalExecutorWiringApproved",
        "controlledTargetWritesApproved",
        "executorWiredForControlledTargets",
        "realWritePathEnabledForControlledTargets",
        "backupCreationEnabledForControlledTargets",
        "restoreExecutionEnabledForControlledTargets",
        "rollbackExecutionEnabledForControlledTargets",
    ] {
        assert_eq!(report[key], true, "{key} should be true");
    }
    for key in [
        "activeRealConfigWritesApproved",
        "firstRealConfigWriteApproved",
        "executorWiredForActiveConfig",
        "realWritePathEnabledForActiveConfig",
        "backupCreationEnabledForActiveConfig",
        "restoreExecutionEnabledForActiveConfig",
        "rollbackExecutionEnabledForActiveConfig",
        "hyprctlReloadEnabled",
        "runtimeMutationEnabled",
        "guiLiveApplyControlsEnabled",
    ] {
        assert_eq!(report[key], false, "{key} should remain false");
    }
    assert_eq!(
        report["productionReadinessDecision"],
        "not production ready for active real config writes"
    );
    for key in [
        "targetPolicies",
        "writeExecutorBehavior",
        "backupRestoreProof",
        "rollbackRecoveryProof",
        "rereadVerificationProof",
        "failureModeProof",
        "sourceGuards",
        "testsAdded",
        "filesChanged",
    ] {
        assert!(
            !report[key]
                .as_array()
                .expect("report field should be array")
                .is_empty(),
            "{key} should be populated"
        );
    }
    let next = report["nextRecommendedWork"]
        .as_str()
        .expect("nextRecommendedWork should be text");
    assert!(next.contains("Stop for explicit user decision"));
}
