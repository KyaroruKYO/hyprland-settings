use std::fs;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use hyprland_settings::future_capability::{
    approval_decision_for_gate, assess_hyprland_version_migration,
    copied_config_tree_files_restored, copied_config_tree_originals_unchanged,
    copied_config_tree_report, copy_config_tree_for_proof, current_v0552_data_bundle,
    disabled_future_approval_card_projections, disabled_migration_review,
    disabled_missing_default_insertion_review, disabled_profile_switch_review,
    disabled_profile_switch_selection_review, duplicate_approval_flow,
    duplicate_occurrence_confirmation, duplicate_occurrence_model, duplicate_occurrence_review,
    duplicate_production_approval_gate, duplicate_production_gate_review,
    edit_structured_bind_safe_env, execute_duplicate_replacement_guarded_temp,
    execute_source_include_selected_target_guarded_temp, execute_structured_bind_guarded_temp,
    high_risk_approval_flow, high_risk_guarded_live_readiness_executor,
    high_risk_live_recovery_protocol, high_risk_production_gate_review, high_risk_recovery_review,
    high_risk_recovery_workflow, hyprland_0554_approval_flow, hyprland_version_activation_gate,
    local_hyprland_version_evidence, migration_comparison_review, profile_approval_flow,
    profile_production_gate_review, profile_target_approval_review,
    proven_runtime_approval_evidence_summary, render_structured_entry_lossless,
    replace_duplicate_occurrence_safe_env, replace_duplicate_occurrence_with_confirmation_safe_env,
    runtime_action_policy, runtime_action_review, runtime_approval_flow, runtime_command_risk,
    runtime_eval_syntax_evidence, runtime_guarded_executor, runtime_live_restore_approval_review,
    runtime_live_restore_attempt_review, runtime_live_restore_proof_review,
    runtime_production_gate_review, runtime_socket_diagnosis, source_include_approval_flow,
    source_include_insertion_review, source_include_production_gate_review,
    source_include_selected_target_dry_run_plan, source_include_target_selection_fixture_proof,
    structured_approval_flow, structured_family_model, structured_family_review,
    structured_production_gate_review, switch_profile_symlink_guarded_temp,
    switch_profile_symlink_safe_env, trusted_export_requirement,
    validate_structured_edit_candidate, ApprovalEvidence, ApprovalRequest, ApprovalScope,
    ApprovalStatus, ApprovalToken, ControlledLiveTestGuardRequest, ControlledLiveTestKind,
    DuplicateOccurrenceApprovalState, DuplicateOccurrenceReviewState,
    DuplicateProductionGateStatus, DuplicateReplacementOptions, DuplicateReplacementRequest,
    DuplicateReplacementStatus, GuardedTempExecutionStatus, HighRiskLiveReadinessStatus,
    HighRiskProductionGateStatus, HyprlandVersionActivationStatus, MockWatchdog, MockWatchdogState,
    ProfileProductionGateStatus, ProfileSwitchStatus, ProfileTargetReadiness, RuntimeAction,
    RuntimeApprovalReviewStatus, RuntimeCommandRisk, RuntimeDirectIpcReadOnlyEvidence,
    RuntimeDryRunExecutor, RuntimeEvalSyntaxEvidence, RuntimeLiveRestoreProof,
    RuntimeLiveRestoreStatus, RuntimeMutationCommandPair, RuntimeMutationSyntaxCandidate,
    RuntimeMutationSyntaxStatus, RuntimeProductionGateStatus, RuntimeReadOnlyEvidence,
    RuntimeSocketCandidate, RuntimeSocketDiagnosisStatus, SourceIncludeInsertionReadiness,
    SourceIncludeProductionGateStatus, SourceIncludeSelectedTargetDryRunStatus,
    SourceIncludeTargetCandidate, SourceIncludeTargetSelectionStatus, StructuredBindEditStatus,
    StructuredProductionGateStatus,
};
use hyprland_settings::missing_default_insertion::{
    build_missing_default_insertion_plan, MissingDefaultInsertionRequest,
};

fn temp_root(label: &str) -> PathBuf {
    let stamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("time should work")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-future-capability-{label}-{}-{stamp}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("temp root should be created");
    root
}

fn write_file(path: &Path, contents: &str) {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).expect("parent should exist");
    }
    fs::write(path, contents).expect("fixture should write");
}

#[test]
fn missing_default_review_scaffold_stays_disabled_for_production() {
    let root = temp_root("missing-review");
    let config = root.join("hyprland.conf");
    write_file(&config, "decoration:blur:enabled = true\n");
    let plan = build_missing_default_insertion_plan(MissingDefaultInsertionRequest {
        setting_id: "misc.disable_splash_rendering".to_string(),
        proposed_value: "true".to_string(),
        target_path: config,
        backup_stamp: "fixture".to_string(),
    });
    let review = disabled_missing_default_insertion_review(&plan);

    assert!(!review.production_apply_enabled);
    assert!(review
        .user_copy
        .contains("limited to reviewed single-file safe-batch targets"));
    assert!(review
        .required_gates
        .iter()
        .any(|gate| gate.contains("production UI approval")));
}

#[test]
fn source_include_insertion_review_allows_only_single_root_and_blocks_target_selection() {
    let root = temp_root("source-insertion-review");
    let root_conf = root.join("hyprland.conf");
    let sourced_conf = root.join("appearance.conf");

    let single = source_include_insertion_review(
        &root_conf,
        vec![root_conf.clone()],
        Some(root_conf.clone()),
        false,
    );
    assert_eq!(
        single.readiness,
        SourceIncludeInsertionReadiness::SingleRootEligible
    );
    assert!(single.production_insertion_enabled);

    let sourced = source_include_insertion_review(
        &root_conf,
        vec![root_conf.clone(), sourced_conf],
        Some(root_conf.clone()),
        false,
    );
    assert_eq!(
        sourced.readiness,
        SourceIncludeInsertionReadiness::SourceIncludeTargetSelectionRequired
    );
    assert!(!sourced.production_insertion_enabled);

    let managed = source_include_insertion_review(&root_conf, vec![root_conf.clone()], None, true);
    assert_eq!(
        managed.readiness,
        SourceIncludeInsertionReadiness::ManagedTargetBlocked
    );
    assert!(!managed.production_insertion_enabled);
}

#[test]
fn source_include_target_selection_fixture_proof_requires_explicit_safe_target() {
    let root = temp_root("source-target-fixture-proof");
    let root_conf = root.join("hyprland.conf");
    let sourced_conf = root.join("appearance.conf");
    let generated_conf = root.join("generated.conf");
    let profile_conf = root.join("profiles/current.conf");
    let outside_conf = root.join("unknown.conf");
    let candidates = vec![
        SourceIncludeTargetCandidate {
            path: root_conf.clone(),
            source_depth: 0,
            generated_or_script_managed: false,
            symlink_or_profile_managed: false,
        },
        SourceIncludeTargetCandidate {
            path: sourced_conf.clone(),
            source_depth: 1,
            generated_or_script_managed: false,
            symlink_or_profile_managed: false,
        },
        SourceIncludeTargetCandidate {
            path: generated_conf.clone(),
            source_depth: 1,
            generated_or_script_managed: true,
            symlink_or_profile_managed: false,
        },
        SourceIncludeTargetCandidate {
            path: profile_conf.clone(),
            source_depth: 1,
            generated_or_script_managed: false,
            symlink_or_profile_managed: true,
        },
    ];

    let no_selection =
        source_include_target_selection_fixture_proof(&root_conf, candidates.clone(), None, false);
    assert_eq!(
        no_selection.status,
        SourceIncludeTargetSelectionStatus::NoTargetSelected
    );
    assert!(!no_selection.fixture_plan_allowed);
    assert!(!no_selection.production_insertion_enabled);

    let selected = source_include_target_selection_fixture_proof(
        &root_conf,
        candidates.clone(),
        Some(sourced_conf.clone()),
        false,
    );
    assert_eq!(
        selected.status,
        SourceIncludeTargetSelectionStatus::SelectedTargetReadyForFixture
    );
    assert!(selected.fixture_plan_allowed);
    assert!(!selected.production_insertion_enabled);
    assert!(!selected.real_config_touched);
    assert_eq!(
        selected
            .precondition
            .as_ref()
            .expect("precondition")
            .source_depth,
        1
    );

    let generated = source_include_target_selection_fixture_proof(
        &root_conf,
        candidates.clone(),
        Some(generated_conf),
        false,
    );
    assert_eq!(
        generated.status,
        SourceIncludeTargetSelectionStatus::ManagedTargetBlocked
    );
    assert!(!generated.fixture_plan_allowed);

    let profile = source_include_target_selection_fixture_proof(
        &root_conf,
        candidates.clone(),
        Some(profile_conf),
        false,
    );
    assert_eq!(
        profile.status,
        SourceIncludeTargetSelectionStatus::ManagedTargetBlocked
    );
    assert!(!profile.fixture_plan_allowed);

    let unknown = source_include_target_selection_fixture_proof(
        &root_conf,
        candidates,
        Some(outside_conf),
        false,
    );
    assert_eq!(
        unknown.status,
        SourceIncludeTargetSelectionStatus::TargetNotCandidate
    );
    assert!(!unknown.fixture_plan_allowed);
}

#[test]
fn source_include_selected_target_dry_run_plans_only_explicit_fixture_targets() {
    let root = temp_root("source-selected-dry-run");
    let root_conf = root.join("hyprland.conf");
    let sourced_conf = root.join("appearance.conf");
    let generated_conf = root.join("generated.conf");
    write_file(&root_conf, "source = appearance.conf\n");
    write_file(&sourced_conf, "decoration:blur:enabled = true\n");
    write_file(&generated_conf, "# generated by fixture\n");
    let candidates = vec![
        SourceIncludeTargetCandidate {
            path: root_conf.clone(),
            source_depth: 0,
            generated_or_script_managed: false,
            symlink_or_profile_managed: false,
        },
        SourceIncludeTargetCandidate {
            path: sourced_conf.clone(),
            source_depth: 1,
            generated_or_script_managed: false,
            symlink_or_profile_managed: false,
        },
        SourceIncludeTargetCandidate {
            path: generated_conf.clone(),
            source_depth: 1,
            generated_or_script_managed: true,
            symlink_or_profile_managed: false,
        },
    ];

    let selected_source = source_include_target_selection_fixture_proof(
        &root_conf,
        candidates.clone(),
        Some(sourced_conf.clone()),
        false,
    );
    let source_plan = build_missing_default_insertion_plan(MissingDefaultInsertionRequest {
        setting_id: "misc.disable_splash_rendering".to_string(),
        proposed_value: "true".to_string(),
        target_path: sourced_conf.clone(),
        backup_stamp: "source-dry-run".to_string(),
    });
    let source_dry_run =
        source_include_selected_target_dry_run_plan(&selected_source, &source_plan);
    assert_eq!(
        source_dry_run.status,
        SourceIncludeSelectedTargetDryRunStatus::Planned
    );
    assert_eq!(source_dry_run.selected_target.as_ref(), Some(&sourced_conf));
    assert_eq!(source_dry_run.source_depth, Some(1));
    assert_eq!(
        source_dry_run.insertion_line.as_deref(),
        Some("misc:disable_splash_rendering = true")
    );
    assert!(source_dry_run
        .dry_run_preview
        .as_ref()
        .expect("preview")
        .contains("Would insert"));
    assert!(!source_dry_run.production_insertion_enabled);
    assert!(!source_dry_run.real_config_touched);
    assert!(!source_dry_run.runtime_touched);

    let selected_root = source_include_target_selection_fixture_proof(
        &root_conf,
        candidates.clone(),
        Some(root_conf.clone()),
        false,
    );
    let root_plan = build_missing_default_insertion_plan(MissingDefaultInsertionRequest {
        setting_id: "misc.disable_hyprland_logo".to_string(),
        proposed_value: "true".to_string(),
        target_path: root_conf.clone(),
        backup_stamp: "root-dry-run".to_string(),
    });
    let root_dry_run = source_include_selected_target_dry_run_plan(&selected_root, &root_plan);
    assert_eq!(
        root_dry_run.status,
        SourceIncludeSelectedTargetDryRunStatus::Planned
    );
    assert_eq!(root_dry_run.selected_target.as_ref(), Some(&root_conf));
    assert_eq!(root_dry_run.source_depth, Some(0));

    let no_selection =
        source_include_target_selection_fixture_proof(&root_conf, candidates.clone(), None, false);
    let blocked_no_selection =
        source_include_selected_target_dry_run_plan(&no_selection, &source_plan);
    assert_eq!(
        blocked_no_selection.status,
        SourceIncludeSelectedTargetDryRunStatus::SelectionBlocked
    );

    let generated = source_include_target_selection_fixture_proof(
        &root_conf,
        candidates.clone(),
        Some(generated_conf),
        false,
    );
    let generated_blocked = source_include_selected_target_dry_run_plan(&generated, &source_plan);
    assert_eq!(
        generated_blocked.status,
        SourceIncludeSelectedTargetDryRunStatus::SelectionBlocked
    );

    let ambiguous = source_include_target_selection_fixture_proof(
        &root_conf,
        candidates.clone(),
        Some(sourced_conf.clone()),
        true,
    );
    let ambiguous_blocked = source_include_selected_target_dry_run_plan(&ambiguous, &source_plan);
    assert_eq!(
        ambiguous_blocked.status,
        SourceIncludeSelectedTargetDryRunStatus::SelectionBlocked
    );

    let mismatch = source_include_selected_target_dry_run_plan(&selected_source, &root_plan);
    assert_eq!(
        mismatch.status,
        SourceIncludeSelectedTargetDryRunStatus::TargetMismatch
    );
}

#[test]
fn source_include_selected_target_dry_run_refuses_non_fixture_and_blocked_insertions() {
    let root_conf = PathBuf::from("/home/kyo/Documents/non-fixture-hyprland.conf");
    let proof = source_include_target_selection_fixture_proof(
        &root_conf,
        vec![SourceIncludeTargetCandidate {
            path: root_conf.clone(),
            source_depth: 0,
            generated_or_script_managed: false,
            symlink_or_profile_managed: false,
        }],
        Some(root_conf.clone()),
        false,
    );
    let plan = build_missing_default_insertion_plan(MissingDefaultInsertionRequest {
        setting_id: "misc.disable_splash_rendering".to_string(),
        proposed_value: "true".to_string(),
        target_path: root_conf,
        backup_stamp: "non-fixture".to_string(),
    });
    let non_fixture = source_include_selected_target_dry_run_plan(&proof, &plan);
    assert_eq!(
        non_fixture.status,
        SourceIncludeSelectedTargetDryRunStatus::NonFixtureTargetRefused
    );
    assert!(!non_fixture.real_config_touched);

    let root = temp_root("source-selected-dry-run-blocked-plan");
    let config = root.join("hyprland.conf");
    write_file(&config, "misc:disable_splash_rendering = false\n");
    let proof = source_include_target_selection_fixture_proof(
        &config,
        vec![SourceIncludeTargetCandidate {
            path: config.clone(),
            source_depth: 0,
            generated_or_script_managed: false,
            symlink_or_profile_managed: false,
        }],
        Some(config.clone()),
        false,
    );
    let duplicate_plan = build_missing_default_insertion_plan(MissingDefaultInsertionRequest {
        setting_id: "misc.disable_splash_rendering".to_string(),
        proposed_value: "true".to_string(),
        target_path: config,
        backup_stamp: "blocked".to_string(),
    });
    let blocked_plan = source_include_selected_target_dry_run_plan(&proof, &duplicate_plan);
    assert_eq!(
        blocked_plan.status,
        SourceIncludeSelectedTargetDryRunStatus::InsertionPlanBlocked
    );
    assert!(blocked_plan
        .blocked_reasons
        .iter()
        .any(|reason| reason.contains("already configured")));
    assert!(!blocked_plan.production_insertion_enabled);
}

fn complete_live_guard_request(target: PathBuf) -> ControlledLiveTestGuardRequest {
    ControlledLiveTestGuardRequest {
        test_id: "20260619_000000_fixture".to_string(),
        target_paths: vec![target],
        backup_paths_recorded: true,
        original_hashes_recorded: true,
        symlink_targets_recorded: true,
        read_only_runtime_snapshot_recorded: true,
        restore_plan_recorded: true,
        post_restore_verification_planned: true,
        out_of_band_recovery_recorded: true,
        trusted_data_available: true,
        explicit_live_flag: true,
    }
}

fn approval_request(
    scope: ApprovalScope,
    target_path: Option<PathBuf>,
    runtime_command: Option<&str>,
    copied_proof: bool,
    live_restore_proof: bool,
) -> ApprovalRequest {
    ApprovalRequest {
        scope,
        evidence: ApprovalEvidence {
            target_path,
            runtime_command: runtime_command.map(ToOwned::to_owned),
            copied_config_tree_proof_restored: copied_proof,
            live_restore_proof_restored: live_restore_proof,
            old_state: Some("old".to_string()),
            proposed_new_state: Some("new".to_string()),
            restore_plan: Some("restore old state".to_string()),
        },
        token: ApprovalToken {
            token: "approve".to_string(),
            expires_at_tick: Some(100),
            one_shot: true,
            used: false,
        },
        provided_token: Some("approve".to_string()),
        current_tick: 1,
        rejected: false,
    }
}

fn runtime_readonly_evidence(
    succeeded: bool,
    raw_error_text: Option<&str>,
) -> RuntimeReadOnlyEvidence {
    RuntimeReadOnlyEvidence {
        hyprctl_binary_path: Some(PathBuf::from("/usr/bin/hyprctl")),
        instance_signature: Some(
            "a0136d8c04687bb36eb8a28eb9d1ff92aea99704_1781857006_1638495299".to_string(),
        ),
        xdg_runtime_dir: Some(PathBuf::from("/run/user/1000")),
        version_succeeded: succeeded,
        monitors_json_succeeded: succeeded,
        gaps_in_succeeded: succeeded,
        gaps_out_succeeded: succeeded,
        blur_enabled_succeeded: succeeded,
        logo_disabled_succeeded: succeeded,
        raw_error_text: raw_error_text.map(ToOwned::to_owned),
    }
}

fn proven_runtime_eval_syntax_evidence() -> RuntimeEvalSyntaxEvidence {
    runtime_eval_syntax_evidence(
        "general:gaps_in",
        "5",
        "6",
        vec![
            RuntimeMutationSyntaxCandidate {
                syntax_name: "legacy keyword".to_string(),
                command_pair: RuntimeMutationCommandPair {
                    mutation_command: "hyprctl keyword general:gaps_in 6".to_string(),
                    restore_command: "hyprctl keyword general:gaps_in 5".to_string(),
                },
                status: RuntimeMutationSyntaxStatus::FailedBeforeValueChange,
                error: Some("keyword can't work with non-legacy parsers. Use eval.".to_string()),
                post_mutation_value: Some("5".to_string()),
                post_restore_value: Some("5".to_string()),
            },
            RuntimeMutationSyntaxCandidate {
                syntax_name: "assignment eval".to_string(),
                command_pair: RuntimeMutationCommandPair {
                    mutation_command: "hyprctl eval 'general:gaps_in = 6'".to_string(),
                    restore_command: "hyprctl eval 'general:gaps_in = 5'".to_string(),
                },
                status: RuntimeMutationSyntaxStatus::FailedBeforeValueChange,
                error: Some("function arguments expected near '='".to_string()),
                post_mutation_value: Some("5".to_string()),
                post_restore_value: Some("5".to_string()),
            },
            RuntimeMutationSyntaxCandidate {
                syntax_name: "lua hl.config eval".to_string(),
                command_pair: RuntimeMutationCommandPair {
                    mutation_command: "hyprctl eval 'hl.config({ general = { gaps_in = 6 } })'"
                        .to_string(),
                    restore_command: "hyprctl eval 'hl.config({ general = { gaps_in = 5 } })'"
                        .to_string(),
                },
                status: RuntimeMutationSyntaxStatus::MutatedAndRestored,
                error: None,
                post_mutation_value: Some("6".to_string()),
                post_restore_value: Some("5".to_string()),
            },
        ],
    )
}

fn proven_runtime_live_restore_proof() -> RuntimeLiveRestoreProof {
    runtime_live_restore_attempt_review(
        RuntimeAction::Keyword {
            key: "general:gaps_in".to_string(),
            value: "6".to_string(),
        },
        true,
        Some("5"),
        Some("6"),
        Some("hyprctl eval 'hl.config({ general = { gaps_in = 5 } })'"),
        Some("hyprctl eval 'hl.config({ general = { gaps_in = 6 } })'"),
        true,
        Some("6"),
        Some("5"),
    )
}

#[test]
fn controlled_live_test_guard_requires_reversible_proof_before_any_mutation() {
    let root = temp_root("controlled-live-guard");
    let target = root.join("hyprland.conf");
    write_file(&target, "misc:disable_splash_rendering = true\n");
    let missing = ControlledLiveTestGuardRequest {
        test_id: String::new(),
        target_paths: Vec::new(),
        backup_paths_recorded: false,
        original_hashes_recorded: false,
        symlink_targets_recorded: false,
        read_only_runtime_snapshot_recorded: false,
        restore_plan_recorded: false,
        post_restore_verification_planned: false,
        out_of_band_recovery_recorded: false,
        trusted_data_available: false,
        explicit_live_flag: false,
    };
    let blocked = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::HighRiskDisplayWrite,
        missing,
    );
    assert!(!blocked.live_mutation_allowed);
    assert!(!blocked.real_config_touch_allowed);
    assert!(!blocked.runtime_mutation_allowed);
    assert!(blocked
        .blockers
        .iter()
        .any(|blocker| blocker.contains("out-of-band recovery")));
    assert!(blocked
        .blockers
        .iter()
        .any(|blocker| blocker.contains("explicit live-test flag")));

    let duplicate = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::DuplicateReplacement,
        complete_live_guard_request(target.clone()),
    );
    assert!(duplicate.live_mutation_allowed);
    assert!(duplicate.real_config_touch_allowed);
    assert!(!duplicate.runtime_mutation_allowed);
    assert!(duplicate.restore_required);

    let structured = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::StructuredWrite,
        complete_live_guard_request(target.clone()),
    );
    assert!(structured.live_mutation_allowed);
    assert!(structured.real_config_touch_allowed);

    let source_include = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::SourceIncludeInsertion,
        complete_live_guard_request(target),
    );
    assert!(source_include.live_mutation_allowed);
    assert!(source_include.real_config_touch_allowed);
}

#[test]
fn controlled_live_test_guard_has_category_specific_profile_runtime_and_migration_gates() {
    let root = temp_root("controlled-live-category-gates");
    let target = root.join("hyprland.conf");
    write_file(&target, "source = profiles/current.conf\n");

    let mut no_symlink = complete_live_guard_request(target.clone());
    no_symlink.symlink_targets_recorded = false;
    let profile = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::ProfileSwitch,
        no_symlink,
    );
    assert!(!profile.live_mutation_allowed);
    assert!(profile
        .blockers
        .iter()
        .any(|blocker| blocker.contains("symlink targets")));

    let mut no_runtime_snapshot = complete_live_guard_request(target.clone());
    no_runtime_snapshot.read_only_runtime_snapshot_recorded = false;
    let runtime = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::RuntimeMutation,
        no_runtime_snapshot,
    );
    assert!(!runtime.live_mutation_allowed);
    assert!(runtime
        .blockers
        .iter()
        .any(|blocker| blocker.contains("runtime snapshot")));

    let runtime_ready = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::RuntimeMutation,
        complete_live_guard_request(target.clone()),
    );
    assert!(runtime_ready.live_mutation_allowed);
    assert!(!runtime_ready.real_config_touch_allowed);
    assert!(runtime_ready.runtime_mutation_allowed);

    let mut no_trusted_data = complete_live_guard_request(target);
    no_trusted_data.trusted_data_available = false;
    let migration = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::HyprlandVersionMigration,
        no_trusted_data,
    );
    assert!(!migration.live_mutation_allowed);
    assert!(migration
        .blockers
        .iter()
        .any(|blocker| blocker.contains("trusted versioned data")));
}

#[test]
fn source_include_guarded_executor_inserts_verifies_and_restores_temp_fixture() {
    let root = temp_root("source-guarded-executor");
    let root_conf = root.join("hyprland.conf");
    let sourced_conf = root.join("appearance.conf");
    write_file(&root_conf, "source = appearance.conf\n");
    write_file(&sourced_conf, "decoration:blur:enabled = true\n");
    let proof = source_include_target_selection_fixture_proof(
        &root_conf,
        vec![SourceIncludeTargetCandidate {
            path: sourced_conf.clone(),
            source_depth: 1,
            generated_or_script_managed: false,
            symlink_or_profile_managed: false,
        }],
        Some(sourced_conf.clone()),
        false,
    );
    let plan = build_missing_default_insertion_plan(MissingDefaultInsertionRequest {
        setting_id: "misc.disable_splash_rendering".to_string(),
        proposed_value: "true".to_string(),
        target_path: sourced_conf.clone(),
        backup_stamp: "guarded-source".to_string(),
    });
    let dry_run = source_include_selected_target_dry_run_plan(&proof, &plan);
    let guard = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::SourceIncludeInsertion,
        complete_live_guard_request(sourced_conf.clone()),
    );
    let original = fs::read_to_string(&sourced_conf).expect("original should read");
    let report =
        execute_source_include_selected_target_guarded_temp(&proof, &dry_run, &guard, false, false);
    assert_eq!(
        report.status,
        GuardedTempExecutionStatus::SucceededAndRestored
    );
    assert!(report.mutation_verified);
    assert!(report.restore_succeeded);
    assert_eq!(
        fs::read_to_string(&sourced_conf).expect("restored should read"),
        original
    );
    assert!(!report.production_write_enabled);
    assert!(!report.real_config_touched);
}

#[test]
fn source_include_guarded_executor_refuses_missing_guard_non_temp_and_restores_after_verify_failure(
) {
    let root = temp_root("source-guarded-blocks");
    let config = root.join("hyprland.conf");
    write_file(&config, "decoration:blur:enabled = true\n");
    let proof = source_include_target_selection_fixture_proof(
        &config,
        vec![SourceIncludeTargetCandidate {
            path: config.clone(),
            source_depth: 0,
            generated_or_script_managed: false,
            symlink_or_profile_managed: false,
        }],
        Some(config.clone()),
        false,
    );
    let plan = build_missing_default_insertion_plan(MissingDefaultInsertionRequest {
        setting_id: "misc.disable_splash_rendering".to_string(),
        proposed_value: "true".to_string(),
        target_path: config.clone(),
        backup_stamp: "guarded-source-blocked".to_string(),
    });
    let dry_run = source_include_selected_target_dry_run_plan(&proof, &plan);
    let blocked_guard = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::SourceIncludeInsertion,
        ControlledLiveTestGuardRequest {
            explicit_live_flag: false,
            ..complete_live_guard_request(config.clone())
        },
    );
    let blocked = execute_source_include_selected_target_guarded_temp(
        &proof,
        &dry_run,
        &blocked_guard,
        false,
        false,
    );
    assert_eq!(blocked.status, GuardedTempExecutionStatus::Blocked);

    let guard = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::SourceIncludeInsertion,
        complete_live_guard_request(config.clone()),
    );
    let original = fs::read_to_string(&config).expect("original should read");
    let verify_failure =
        execute_source_include_selected_target_guarded_temp(&proof, &dry_run, &guard, true, false);
    assert_eq!(
        verify_failure.status,
        GuardedTempExecutionStatus::VerificationFailedRestored
    );
    assert_eq!(fs::read_to_string(&config).expect("restored"), original);

    let non_temp_proof = source_include_target_selection_fixture_proof(
        "/home/kyo/Documents/hyprland.conf",
        vec![SourceIncludeTargetCandidate {
            path: PathBuf::from("/home/kyo/Documents/hyprland.conf"),
            source_depth: 0,
            generated_or_script_managed: false,
            symlink_or_profile_managed: false,
        }],
        Some(PathBuf::from("/home/kyo/Documents/hyprland.conf")),
        false,
    );
    let non_temp_plan = build_missing_default_insertion_plan(MissingDefaultInsertionRequest {
        setting_id: "misc.disable_splash_rendering".to_string(),
        proposed_value: "true".to_string(),
        target_path: PathBuf::from("/home/kyo/Documents/hyprland.conf"),
        backup_stamp: "non-temp".to_string(),
    });
    let non_temp_dry_run =
        source_include_selected_target_dry_run_plan(&non_temp_proof, &non_temp_plan);
    let non_temp = execute_source_include_selected_target_guarded_temp(
        &non_temp_proof,
        &non_temp_dry_run,
        &guard,
        false,
        false,
    );
    assert_eq!(non_temp.status, GuardedTempExecutionStatus::Blocked);
}

#[cfg(unix)]
#[test]
fn copied_config_tree_runs_guarded_executors_on_copies_and_restores_everything() {
    let realish = temp_root("copied-tree-realish");
    let root_conf = realish.join("hyprland.conf");
    let sourced_conf = realish.join("appearance.conf");
    let profiles = realish.join("profiles");
    let desktop = profiles.join("desktop.conf");
    let gaming = profiles.join("gaming.conf");
    let current = profiles.join("current.conf");
    write_file(
        &root_conf,
        "source = appearance.conf\nsource = profiles/current.conf\nsource = profiles/gaming.conf\n",
    );
    write_file(
        &sourced_conf,
        "decoration:blur:enabled = true\ndecoration:blur:enabled = false\nbind = SUPER, Return, exec, foot\n# keep comment\n",
    );
    write_file(&desktop, "misc:disable_splash_rendering = false\n");
    write_file(&gaming, "misc:disable_splash_rendering = true\n");
    std::os::unix::fs::symlink(&desktop, &current).expect("profile symlink should create");

    let copy_root = temp_root("copied-tree-copy");
    let snapshot = copy_config_tree_for_proof(&root_conf, &copy_root);
    assert!(snapshot.errors.is_empty(), "{:?}", snapshot.errors);
    assert!(!snapshot.real_config_touched);
    assert!(!snapshot.runtime_touched);
    assert!(copied_config_tree_originals_unchanged(&snapshot));
    assert!(copied_config_tree_files_restored(&snapshot));

    let copied_sourced = snapshot
        .files
        .iter()
        .find(|file| file.original_path == sourced_conf)
        .expect("sourced file should be copied")
        .clone();
    let copied_root = snapshot
        .files
        .iter()
        .find(|file| file.original_path == root_conf)
        .expect("root file should be copied")
        .clone();
    let copied_current = snapshot
        .files
        .iter()
        .find(|file| file.original_path == current)
        .expect("current symlink should be copied")
        .clone();
    let copied_gaming = snapshot
        .files
        .iter()
        .find(|file| file.original_path == gaming)
        .expect("gaming profile should be copied")
        .clone();
    assert!(copied_root.target_eligible);
    assert!(copied_sourced.target_eligible);
    assert!(!copied_current.target_eligible);

    let source_proof = source_include_target_selection_fixture_proof(
        snapshot.copied_root_path.clone(),
        vec![
            SourceIncludeTargetCandidate {
                path: copied_root.copied_path.clone(),
                source_depth: copied_root.source_depth,
                generated_or_script_managed: copied_root.generated_or_script_managed,
                symlink_or_profile_managed: copied_root.symlink_or_profile_managed,
            },
            SourceIncludeTargetCandidate {
                path: copied_sourced.copied_path.clone(),
                source_depth: copied_sourced.source_depth,
                generated_or_script_managed: copied_sourced.generated_or_script_managed,
                symlink_or_profile_managed: copied_sourced.symlink_or_profile_managed,
            },
        ],
        Some(copied_sourced.copied_path.clone()),
        false,
    );
    let insertion_plan = build_missing_default_insertion_plan(MissingDefaultInsertionRequest {
        setting_id: "misc.disable_splash_rendering".to_string(),
        proposed_value: "true".to_string(),
        target_path: copied_sourced.copied_path.clone(),
        backup_stamp: "copied-tree-source".to_string(),
    });
    let dry_run = source_include_selected_target_dry_run_plan(&source_proof, &insertion_plan);
    let source_guard = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::SourceIncludeInsertion,
        complete_live_guard_request(copied_sourced.copied_path.clone()),
    );
    let source_report = execute_source_include_selected_target_guarded_temp(
        &source_proof,
        &dry_run,
        &source_guard,
        false,
        false,
    );
    assert_eq!(
        source_report.status,
        GuardedTempExecutionStatus::SucceededAndRestored
    );
    assert!(copied_config_tree_originals_unchanged(&snapshot));
    assert!(copied_config_tree_files_restored(&snapshot));

    let duplicate_model = duplicate_occurrence_model(
        "decoration.blur.enabled",
        &[(
            copied_sourced.copied_path.clone(),
            copied_sourced.source_depth,
        )],
    )
    .expect("duplicate model should build from copied tree");
    let duplicate_occurrence = duplicate_model.occurrences[1].clone();
    let duplicate_confirmation = duplicate_occurrence_confirmation(
        Some(&duplicate_occurrence),
        Some("token"),
        "token",
        false,
        false,
    );
    let duplicate_request = DuplicateReplacementRequest {
        occurrence: duplicate_occurrence,
        expected_old_value: "false".to_string(),
        proposed_value: "true".to_string(),
        backup_stamp: "copied-tree-duplicate".to_string(),
    };
    let duplicate_guard = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::DuplicateReplacement,
        complete_live_guard_request(copied_sourced.copied_path.clone()),
    );
    let duplicate_report = execute_duplicate_replacement_guarded_temp(
        &duplicate_confirmation,
        &duplicate_request,
        &duplicate_guard,
        false,
        false,
    );
    assert_eq!(
        duplicate_report.status,
        GuardedTempExecutionStatus::SucceededAndRestored
    );
    assert!(copied_config_tree_originals_unchanged(&snapshot));
    assert!(copied_config_tree_files_restored(&snapshot));

    let structured_guard = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::StructuredWrite,
        complete_live_guard_request(copied_sourced.copied_path.clone()),
    );
    let structured_report = execute_structured_bind_guarded_temp(
        &copied_sourced.copied_path,
        3,
        "bind = SUPER, Return, exec, foot",
        "bind = SUPER, Return, exec, kitty",
        &structured_guard,
        false,
        false,
    );
    assert_eq!(
        structured_report.status,
        GuardedTempExecutionStatus::SucceededAndRestored
    );
    assert!(copied_config_tree_originals_unchanged(&snapshot));
    assert!(copied_config_tree_files_restored(&snapshot));

    let profile_guard = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::ProfileSwitch,
        ControlledLiveTestGuardRequest {
            symlink_targets_recorded: true,
            ..complete_live_guard_request(copied_current.copied_path.clone())
        },
    );
    let profile_report = switch_profile_symlink_guarded_temp(
        &snapshot.copy_root,
        &copied_current.copied_path,
        &copied_gaming.copied_path,
        &profile_guard,
        false,
    );
    assert_eq!(profile_report.status, ProfileSwitchStatus::Succeeded);
    let profile_restored = profile_report.restored_target == profile_report.original_target;
    assert!(profile_restored);
    assert!(copied_config_tree_originals_unchanged(&snapshot));
    assert!(copied_config_tree_files_restored(&snapshot));

    let report = copied_config_tree_report(
        snapshot,
        Some(&source_report),
        Some(&duplicate_report),
        Some(&structured_report),
        Some(profile_restored),
    );
    assert!(report.originals_unchanged);
    assert!(report.copied_files_restored);
    assert_eq!(report.source_include_executor_restored, Some(true));
    assert_eq!(report.duplicate_executor_restored, Some(true));
    assert_eq!(report.structured_executor_restored, Some(true));
    assert_eq!(report.profile_executor_restored, Some(true));
}

#[cfg(unix)]
#[test]
fn copied_config_tree_proof_drives_default_disabled_production_gates() {
    let realish = temp_root("production-gates-copied-tree-realish");
    let root_conf = realish.join("hyprland.conf");
    let sourced_conf = realish.join("appearance.conf");
    let profiles = realish.join("profiles");
    let desktop = profiles.join("desktop.conf");
    let gaming = profiles.join("gaming.conf");
    let current = profiles.join("current.conf");
    write_file(
        &root_conf,
        "source = appearance.conf\nsource = profiles/current.conf\nsource = profiles/gaming.conf\n",
    );
    write_file(
        &sourced_conf,
        "decoration:blur:enabled = true\ndecoration:blur:enabled = false\nbind = SUPER, Return, exec, foot\n# keep comment\n",
    );
    write_file(&desktop, "misc:disable_splash_rendering = false\n");
    write_file(&gaming, "misc:disable_splash_rendering = true\n");
    std::os::unix::fs::symlink(&desktop, &current).expect("profile symlink should create");

    let copy_root = temp_root("production-gates-copied-tree-copy");
    let snapshot = copy_config_tree_for_proof(&root_conf, &copy_root);
    let copied_sourced = snapshot
        .files
        .iter()
        .find(|file| file.original_path == sourced_conf)
        .expect("sourced file should be copied")
        .clone();
    let copied_root = snapshot
        .files
        .iter()
        .find(|file| file.original_path == root_conf)
        .expect("root file should be copied")
        .clone();
    let copied_current = snapshot
        .files
        .iter()
        .find(|file| file.original_path == current)
        .expect("current symlink should be copied")
        .clone();
    let copied_gaming = snapshot
        .files
        .iter()
        .find(|file| file.original_path == gaming)
        .expect("gaming profile should be copied")
        .clone();

    let candidates = vec![
        SourceIncludeTargetCandidate {
            path: copied_root.copied_path.clone(),
            source_depth: copied_root.source_depth,
            generated_or_script_managed: copied_root.generated_or_script_managed,
            symlink_or_profile_managed: copied_root.symlink_or_profile_managed,
        },
        SourceIncludeTargetCandidate {
            path: copied_sourced.copied_path.clone(),
            source_depth: copied_sourced.source_depth,
            generated_or_script_managed: copied_sourced.generated_or_script_managed,
            symlink_or_profile_managed: copied_sourced.symlink_or_profile_managed,
        },
    ];
    let source_proof = source_include_target_selection_fixture_proof(
        snapshot.copied_root_path.clone(),
        candidates.clone(),
        Some(copied_sourced.copied_path.clone()),
        false,
    );
    let insertion_plan = build_missing_default_insertion_plan(MissingDefaultInsertionRequest {
        setting_id: "misc.disable_splash_rendering".to_string(),
        proposed_value: "true".to_string(),
        target_path: copied_sourced.copied_path.clone(),
        backup_stamp: "production-gate-source".to_string(),
    });
    let dry_run = source_include_selected_target_dry_run_plan(&source_proof, &insertion_plan);
    let source_guard = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::SourceIncludeInsertion,
        complete_live_guard_request(copied_sourced.copied_path.clone()),
    );
    let source_report = execute_source_include_selected_target_guarded_temp(
        &source_proof,
        &dry_run,
        &source_guard,
        false,
        false,
    );

    let duplicate_model = duplicate_occurrence_model(
        "decoration.blur.enabled",
        &[(
            copied_sourced.copied_path.clone(),
            copied_sourced.source_depth,
        )],
    )
    .expect("duplicate model should build");
    let duplicate_occurrence = duplicate_model.occurrences[1].clone();
    let duplicate_confirmation = duplicate_occurrence_confirmation(
        Some(&duplicate_occurrence),
        Some("token"),
        "token",
        false,
        false,
    );
    let duplicate_request = DuplicateReplacementRequest {
        occurrence: duplicate_occurrence.clone(),
        expected_old_value: "false".to_string(),
        proposed_value: "true".to_string(),
        backup_stamp: "production-gate-duplicate".to_string(),
    };
    let duplicate_guard = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::DuplicateReplacement,
        complete_live_guard_request(copied_sourced.copied_path.clone()),
    );
    let duplicate_report = execute_duplicate_replacement_guarded_temp(
        &duplicate_confirmation,
        &duplicate_request,
        &duplicate_guard,
        false,
        false,
    );

    let structured_guard = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::StructuredWrite,
        complete_live_guard_request(copied_sourced.copied_path.clone()),
    );
    let structured_report = execute_structured_bind_guarded_temp(
        &copied_sourced.copied_path,
        3,
        "bind = SUPER, Return, exec, foot",
        "bind = SUPER, Return, exec, kitty",
        &structured_guard,
        false,
        false,
    );

    let profile_guard = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::ProfileSwitch,
        ControlledLiveTestGuardRequest {
            symlink_targets_recorded: true,
            ..complete_live_guard_request(copied_current.copied_path.clone())
        },
    );
    let profile_report = switch_profile_symlink_guarded_temp(
        &snapshot.copy_root,
        &copied_current.copied_path,
        &copied_gaming.copied_path,
        &profile_guard,
        false,
    );
    let profile_restored = profile_report.restored_target == profile_report.original_target;
    let copied_report = copied_config_tree_report(
        snapshot,
        Some(&source_report),
        Some(&duplicate_report),
        Some(&structured_report),
        Some(profile_restored),
    );

    let source_gate = source_include_production_gate_review(
        &source_proof,
        Some(&dry_run),
        Some(&copied_report),
        false,
    );
    assert_eq!(
        source_gate.gate.status,
        SourceIncludeProductionGateStatus::ReadyButDefaultDisabled
    );
    assert!(source_gate.gate.copied_proof_restored);
    assert!(!source_gate.gate.production_apply_enabled);

    let no_target = source_include_target_selection_fixture_proof(
        copied_root.copied_path.clone(),
        candidates,
        None,
        false,
    );
    let no_target_gate = source_include_production_gate_review(&no_target, None, None, false);
    assert_eq!(
        no_target_gate.gate.status,
        SourceIncludeProductionGateStatus::NoTargetSelected
    );
    let missing_source_approval = source_include_approval_flow(&source_gate, None);
    assert_eq!(
        missing_source_approval.status,
        ApprovalStatus::MissingEvidence
    );
    let wrong_source_target = approval_request(
        ApprovalScope::SourceIncludeInsertion,
        Some(copied_root.copied_path.clone()),
        None,
        true,
        false,
    );
    let wrong_source_target_decision =
        source_include_approval_flow(&source_gate, Some(&wrong_source_target));
    assert_eq!(
        wrong_source_target_decision.status,
        ApprovalStatus::MissingEvidence
    );
    let source_approval = approval_request(
        ApprovalScope::SourceIncludeInsertion,
        source_gate.gate.selected_target_path.clone(),
        None,
        true,
        false,
    );
    let approved_source = source_include_approval_flow(&source_gate, Some(&source_approval));
    assert_eq!(
        approved_source.status,
        ApprovalStatus::ApprovedButDefaultDisabled
    );
    assert!(!approved_source.production_apply_enabled);

    let duplicate_gate = duplicate_production_gate_review(
        Some(&duplicate_occurrence),
        Some(&duplicate_confirmation),
        Some(&copied_report),
        Some("true".to_string()),
        false,
    );
    assert_eq!(
        duplicate_gate.status,
        DuplicateProductionGateStatus::ReadyButDefaultDisabled
    );
    assert!(duplicate_gate.copied_proof_restored);
    assert!(!duplicate_gate.production_apply_enabled);
    let pending = duplicate_occurrence_confirmation(
        Some(&duplicate_occurrence),
        Some("wrong"),
        "token",
        false,
        false,
    );
    let pending_gate = duplicate_production_gate_review(
        Some(&duplicate_occurrence),
        Some(&pending),
        Some(&copied_report),
        Some("true".to_string()),
        false,
    );
    assert_eq!(
        pending_gate.status,
        DuplicateProductionGateStatus::PendingConfirmation
    );
    let wrong_duplicate_scope = approval_request(
        ApprovalScope::SourceIncludeInsertion,
        duplicate_gate.selected_path.clone(),
        None,
        true,
        false,
    );
    let wrong_duplicate_scope_decision =
        duplicate_approval_flow(&duplicate_gate, Some(&wrong_duplicate_scope));
    assert_eq!(
        wrong_duplicate_scope_decision.status,
        ApprovalStatus::WrongScope
    );
    let duplicate_approval = approval_request(
        ApprovalScope::DuplicateReplacement,
        duplicate_gate.selected_path.clone(),
        None,
        true,
        false,
    );
    let approved_duplicate = duplicate_approval_flow(&duplicate_gate, Some(&duplicate_approval));
    assert_eq!(
        approved_duplicate.status,
        ApprovalStatus::ApprovedButDefaultDisabled
    );
    assert!(!approved_duplicate.production_apply_enabled);

    let structured_gate = structured_production_gate_review(
        "hl.bind",
        copied_sourced.copied_path.clone(),
        3,
        "bind = SUPER, Return, exec, foot",
        "bind = SUPER, Return, exec, kitty",
        Some(&copied_report),
        true,
        false,
    );
    assert_eq!(
        structured_gate.status,
        StructuredProductionGateStatus::ReadyButDefaultDisabled
    );
    assert!(structured_gate.copied_proof_restored);
    assert!(!structured_gate.production_apply_enabled);
    let invalid_structured = structured_production_gate_review(
        "hl.bind",
        copied_sourced.copied_path.clone(),
        3,
        "bind = SUPER, Return, exec, foot",
        "monitor = HDMI-A-1, preferred, auto, 1",
        Some(&copied_report),
        true,
        false,
    );
    assert_eq!(
        invalid_structured.status,
        StructuredProductionGateStatus::InvalidCandidate
    );
    let structured_approval = approval_request(
        ApprovalScope::StructuredHlBindWrite,
        Some(structured_gate.target_path.clone()),
        None,
        true,
        false,
    );
    let approved_structured =
        structured_approval_flow(&structured_gate, Some(&structured_approval));
    assert_eq!(
        approved_structured.status,
        ApprovalStatus::ApprovedButDefaultDisabled
    );
    assert!(!approved_structured.production_apply_enabled);

    let profile_gate = profile_production_gate_review(
        copied_current.copied_path.clone(),
        copied_current.copied_symlink_target.clone(),
        Some(copied_gaming.copied_path.clone()),
        Some(&copied_report),
        false,
    );
    assert_eq!(
        profile_gate.status,
        ProfileProductionGateStatus::ReadyButDefaultDisabled
    );
    assert!(profile_gate.copied_proof_restored);
    assert!(!profile_gate.production_switch_enabled);
    let profile_missing_selection = profile_production_gate_review(
        copied_current.copied_path,
        copied_current.copied_symlink_target,
        None,
        Some(&copied_report),
        false,
    );
    assert_eq!(
        profile_missing_selection.status,
        ProfileProductionGateStatus::NoSelection
    );
    let profile_approval = approval_request(
        ApprovalScope::ProfileModeSwitch,
        Some(profile_gate.symlink_path.clone()),
        None,
        true,
        false,
    );
    let approved_profile = profile_approval_flow(&profile_gate, Some(&profile_approval));
    assert_eq!(
        approved_profile.status,
        ApprovalStatus::ApprovedButDefaultDisabled
    );
    assert!(!approved_profile.production_apply_enabled);
}

#[test]
fn explicit_approval_flow_blocks_missing_wrong_expired_rejected_and_incomplete_evidence() {
    let target = temp_root("approval-flow").join("hyprland.conf");
    let missing = approval_decision_for_gate(
        ApprovalScope::DuplicateReplacement,
        true,
        Some(&target),
        None,
        None,
        false,
    );
    assert_eq!(missing.status, ApprovalStatus::MissingEvidence);

    let wrong_scope = approval_request(
        ApprovalScope::SourceIncludeInsertion,
        Some(target.clone()),
        None,
        true,
        false,
    );
    let wrong = approval_decision_for_gate(
        ApprovalScope::DuplicateReplacement,
        true,
        Some(&target),
        None,
        Some(&wrong_scope),
        false,
    );
    assert_eq!(wrong.status, ApprovalStatus::WrongScope);

    let mut expired = approval_request(
        ApprovalScope::DuplicateReplacement,
        Some(target.clone()),
        None,
        true,
        false,
    );
    expired.current_tick = 100;
    let expired_decision = approval_decision_for_gate(
        ApprovalScope::DuplicateReplacement,
        true,
        Some(&target),
        None,
        Some(&expired),
        false,
    );
    assert_eq!(expired_decision.status, ApprovalStatus::Expired);

    let mut rejected = approval_request(
        ApprovalScope::DuplicateReplacement,
        Some(target.clone()),
        None,
        true,
        false,
    );
    rejected.rejected = true;
    let rejected_decision = approval_decision_for_gate(
        ApprovalScope::DuplicateReplacement,
        true,
        Some(&target),
        None,
        Some(&rejected),
        false,
    );
    assert_eq!(rejected_decision.status, ApprovalStatus::Rejected);

    let mut incomplete = approval_request(
        ApprovalScope::DuplicateReplacement,
        Some(target.clone()),
        None,
        false,
        false,
    );
    incomplete.evidence.restore_plan = None;
    let incomplete_decision = approval_decision_for_gate(
        ApprovalScope::DuplicateReplacement,
        true,
        Some(&target),
        None,
        Some(&incomplete),
        false,
    );
    assert_eq!(incomplete_decision.status, ApprovalStatus::MissingEvidence);

    let live_restore = approval_request(
        ApprovalScope::DuplicateReplacement,
        Some(target.clone()),
        None,
        false,
        true,
    );
    let live_ready = approval_decision_for_gate(
        ApprovalScope::DuplicateReplacement,
        true,
        Some(&target),
        None,
        Some(&live_restore),
        false,
    );
    assert_eq!(live_ready.status, ApprovalStatus::ReadyButDefaultDisabled);
    assert!(!live_ready.production_apply_enabled);
}

#[test]
fn copied_config_tree_classifies_generated_and_unknown_targets_without_writing_originals() {
    let realish = temp_root("copied-tree-generated");
    let root_conf = realish.join("hyprland.conf");
    let generated = realish.join("generated.conf");
    write_file(&root_conf, "source = generated.conf\n");
    write_file(
        &generated,
        "# Generated by a script; do not edit\nmisc:disable_splash_rendering = false\n",
    );
    let copy_root = temp_root("copied-tree-generated-copy");
    let snapshot = copy_config_tree_for_proof(&root_conf, &copy_root);
    assert!(snapshot.errors.is_empty(), "{:?}", snapshot.errors);
    let copied_generated = snapshot
        .files
        .iter()
        .find(|file| file.original_path == generated)
        .expect("generated file should be copied");
    assert!(copied_generated.generated_or_script_managed);
    assert!(!copied_generated.target_eligible);
    assert!(copied_config_tree_originals_unchanged(&snapshot));
    assert!(copied_config_tree_files_restored(&snapshot));
}

#[test]
fn duplicate_occurrence_model_lists_same_file_and_source_layer_occurrences() {
    let root = temp_root("duplicate-model");
    let root_conf = root.join("hyprland.conf");
    let sourced_conf = root.join("appearance.conf");
    write_file(&root_conf, "decoration:blur:enabled = true\n");
    write_file(&sourced_conf, "decoration:blur:enabled = false\n");

    let model = duplicate_occurrence_model(
        "decoration.blur.enabled",
        &[(root_conf.clone(), 0), (sourced_conf.clone(), 1)],
    )
    .expect("duplicate model should build");

    assert_eq!(model.occurrences.len(), 2);
    assert!(!model.selector_enabled);
    assert!(!model.production_write_enabled);
    assert!(model
        .occurrences
        .iter()
        .any(|occurrence| occurrence.path == root_conf && occurrence.source_depth == 0));
    assert!(model
        .occurrences
        .iter()
        .any(|occurrence| occurrence.path == sourced_conf && occurrence.source_depth == 1));
}

#[test]
fn duplicate_occurrence_review_tracks_no_selection_and_selected_disabled_state() {
    let root = temp_root("duplicate-review");
    let root_conf = root.join("hyprland.conf");
    let sourced_conf = root.join("appearance.conf");
    write_file(&root_conf, "decoration:blur:enabled = true\n");
    write_file(&sourced_conf, "decoration:blur:enabled = false\n");
    let model = duplicate_occurrence_model(
        "decoration.blur.enabled",
        &[(root_conf.clone(), 0), (sourced_conf.clone(), 1)],
    )
    .expect("duplicate model should build");

    let no_selection = duplicate_occurrence_review(&model, None, Some("false".to_string()));
    assert_eq!(
        no_selection.state,
        DuplicateOccurrenceReviewState::NoOccurrenceSelected
    );
    assert!(!no_selection.apply_enabled);
    assert!(!no_selection.production_write_enabled);
    assert!(!no_selection.write_execution_attempted);
    assert!(no_selection
        .review_lines
        .iter()
        .any(|line| line.contains("will not auto-choose")));

    let selected = duplicate_occurrence_review(&model, Some(1), Some("true".to_string()));
    assert_eq!(
        selected.state,
        DuplicateOccurrenceReviewState::OccurrenceSelectedProductionDisabled
    );
    assert_eq!(selected.selected_path.as_ref(), Some(&sourced_conf));
    assert_eq!(selected.selected_line_number, Some(1));
    assert_eq!(
        selected.selected_raw_line.as_deref(),
        Some("decoration:blur:enabled = false")
    );
    assert_eq!(selected.selected_current_value.as_deref(), Some("false"));
    assert_eq!(selected.proposed_value.as_deref(), Some("true"));
    assert_eq!(selected.source_depth, Some(1));
    assert!(!selected.apply_enabled);
    assert!(!selected.production_write_enabled);
    assert!(!selected.write_execution_attempted);
}

#[test]
fn duplicate_occurrence_review_rejects_stale_selection_without_writing() {
    let root = temp_root("duplicate-review-invalid");
    let config = root.join("hyprland.conf");
    write_file(&config, "decoration:blur:enabled = true\n");
    let model = duplicate_occurrence_model("decoration.blur.enabled", &[(config, 0)])
        .expect("duplicate model should build");

    let review = duplicate_occurrence_review(&model, Some(99), Some("false".to_string()));

    assert_eq!(
        review.state,
        DuplicateOccurrenceReviewState::InvalidSelection
    );
    assert!(!review.apply_enabled);
    assert!(!review.production_write_enabled);
    assert!(!review.write_execution_attempted);
    assert!(review.selected_path.is_none());
}

#[test]
fn duplicate_occurrence_confirmation_requires_matching_token_and_keeps_production_disabled() {
    let root = temp_root("duplicate-confirmation");
    let config = root.join("hyprland.conf");
    write_file(&config, "decoration:blur:enabled = true\n");
    let occurrence = duplicate_occurrence_model("decoration.blur.enabled", &[(config.clone(), 0)])
        .expect("model should build")
        .occurrences[0]
        .clone();

    let missing = duplicate_occurrence_confirmation(None, None, "token", false, false);
    assert_eq!(
        missing.approval_state,
        DuplicateOccurrenceApprovalState::Missing
    );
    assert!(!missing.safe_env_replacement_allowed);
    assert!(!missing.apply_enabled);

    let pending =
        duplicate_occurrence_confirmation(Some(&occurrence), Some("wrong"), "token", false, false);
    assert_eq!(
        pending.approval_state,
        DuplicateOccurrenceApprovalState::Pending
    );
    assert!(!pending.token_matched);
    assert!(!pending.safe_env_replacement_allowed);

    let confirmed =
        duplicate_occurrence_confirmation(Some(&occurrence), Some("token"), "token", false, false);
    assert_eq!(
        confirmed.approval_state,
        DuplicateOccurrenceApprovalState::Confirmed
    );
    assert!(confirmed.token_matched);
    assert!(confirmed.safe_env_replacement_allowed);
    assert!(!confirmed.production_write_enabled);
    assert!(!confirmed.apply_enabled);
    assert!(confirmed
        .occurrence_fingerprint
        .as_deref()
        .expect("fingerprint")
        .contains("decoration:blur:enabled = true"));

    let expired =
        duplicate_occurrence_confirmation(Some(&occurrence), Some("token"), "token", false, true);
    assert_eq!(
        expired.approval_state,
        DuplicateOccurrenceApprovalState::Expired
    );
    assert!(!expired.safe_env_replacement_allowed);
}

#[test]
fn duplicate_production_gate_requires_confirmed_matching_occurrence_and_blocks_apply() {
    let root = temp_root("duplicate-production-gate");
    let config = root.join("hyprland.conf");
    write_file(&config, "decoration:blur:enabled = true\n");
    let occurrence = duplicate_occurrence_model("decoration.blur.enabled", &[(config.clone(), 1)])
        .expect("model should build")
        .occurrences[0]
        .clone();

    let missing = duplicate_production_approval_gate(Some(&occurrence), None);
    assert_eq!(
        missing.status,
        DuplicateProductionGateStatus::MissingConfirmation
    );
    assert!(!missing.safe_env_replacement_allowed);
    assert!(!missing.production_apply_enabled);
    assert!(!missing.duplicate_write_enabled);

    let pending =
        duplicate_occurrence_confirmation(Some(&occurrence), Some("wrong"), "token", false, false);
    let pending_gate = duplicate_production_approval_gate(Some(&occurrence), Some(&pending));
    assert_eq!(
        pending_gate.status,
        DuplicateProductionGateStatus::PendingConfirmation
    );
    assert!(!pending_gate.safe_env_replacement_allowed);

    let confirmed =
        duplicate_occurrence_confirmation(Some(&occurrence), Some("token"), "token", false, false);
    let confirmed_gate = duplicate_production_approval_gate(Some(&occurrence), Some(&confirmed));
    assert_eq!(
        confirmed_gate.status,
        DuplicateProductionGateStatus::ConfirmedButProductionDisabled
    );
    assert!(confirmed_gate.safe_env_replacement_allowed);
    assert!(!confirmed_gate.production_apply_enabled);
    assert!(!confirmed_gate.duplicate_write_enabled);
    assert_eq!(
        confirmed_gate
            .precondition
            .as_ref()
            .expect("precondition")
            .source_depth,
        1
    );

    let mut stale = occurrence.clone();
    stale.raw_line = "decoration:blur:enabled = false".to_string();
    let stale_gate = duplicate_production_approval_gate(Some(&stale), Some(&confirmed));
    assert_eq!(
        stale_gate.status,
        DuplicateProductionGateStatus::FingerprintMismatch
    );
    assert!(!stale_gate.safe_env_replacement_allowed);
}

#[test]
fn duplicate_replacement_safe_env_replaces_exact_selected_line_and_verifies() {
    let root = temp_root("duplicate-replace");
    let config = root.join("hyprland.conf");
    write_file(
        &config,
        "decoration:blur:enabled = true\nmisc:disable_splash_rendering = false\n",
    );
    let model = duplicate_occurrence_model("decoration.blur.enabled", &[(config.clone(), 0)])
        .expect("model should build");
    let occurrence = model.occurrences[0].clone();

    let report = replace_duplicate_occurrence_safe_env(
        &DuplicateReplacementRequest {
            occurrence,
            expected_old_value: "true".to_string(),
            proposed_value: "false".to_string(),
            backup_stamp: "fixture".to_string(),
        },
        &DuplicateReplacementOptions::default(),
    );

    assert_eq!(report.status, DuplicateReplacementStatus::Succeeded);
    assert!(report.backup_bytes_equal);
    assert!(report.exact_line_replaced);
    assert!(report.reread_verified);
    assert!(!report.production_write_enabled);
    assert!(!report.real_config_touched);
    assert!(fs::read_to_string(config)
        .expect("config should read")
        .starts_with("decoration:blur:enabled = false\n"));
}

#[test]
fn duplicate_replacement_with_confirmation_blocks_unconfirmed_and_stale_occurrences() {
    let root = temp_root("duplicate-confirmed-replace");
    let config = root.join("hyprland.conf");
    write_file(&config, "decoration:blur:enabled = true\n");
    let occurrence = duplicate_occurrence_model("decoration.blur.enabled", &[(config.clone(), 0)])
        .expect("model should build")
        .occurrences[0]
        .clone();
    let request = DuplicateReplacementRequest {
        occurrence: occurrence.clone(),
        expected_old_value: "true".to_string(),
        proposed_value: "false".to_string(),
        backup_stamp: "confirmed".to_string(),
    };

    let pending =
        duplicate_occurrence_confirmation(Some(&occurrence), Some("wrong"), "token", false, false);
    let blocked = replace_duplicate_occurrence_with_confirmation_safe_env(
        &pending,
        &request,
        &DuplicateReplacementOptions::default(),
    );
    assert_eq!(blocked.status, DuplicateReplacementStatus::Blocked);
    assert!(!blocked.exact_line_replaced);
    assert_eq!(
        fs::read_to_string(&config).expect("config should read"),
        "decoration:blur:enabled = true\n"
    );

    let confirmed =
        duplicate_occurrence_confirmation(Some(&occurrence), Some("token"), "token", false, false);
    let mut stale_request = request.clone();
    stale_request.occurrence.raw_value = "false".to_string();
    let stale = replace_duplicate_occurrence_with_confirmation_safe_env(
        &confirmed,
        &stale_request,
        &DuplicateReplacementOptions::default(),
    );
    assert_eq!(stale.status, DuplicateReplacementStatus::Blocked);
    assert!(!stale.exact_line_replaced);

    let report = replace_duplicate_occurrence_with_confirmation_safe_env(
        &confirmed,
        &request,
        &DuplicateReplacementOptions::default(),
    );
    assert_eq!(report.status, DuplicateReplacementStatus::Succeeded);
    assert!(report.exact_line_replaced);
    assert!(!report.production_write_enabled);
}

#[test]
fn duplicate_replacement_restores_after_verification_failure() {
    let root = temp_root("duplicate-restore");
    let config = root.join("hyprland.conf");
    let original = "decoration:blur:enabled = true\n";
    write_file(&config, original);
    let occurrence = duplicate_occurrence_model("decoration.blur.enabled", &[(config.clone(), 0)])
        .expect("model should build")
        .occurrences[0]
        .clone();

    let report = replace_duplicate_occurrence_safe_env(
        &DuplicateReplacementRequest {
            occurrence,
            expected_old_value: "true".to_string(),
            proposed_value: "false".to_string(),
            backup_stamp: "fixture".to_string(),
        },
        &DuplicateReplacementOptions {
            force_verification_failure: true,
            ..DuplicateReplacementOptions::default()
        },
    );

    assert_eq!(report.status, DuplicateReplacementStatus::RecoveredFailure);
    assert!(report.restore_attempted);
    assert!(report.restore_succeeded);
    assert_eq!(
        fs::read_to_string(config).expect("config should read"),
        original
    );
}

#[test]
fn duplicate_guarded_executor_replaces_and_restores_confirmed_temp_fixture() {
    let root = temp_root("duplicate-guarded");
    let config = root.join("hyprland.conf");
    let original =
        "decoration:blur:enabled = true\nsource = appearance.conf\ndecoration:blur:enabled = false\n";
    write_file(&config, original);
    let model = duplicate_occurrence_model("decoration.blur.enabled", &[(config.clone(), 0)])
        .expect("model should build");
    let occurrence = model.occurrences[1].clone();
    let confirmed =
        duplicate_occurrence_confirmation(Some(&occurrence), Some("token"), "token", false, false);
    let guard = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::DuplicateReplacement,
        complete_live_guard_request(config.clone()),
    );
    let request = DuplicateReplacementRequest {
        occurrence,
        expected_old_value: "false".to_string(),
        proposed_value: "true".to_string(),
        backup_stamp: "guarded-duplicate".to_string(),
    };
    let report =
        execute_duplicate_replacement_guarded_temp(&confirmed, &request, &guard, false, false);
    assert_eq!(
        report.status,
        GuardedTempExecutionStatus::SucceededAndRestored
    );
    assert!(report.mutation_verified);
    assert!(report.restore_succeeded);
    assert_eq!(
        fs::read_to_string(&config).expect("config should read"),
        original
    );
    assert!(!report.production_write_enabled);

    let pending = duplicate_occurrence_confirmation(
        Some(&request.occurrence),
        Some("wrong"),
        "token",
        false,
        false,
    );
    let blocked =
        execute_duplicate_replacement_guarded_temp(&pending, &request, &guard, false, false);
    assert_eq!(blocked.status, GuardedTempExecutionStatus::Blocked);

    let mut stale_request = request.clone();
    stale_request.occurrence.raw_line = "decoration:blur:enabled = maybe".to_string();
    let stale = execute_duplicate_replacement_guarded_temp(
        &confirmed,
        &stale_request,
        &guard,
        false,
        false,
    );
    assert_eq!(stale.status, GuardedTempExecutionStatus::Blocked);
}

#[test]
fn high_risk_mock_watchdog_handles_confirm_timeout_revert_and_failure() {
    let mut confirmed = MockWatchdog::arm("session", "token", 10);
    assert_eq!(confirmed.state, MockWatchdogState::Pending);
    assert_eq!(confirmed.confirm("wrong"), MockWatchdogState::Pending);
    assert_eq!(confirmed.confirm("token"), MockWatchdogState::Confirmed);
    assert!(!confirmed.real_runtime_enabled);

    let mut reverted = MockWatchdog::arm("session-2", "token", 10);
    assert_eq!(reverted.tick(11, true), MockWatchdogState::Reverted);

    let mut failed = MockWatchdog::arm("session-3", "token", 10);
    assert_eq!(failed.tick(11, false), MockWatchdogState::RecoveryFailed);
}

#[test]
fn high_risk_recovery_review_stays_non_mutating_and_disabled() {
    let mut watchdog = MockWatchdog::arm("session", "token", 10);
    watchdog.tick(11, true);

    let review = high_risk_recovery_review("render.direct_scanout", &watchdog);

    assert_eq!(review.state, MockWatchdogState::Reverted);
    assert!(!review.production_write_enabled);
    assert!(!review.real_runtime_enabled);
    assert!(review
        .review_lines
        .iter()
        .any(|line| line.contains("does not reload Hyprland")));
}

#[test]
fn high_risk_recovery_workflow_records_rollback_proof_without_runtime() {
    let pending = MockWatchdog::arm("session", "token", 10);
    let pending_workflow = high_risk_recovery_workflow("render.direct_scanout", &pending);
    assert_eq!(pending_workflow.state, MockWatchdogState::Pending);
    assert!(!pending_workflow.confirmation_enabled);
    assert!(!pending_workflow.revert_enabled);
    assert!(!pending_workflow.production_write_enabled);
    assert!(!pending_workflow.real_runtime_enabled);
    assert!(pending_workflow.rollback_proof.backup_before_write_required);
    assert!(pending_workflow.rollback_proof.reread_after_write_required);
    assert!(pending_workflow.rollback_proof.restore_on_timeout_required);
    assert!(
        pending_workflow
            .rollback_proof
            .reread_after_restore_required
    );
    assert!(!pending_workflow.rollback_proof.real_runtime_enabled);

    let mut failed = MockWatchdog::arm("session-fail", "token", 10);
    failed.tick(11, false);
    let failed_workflow = high_risk_recovery_workflow("decoration.screen_shader", &failed);
    assert_eq!(failed_workflow.state, MockWatchdogState::RecoveryFailed);
    assert!(failed_workflow
        .review_lines
        .iter()
        .any(|line| line.contains("recovery failure")));
}

#[test]
fn high_risk_live_recovery_protocol_is_noop_and_refuses_real_paths_or_runtime() {
    let root = temp_root("high-risk-live-protocol");
    let temp_config = root.join("hyprland.conf");
    write_file(&temp_config, "decoration:blur:enabled = true\n");

    let ready =
        high_risk_live_recovery_protocol("render.direct_scanout", &temp_config, true, false);
    assert_eq!(
        ready.status,
        HighRiskLiveReadinessStatus::NoopReadyForReview
    );
    assert!(ready.no_op_harness);
    assert!(!ready.accepts_real_config);
    assert!(!ready.mutating_runtime_enabled);
    assert!(!ready.live_write_enabled);

    let real_path = PathBuf::from("/home/kyo/.config/hypr/hyprland.conf");
    let real = high_risk_live_recovery_protocol("render.direct_scanout", real_path, true, false);
    assert_eq!(real.status, HighRiskLiveReadinessStatus::RealConfigRefused);
    assert!(!real.live_write_enabled);

    let runtime =
        high_risk_live_recovery_protocol("render.direct_scanout", &temp_config, true, true);
    assert_eq!(
        runtime.status,
        HighRiskLiveReadinessStatus::RuntimeMutationRefused
    );
    assert!(!runtime.mutating_runtime_enabled);
}

#[test]
fn high_risk_guarded_live_readiness_executor_requires_recovery_timeout_and_restore() {
    let root = temp_root("high-risk-guarded-readiness");
    let temp_config = root.join("hyprland.conf");
    write_file(&temp_config, "render:direct_scanout = false\n");
    let guard = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::HighRiskDisplayWrite,
        complete_live_guard_request(temp_config.clone()),
    );

    let ready = high_risk_guarded_live_readiness_executor(
        "render.direct_scanout",
        &temp_config,
        &guard,
        true,
        true,
    );
    assert_eq!(
        ready.status,
        HighRiskLiveReadinessStatus::NoopReadyForReview
    );
    assert!(ready.no_op_harness);
    assert!(!ready.live_write_enabled);
    assert!(!ready.mutating_runtime_enabled);

    let missing_timeout = high_risk_guarded_live_readiness_executor(
        "render.direct_scanout",
        &temp_config,
        &guard,
        false,
        true,
    );
    assert_eq!(
        missing_timeout.status,
        HighRiskLiveReadinessStatus::RecoveryProofMissing
    );
    assert!(missing_timeout
        .required_manual_steps
        .iter()
        .any(|step| step.contains("dead-man timeout")));

    let blocked_guard = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::HighRiskDisplayWrite,
        ControlledLiveTestGuardRequest {
            out_of_band_recovery_recorded: false,
            ..complete_live_guard_request(temp_config)
        },
    );
    let blocked = high_risk_guarded_live_readiness_executor(
        "render.direct_scanout",
        "/home/kyo/.config/hypr/hyprland.conf",
        &blocked_guard,
        true,
        true,
    );
    assert_eq!(
        blocked.status,
        HighRiskLiveReadinessStatus::RealConfigRefused
    );
    assert!(!blocked.live_write_enabled);
}

#[test]
fn structured_family_model_keeps_bind_entries_read_only() {
    let root = temp_root("structured");
    let config = root.join("hyprland.conf");
    write_file(
        &config,
        "bind = SUPER, Return, exec, foot\nmonitor = eDP-1,preferred,auto,1\n",
    );

    let model = structured_family_model(&config, "hl.bind").expect("model should build");

    assert_eq!(model.family_id, "hl.bind");
    assert_eq!(model.entries.len(), 1);
    assert_eq!(model.entries[0].parsed_key, "bind");
    assert!(!model.editor_enabled);
    assert!(!model.production_write_enabled);
    assert!(!model.lossless_render_proven);
}

#[test]
fn structured_family_review_keeps_repeated_bind_entries_disabled_and_lossless() {
    let root = temp_root("structured-review");
    let config = root.join("hyprland.conf");
    write_file(
        &config,
        "# keep this comment before binds\nbind = SUPER, Return, exec, foot\nbind = SUPER, Q, killactive\n",
    );
    let model = structured_family_model(&config, "hl.bind").expect("model should build");

    let review =
        structured_family_review(&model, Some("bind = SUPER, Space, exec, wofi --show drun"));

    assert_eq!(review.family_id, "hl.bind");
    assert_eq!(review.entries.len(), 2);
    assert!(review.proposed_edit.as_ref().expect("candidate").accepted);
    assert!(!review.editor_enabled);
    assert!(!review.production_write_enabled);
    assert!(review.raw_line_preservation_required);
    assert!(review.comments_order_preservation_required);
    assert_eq!(
        review.first_safe_env_write_candidate.as_deref(),
        Some("hl.bind single-line replacement after lossless render proof")
    );

    let invalid = structured_family_review(&model, Some("monitor = eDP-1,preferred,auto,1"));
    assert!(!invalid.proposed_edit.as_ref().expect("candidate").accepted);
    assert!(invalid
        .invalid_input_reasons
        .iter()
        .any(|reason| reason.contains("must start with bind")));
}

#[test]
fn structured_bind_lossless_render_and_safe_env_edit_proof_preserve_comments_and_order() {
    let root = temp_root("structured-bind-edit");
    let config = root.join("hyprland.conf");
    write_file(
        &config,
        "# keep before\nbind = SUPER, Return, exec, foot\n# keep middle\nbind = SUPER, Q, killactive\n",
    );
    let model = structured_family_model(&config, "hl.bind").expect("model should build");
    assert_eq!(
        render_structured_entry_lossless(&model.entries[0]),
        "bind = SUPER, Return, exec, foot"
    );

    let proof = edit_structured_bind_safe_env(
        &config,
        2,
        "bind = SUPER, Return, exec, foot",
        "bind = SUPER, Return, exec, kitty",
    );
    assert_eq!(proof.status, StructuredBindEditStatus::Succeeded);
    assert!(proof.comments_and_order_preserved);
    assert!(proof.reread_verified);
    assert!(!proof.production_write_enabled);
    assert!(!proof.real_config_touched);
    let updated = fs::read_to_string(config).expect("config should read");
    assert!(
        updated.starts_with("# keep before\nbind = SUPER, Return, exec, kitty\n# keep middle\n")
    );
}

#[test]
fn structured_bind_safe_env_edit_blocks_stale_line_invalid_input_and_real_paths() {
    let root = temp_root("structured-bind-blocked");
    let config = root.join("hyprland.conf");
    write_file(&config, "bind = SUPER, Return, exec, foot\n");

    let stale = edit_structured_bind_safe_env(
        &config,
        1,
        "bind = SUPER, Space, exec, wofi",
        "bind = SUPER, Return, exec, kitty",
    );
    assert_eq!(stale.status, StructuredBindEditStatus::Blocked);
    assert!(stale
        .errors
        .iter()
        .any(|error| error.contains("no longer matches")));

    let invalid = edit_structured_bind_safe_env(
        &config,
        1,
        "bind = SUPER, Return, exec, foot",
        "monitor = eDP-1,preferred,auto,1",
    );
    assert_eq!(invalid.status, StructuredBindEditStatus::Blocked);
    assert!(invalid
        .errors
        .iter()
        .any(|error| error.contains("must start with bind")));

    let real_path = PathBuf::from("/home/kyo/.config/hypr/hyprland.conf");
    let real = edit_structured_bind_safe_env(
        real_path,
        1,
        "bind = SUPER, Return, exec, foot",
        "bind = SUPER, Return, exec, kitty",
    );
    assert_eq!(real.status, StructuredBindEditStatus::Blocked);
    assert!(!real.real_config_touched);
}

#[test]
fn structured_bind_guarded_executor_edits_and_restores_selected_line() {
    let root = temp_root("structured-bind-guarded");
    let config = root.join("hyprland.conf");
    let original = "# keep before\nbind = SUPER, Return, exec, foot\n# keep middle\nbind = SUPER, Q, killactive\n";
    write_file(&config, original);
    let guard = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::StructuredWrite,
        complete_live_guard_request(config.clone()),
    );

    let report = execute_structured_bind_guarded_temp(
        &config,
        2,
        "bind = SUPER, Return, exec, foot",
        "bind = SUPER, Return, exec, kitty",
        &guard,
        false,
        false,
    );
    assert_eq!(
        report.status,
        GuardedTempExecutionStatus::SucceededAndRestored
    );
    assert!(report.mutation_verified);
    assert!(report.restore_succeeded);
    assert_eq!(
        fs::read_to_string(&config).expect("config should read"),
        original
    );
    assert!(!report.production_write_enabled);

    let invalid = execute_structured_bind_guarded_temp(
        &config,
        2,
        "bind = SUPER, Return, exec, foot",
        "monitor = eDP-1,preferred,auto,1",
        &guard,
        false,
        false,
    );
    assert_eq!(invalid.status, GuardedTempExecutionStatus::Blocked);

    let stale = execute_structured_bind_guarded_temp(
        &config,
        2,
        "bind = SUPER, Space, exec, wofi",
        "bind = SUPER, Return, exec, kitty",
        &guard,
        false,
        false,
    );
    assert_eq!(stale.status, GuardedTempExecutionStatus::Blocked);

    let real = execute_structured_bind_guarded_temp(
        "/home/kyo/.config/hypr/hyprland.conf",
        1,
        "bind = SUPER, Return, exec, foot",
        "bind = SUPER, Return, exec, kitty",
        &guard,
        false,
        false,
    );
    assert_eq!(real.status, GuardedTempExecutionStatus::Blocked);
}

#[test]
fn structured_edit_candidate_blocks_invalid_input_and_keeps_production_disabled() {
    let accepted =
        validate_structured_edit_candidate("hl.bind", "bind = SUPER, Return, exec, foot");
    assert!(accepted.accepted);
    assert!(!accepted.production_write_enabled);
    assert!(accepted.lossless_render_required);

    let rejected =
        validate_structured_edit_candidate("hl.bind", "monitor = eDP-1,preferred,auto,1");
    assert!(!rejected.accepted);
    assert!(!rejected.production_write_enabled);
    assert!(rejected
        .errors
        .iter()
        .any(|error| error.contains("must start with bind")));

    let multiline = validate_structured_edit_candidate("hl.bind", "bind = A\nbind = B");
    assert!(!multiline.accepted);
    assert!(multiline
        .errors
        .iter()
        .any(|error| error.contains("single-line")));
}

#[cfg(unix)]
#[test]
fn profile_switch_safe_env_switches_and_restores_temp_symlink_only() {
    use std::os::unix::fs::symlink;

    let root = temp_root("profile-switch");
    let profiles = root.join("profiles");
    fs::create_dir_all(&profiles).expect("profiles should create");
    let desktop = profiles.join("desktop.conf");
    let gaming = profiles.join("gaming.conf");
    write_file(&desktop, "general:layout = dwindle\n");
    write_file(&gaming, "general:layout = master\n");
    let current = profiles.join("current.conf");
    symlink(&desktop, &current).expect("current symlink should create");

    let report = switch_profile_symlink_safe_env(&root, &current, &gaming, false);

    assert_eq!(
        report.status,
        hyprland_settings::future_capability::ProfileSwitchStatus::Succeeded
    );
    assert_eq!(
        fs::read_link(&current).expect("current should read"),
        desktop
    );
    assert!(!report.production_switch_enabled);
    assert!(!report.real_config_touched);
    assert!(!report.runtime_touched);
}

#[cfg(unix)]
#[test]
fn profile_switch_guarded_temp_requires_guard_and_restores_symlink() {
    use std::os::unix::fs::symlink;

    let root = temp_root("profile-guarded-switch");
    let profiles = root.join("profiles");
    fs::create_dir_all(&profiles).expect("profiles should create");
    let desktop = profiles.join("desktop.conf");
    let gaming = profiles.join("gaming.conf");
    write_file(&desktop, "general:layout = dwindle\n");
    write_file(&gaming, "general:layout = master\n");
    let current = profiles.join("current.conf");
    symlink(&desktop, &current).expect("current symlink should create");

    let blocked_guard = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::ProfileSwitch,
        ControlledLiveTestGuardRequest {
            symlink_targets_recorded: false,
            ..complete_live_guard_request(current.clone())
        },
    );
    let blocked =
        switch_profile_symlink_guarded_temp(&root, &current, &gaming, &blocked_guard, false);
    assert_eq!(
        blocked.status,
        hyprland_settings::future_capability::ProfileSwitchStatus::Blocked
    );
    assert_eq!(
        fs::read_link(&current).expect("current should read"),
        desktop
    );

    let guard = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::ProfileSwitch,
        complete_live_guard_request(current.clone()),
    );
    let report = switch_profile_symlink_guarded_temp(&root, &current, &gaming, &guard, false);
    assert_eq!(
        report.status,
        hyprland_settings::future_capability::ProfileSwitchStatus::Succeeded
    );
    assert_eq!(
        fs::read_link(&current).expect("current should read"),
        desktop
    );
    assert!(!report.production_switch_enabled);
    assert!(!report.real_config_touched);
}

#[cfg(unix)]
#[test]
fn profile_switch_review_and_forced_restore_failure_stay_disabled() {
    use std::os::unix::fs::symlink;

    let root = temp_root("profile-failure");
    let profiles = root.join("profiles");
    fs::create_dir_all(&profiles).expect("profiles should create");
    let desktop = profiles.join("desktop.conf");
    let gaming = profiles.join("gaming.conf");
    write_file(&desktop, "general:layout = dwindle\n");
    write_file(&gaming, "general:layout = master\n");
    let current = profiles.join("current.conf");
    symlink(&desktop, &current).expect("current symlink should create");

    let review = disabled_profile_switch_review(&current, Some(desktop.clone()), &gaming);
    assert!(!review.production_switch_enabled);
    assert!(!review.reload_after_switch_enabled);
    assert!(review
        .review_lines
        .iter()
        .any(|line| line.contains("Real profile files")));

    let report = switch_profile_symlink_safe_env(&root, &current, &gaming, true);
    assert_eq!(
        report.status,
        hyprland_settings::future_capability::ProfileSwitchStatus::RestoreFailed
    );
    assert!(!report.production_switch_enabled);
    assert!(!report.real_config_touched);
    assert!(!report.runtime_touched);
}

#[cfg(unix)]
#[test]
fn profile_switch_selection_review_tracks_selected_target_but_keeps_disabled() {
    let root = temp_root("profile-selection-review");
    let symlink = root.join("profiles/current.conf");
    let desktop = root.join("profiles/desktop.conf");
    let gaming = root.join("profiles/gaming.conf");

    let no_selection =
        disabled_profile_switch_selection_review(&symlink, Some(desktop.clone()), None, None);
    assert!(no_selection.selected_target_profile.is_none());
    assert!(!no_selection.confirmation_enabled);
    assert!(!no_selection.production_switch_enabled);
    assert!(!no_selection.reload_after_switch_enabled);
    assert!(no_selection
        .review_lines
        .iter()
        .any(|line| line.contains("No target profile")));

    let selected = disabled_profile_switch_selection_review(
        &symlink,
        Some(desktop.clone()),
        Some(desktop.clone()),
        Some(gaming.clone()),
    );
    assert_eq!(selected.symlink_path, symlink);
    assert_eq!(selected.current_profile.as_ref(), Some(&desktop));
    assert_eq!(selected.resolved_current_target.as_ref(), Some(&desktop));
    assert_eq!(selected.selected_target_profile.as_ref(), Some(&gaming));
    assert!(!selected.confirmation_enabled);
    assert!(!selected.production_switch_enabled);
    assert!(!selected.reload_after_switch_enabled);
}

#[cfg(unix)]
#[test]
fn profile_target_approval_review_blocks_missing_and_real_session_targets() {
    let root = temp_root("profile-target-review");
    let existing = root.join("profiles/gaming.conf");
    write_file(&existing, "general:layout = master\n");

    let none = profile_target_approval_review(&root, None);
    assert_eq!(none.readiness, ProfileTargetReadiness::NoSelection);
    assert!(!none.production_switch_enabled);

    let missing = profile_target_approval_review(&root, Some(root.join("profiles/missing.conf")));
    assert_eq!(missing.readiness, ProfileTargetReadiness::TargetMissing);
    assert!(!missing.real_session_allowed);

    let safe = profile_target_approval_review(&root, Some(existing));
    assert_eq!(safe.readiness, ProfileTargetReadiness::SafeEnvReviewOnly);
    assert!(!safe.production_switch_enabled);

    let real = profile_target_approval_review(
        &root,
        Some(PathBuf::from("/home/kyo/.config/hypr/profiles/gaming.conf")),
    );
    assert_eq!(real.readiness, ProfileTargetReadiness::TargetOutsideSafeEnv);
    assert!(!real.real_session_allowed);
}

#[test]
fn runtime_boundary_is_dry_run_only_and_never_executes_commands() {
    let mut executor = RuntimeDryRunExecutor::default();
    let reload = executor.evaluate(RuntimeAction::Reload);
    let status = executor.evaluate(RuntimeAction::Status {
        query: "version".to_string(),
    });

    assert!(reload.would_mutate_runtime);
    assert!(!reload.accepted_by_allowlist);
    assert!(!reload.real_command_executed);
    assert!(!reload.production_runtime_enabled);
    assert!(!status.would_mutate_runtime);
    assert!(status.accepted_by_allowlist);
    assert!(!status.real_command_executed);
    assert_eq!(executor.recorded_actions.len(), 2);

    let reload_policy = runtime_action_policy(RuntimeAction::Reload);
    assert!(!reload_policy.allowlisted_for_real_execution);
    assert!(reload_policy.dry_run_allowed);
    assert!(!reload_policy.production_runtime_enabled);
    assert!(reload_policy.reason.contains("disabled"));
}

#[test]
fn runtime_action_review_records_policy_and_log_without_execution() {
    let reload_review = runtime_action_review(RuntimeAction::Reload);
    assert!(!reload_review.policy.allowlisted_for_real_execution);
    assert!(reload_review.policy.dry_run_allowed);
    assert!(reload_review.dry_run_result.would_mutate_runtime);
    assert!(!reload_review.production_execution_enabled);
    assert!(!reload_review.real_command_executed);
    assert!(!reload_review.dry_run_result.real_command_executed);
    assert_eq!(reload_review.execution_log.len(), 1);

    let status_review = runtime_action_review(RuntimeAction::Status {
        query: "version".to_string(),
    });
    assert!(!status_review.policy.allowlisted_for_real_execution);
    assert!(status_review.policy.dry_run_allowed);
    assert!(!status_review.dry_run_result.would_mutate_runtime);
    assert!(!status_review.production_execution_enabled);
    assert!(!status_review.real_command_executed);
    assert!(!status_review.dry_run_result.real_command_executed);
}

#[test]
fn runtime_command_risk_classifies_status_reload_keyword_and_dispatch_without_execution() {
    assert_eq!(
        runtime_command_risk(&RuntimeAction::Status {
            query: "version".to_string()
        }),
        RuntimeCommandRisk::ReadOnlyStatus
    );
    assert_eq!(
        runtime_command_risk(&RuntimeAction::Reload),
        RuntimeCommandRisk::MutatingReload
    );
    assert_eq!(
        runtime_command_risk(&RuntimeAction::Keyword {
            key: "general:gaps_in".to_string(),
            value: "4".to_string()
        }),
        RuntimeCommandRisk::MutatingKeyword
    );
    assert_eq!(
        runtime_command_risk(&RuntimeAction::Dispatch {
            command: "workspace 1".to_string()
        }),
        RuntimeCommandRisk::MutatingDispatch
    );
}

#[test]
fn runtime_guarded_executor_keeps_status_read_only_and_requires_restore_for_mutation() {
    let root = temp_root("runtime-guarded");
    let target = root.join("hyprland.conf");
    write_file(&target, "general:gaps_in = 3\n");
    let guard = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::RuntimeMutation,
        complete_live_guard_request(target.clone()),
    );

    let status = runtime_guarded_executor(
        RuntimeAction::Status {
            query: "version".to_string(),
        },
        &guard,
        None,
    );
    assert!(status.guard_allowed);
    assert!(!status.real_command_executed);
    assert!(!status.runtime_touched);

    let keyword = runtime_guarded_executor(
        RuntimeAction::Keyword {
            key: "general:gaps_in".to_string(),
            value: "6".to_string(),
        },
        &guard,
        Some("3"),
    );
    assert!(keyword.guard_allowed);
    assert_eq!(
        keyword.restore_command.as_deref(),
        Some("hyprctl keyword general:gaps_in 3")
    );
    assert!(!keyword.real_command_executed);
    assert!(!keyword.runtime_touched);

    let no_snapshot = runtime_guarded_executor(
        RuntimeAction::Keyword {
            key: "general:gaps_in".to_string(),
            value: "6".to_string(),
        },
        &guard,
        None,
    );
    assert!(!no_snapshot.guard_allowed);
    assert!(no_snapshot
        .errors
        .iter()
        .any(|error| error.contains("restore command")));

    let blocked_guard = hyprland_settings::future_capability::controlled_live_test_guard_review(
        ControlledLiveTestKind::RuntimeMutation,
        ControlledLiveTestGuardRequest {
            read_only_runtime_snapshot_recorded: false,
            ..complete_live_guard_request(target)
        },
    );
    let blocked = runtime_guarded_executor(RuntimeAction::Reload, &blocked_guard, Some("snapshot"));
    assert!(!blocked.guard_allowed);
    assert!(!blocked.real_command_executed);
}

#[test]
fn hyprland_0554_migration_assessment_keeps_0552_default_without_trusted_export() {
    let assessment = assess_hyprland_version_migration("0.55.4", false);

    assert_eq!(assessment.current_default_version, "0.55.2");
    assert_eq!(assessment.requested_version, "0.55.4");
    assert!(!assessment.migration_activated);
    assert!(!assessment.production_default_changed);
    assert!(assessment
        .blockers
        .iter()
        .any(|blocker| blocker.contains("trusted official export")));

    let bundle = current_v0552_data_bundle();
    assert_eq!(bundle.version, "0.55.2");
    assert_eq!(bundle.readable_rows, 341);
    assert_eq!(bundle.writable_rows, 341);
    assert_eq!(bundle.blocked_rows, 0);
    assert!(bundle.default_model);

    let review = disabled_migration_review("0.55.4");
    assert_eq!(review.current_default.version, "0.55.2");
    assert!(!review.migration_enabled);
    assert!(review
        .review_lines
        .iter()
        .any(|line| line.contains("newer runtime package is not enough")));
}

#[test]
fn migration_comparison_review_keeps_0552_default_until_trusted_export_proof_exists() {
    let blocked = migration_comparison_review("0.55.4", false);
    assert_eq!(blocked.current_default.version, "0.55.2");
    assert_eq!(blocked.requested_version, "0.55.4");
    assert!(blocked.requested_bundle.is_none());
    assert!(!blocked.trusted_source_requirement_met);
    assert!(!blocked.migration_enabled);
    assert!(!blocked.production_default_changed);
    assert!(blocked
        .missing_proof
        .iter()
        .any(|proof| proof.contains("trusted official export")));
    assert!(blocked
        .missing_proof
        .iter()
        .any(|proof| proof.contains("GTK safe-env evidence")));

    let current = migration_comparison_review("0.55.2", true);
    assert_eq!(current.current_default.version, "0.55.2");
    assert!(current.requested_bundle.is_some());
    assert!(current.trusted_source_requirement_met);
    assert!(!current.migration_enabled);
    assert!(!current.production_default_changed);
    assert!(current.missing_proof.is_empty());
}

#[test]
fn trusted_export_requirement_blocks_0554_until_all_required_inputs_exist() {
    let blocked = trusted_export_requirement("0.55.4", true, false, false, false);
    assert!(!blocked.can_activate);
    assert!(blocked
        .missing_inputs
        .iter()
        .any(|input| input.contains("row-count diff")));
    assert!(blocked
        .missing_inputs
        .iter()
        .any(|input| input.contains("safe-env GTK evidence")));

    let complete = trusted_export_requirement("0.55.4", true, true, true, true);
    assert!(complete.can_activate);
    assert!(complete.missing_inputs.is_empty());

    let current = trusted_export_requirement("0.55.2", false, false, false, false);
    assert!(current.can_activate);
}

#[test]
fn local_hyprland_0554_evidence_is_advisory_until_trusted_bundle_is_complete() {
    let partial = local_hyprland_version_evidence(
        "0.55.4",
        Some("hyprland 0.55.4-1"),
        Some("Hyprland 0.55.4"),
        false,
        false,
        false,
        false,
        false,
    );
    assert!(!partial.activation_allowed);
    assert!(!partial.production_default_changed);
    assert!(partial
        .missing_inputs
        .iter()
        .any(|input| input.contains("trusted official export")));
    assert!(partial
        .missing_inputs
        .iter()
        .any(|input| input.contains("explicit user approval")));
    assert!(partial
        .evidence_lines
        .iter()
        .any(|line| line.contains("advisory")));

    let missing_local_versions =
        local_hyprland_version_evidence("0.55.4", None, None, true, true, true, true, true);
    assert!(!missing_local_versions.activation_allowed);
    assert!(missing_local_versions
        .missing_inputs
        .iter()
        .any(|input| input.contains("package version")));
    assert!(missing_local_versions
        .missing_inputs
        .iter()
        .any(|input| input.contains("runtime binary")));

    let complete = local_hyprland_version_evidence(
        "0.55.4",
        Some("hyprland 0.55.4-1"),
        Some("Hyprland 0.55.4"),
        true,
        true,
        true,
        true,
        true,
    );
    assert!(complete.activation_allowed);
    assert!(!complete.production_default_changed);

    let current =
        local_hyprland_version_evidence("0.55.2", None, None, false, false, false, false, false);
    assert!(current.activation_allowed);
    assert!(!current.production_default_changed);
    assert!(current.missing_inputs.is_empty());
}

#[test]
fn runtime_production_gate_requires_readonly_evidence_snapshot_restore_and_default_disabled_flag() {
    let missing_socket = runtime_production_gate_review(
        RuntimeAction::Status {
            query: "version".to_string(),
        },
        false,
        None,
        None,
        false,
        false,
    );
    assert_eq!(
        missing_socket.status,
        RuntimeProductionGateStatus::ReadOnlyEvidenceMissing
    );
    assert!(!missing_socket.production_runtime_enabled);

    let keyword_missing_snapshot = runtime_production_gate_review(
        RuntimeAction::Keyword {
            key: "general:gaps_in".to_string(),
            value: "5".to_string(),
        },
        true,
        None,
        None,
        true,
        false,
    );
    assert_eq!(
        keyword_missing_snapshot.status,
        RuntimeProductionGateStatus::RestoreCommandMissing
    );
    assert!(!keyword_missing_snapshot.production_runtime_enabled);

    let keyword_ready = runtime_production_gate_review(
        RuntimeAction::Keyword {
            key: "general:gaps_in".to_string(),
            value: "5".to_string(),
        },
        true,
        Some("4"),
        None,
        true,
        false,
    );
    assert_eq!(
        keyword_ready.status,
        RuntimeProductionGateStatus::ReadyButDefaultDisabled
    );
    assert_eq!(
        keyword_ready.restore_command.as_deref(),
        Some("hyprctl keyword general:gaps_in 4")
    );
    assert!(!keyword_ready.production_runtime_enabled);

    let dispatch_missing_plan = runtime_production_gate_review(
        RuntimeAction::Dispatch {
            command: "workspace 1".to_string(),
        },
        true,
        Some("prior workspace"),
        None,
        true,
        false,
    );
    assert_eq!(
        dispatch_missing_plan.status,
        RuntimeProductionGateStatus::RecoveryPlanMissing
    );
}

#[test]
fn runtime_live_restore_proof_blocks_failed_readonly_and_can_reach_ready_states_without_enabling_production(
) {
    let action = RuntimeAction::Keyword {
        key: "general:gaps_in".to_string(),
        value: "6".to_string(),
    };
    let blocked = runtime_live_restore_proof_review(
        action.clone(),
        false,
        Some("5"),
        Some("6"),
        None,
        None,
        false,
    );
    assert_eq!(
        blocked.status,
        RuntimeLiveRestoreStatus::ReadOnlyEvidenceMissing
    );
    assert!(!blocked.production_runtime_enabled);
    assert!(!blocked.real_command_executed);

    let ready_without_mutation = runtime_live_restore_proof_review(
        action.clone(),
        true,
        Some("5"),
        Some("6"),
        None,
        None,
        false,
    );
    assert_eq!(
        ready_without_mutation.status,
        RuntimeLiveRestoreStatus::ReadyButDefaultDisabled
    );
    assert!(!ready_without_mutation.runtime_touched);

    let restored = runtime_live_restore_proof_review(
        action,
        true,
        Some("5"),
        Some("6"),
        Some("6"),
        Some("5"),
        true,
    );
    assert_eq!(restored.status, RuntimeLiveRestoreStatus::LiveRestoreProven);
    assert!(restored.runtime_touched);
    assert!(restored.restored);
    assert!(!restored.production_runtime_enabled);
}

#[test]
fn runtime_socket_diagnosis_distinguishes_sandbox_permission_and_real_readonly_success() {
    let socket = PathBuf::from(
        "/run/user/1000/hypr/a0136d8c04687bb36eb8a28eb9d1ff92aea99704_1781857006_1638495299/.socket.sock",
    );
    let sandboxed = runtime_socket_diagnosis(
        runtime_readonly_evidence(false, Some("Couldn't set socket timeout (2)")),
        vec![RuntimeSocketCandidate {
            signature: "a0136d8c04687bb36eb8a28eb9d1ff92aea99704_1781857006_1638495299".to_string(),
            socket_path: socket.clone(),
            exists: true,
            hyprctl_version_succeeded: false,
            raw_socket_succeeded: false,
            error: Some("Operation not permitted".to_string()),
        }],
        RuntimeDirectIpcReadOnlyEvidence {
            socket_path: socket.clone(),
            attempted: true,
            succeeded: false,
            error: Some("Operation not permitted".to_string()),
        },
        false,
        false,
    );
    assert_eq!(
        sandboxed.status,
        RuntimeSocketDiagnosisStatus::PermissionMismatch
    );
    assert!(!sandboxed.mutation_allowed);

    let real_session = runtime_socket_diagnosis(
        runtime_readonly_evidence(true, None),
        vec![RuntimeSocketCandidate {
            signature: "a0136d8c04687bb36eb8a28eb9d1ff92aea99704_1781857006_1638495299".to_string(),
            socket_path: socket.clone(),
            exists: true,
            hyprctl_version_succeeded: true,
            raw_socket_succeeded: true,
            error: None,
        }],
        RuntimeDirectIpcReadOnlyEvidence {
            socket_path: socket,
            attempted: false,
            succeeded: false,
            error: None,
        },
        true,
        true,
    );
    assert_eq!(
        real_session.status,
        RuntimeSocketDiagnosisStatus::HyprctlReadOnlySucceeded
    );
    assert!(real_session.mutation_allowed);
}

#[test]
fn runtime_socket_diagnosis_records_raw_socket_success_with_hyprctl_failure() {
    let socket = PathBuf::from("/run/user/1000/hypr/signature/.socket.sock");
    let diagnosis = runtime_socket_diagnosis(
        runtime_readonly_evidence(false, Some("hyprctl failed")),
        vec![RuntimeSocketCandidate {
            signature: "signature".to_string(),
            socket_path: socket.clone(),
            exists: true,
            hyprctl_version_succeeded: false,
            raw_socket_succeeded: true,
            error: None,
        }],
        RuntimeDirectIpcReadOnlyEvidence {
            socket_path: socket,
            attempted: true,
            succeeded: true,
            error: None,
        },
        true,
        true,
    );
    assert_eq!(
        diagnosis.status,
        RuntimeSocketDiagnosisStatus::RawSocketSucceededHyprctlFailed
    );
    assert!(!diagnosis.mutation_allowed);
}

#[test]
fn runtime_live_restore_attempt_records_failed_mutation_syntax_without_enabling_production() {
    let action = RuntimeAction::Keyword {
        key: "general:gaps_in".to_string(),
        value: "6".to_string(),
    };
    let unparseable = runtime_live_restore_attempt_review(
        action.clone(),
        true,
        None,
        Some("6"),
        Some("hyprctl eval 'general:gaps_in = 5'"),
        Some("hyprctl eval 'general:gaps_in = 6'"),
        false,
        None,
        None,
    );
    assert_eq!(
        unparseable.status,
        RuntimeLiveRestoreStatus::PriorValueMissing
    );
    assert!(!unparseable.production_runtime_enabled);

    let failed_keyword = runtime_live_restore_attempt_review(
        action,
        true,
        Some("5"),
        Some("6"),
        Some("hyprctl keyword general:gaps_in 5"),
        Some("hyprctl keyword general:gaps_in 6"),
        false,
        Some("5"),
        Some("5"),
    );
    assert_eq!(
        failed_keyword.status,
        RuntimeLiveRestoreStatus::LiveRestoreBlocked
    );
    assert!(!failed_keyword.real_command_executed);
    assert!(!failed_keyword.runtime_touched);
    assert!(!failed_keyword.production_runtime_enabled);
}

#[test]
fn runtime_mutation_syntax_evidence_records_proven_lua_config_restore_without_enabling_production()
{
    let keyword_failure = RuntimeMutationSyntaxCandidate {
        syntax_name: "legacy keyword".to_string(),
        command_pair: RuntimeMutationCommandPair {
            mutation_command: "hyprctl keyword general:gaps_in 6".to_string(),
            restore_command: "hyprctl keyword general:gaps_in 5".to_string(),
        },
        status: RuntimeMutationSyntaxStatus::FailedBeforeValueChange,
        error: Some("keyword can't work with non-legacy parsers. Use eval.".to_string()),
        post_mutation_value: Some("5".to_string()),
        post_restore_value: Some("5".to_string()),
    };
    let assignment_eval_failure = RuntimeMutationSyntaxCandidate {
        syntax_name: "assignment eval".to_string(),
        command_pair: RuntimeMutationCommandPair {
            mutation_command: "hyprctl eval 'general:gaps_in = 6'".to_string(),
            restore_command: "hyprctl eval 'general:gaps_in = 5'".to_string(),
        },
        status: RuntimeMutationSyntaxStatus::FailedBeforeValueChange,
        error: Some("function arguments expected near '='".to_string()),
        post_mutation_value: Some("5".to_string()),
        post_restore_value: Some("5".to_string()),
    };
    let lua_config_success = RuntimeMutationSyntaxCandidate {
        syntax_name: "lua hl.config eval".to_string(),
        command_pair: RuntimeMutationCommandPair {
            mutation_command: "hyprctl eval 'hl.config({ general = { gaps_in = 6 } })'".to_string(),
            restore_command: "hyprctl eval 'hl.config({ general = { gaps_in = 5 } })'".to_string(),
        },
        status: RuntimeMutationSyntaxStatus::MutatedAndRestored,
        error: None,
        post_mutation_value: Some("6".to_string()),
        post_restore_value: Some("5".to_string()),
    };
    let evidence = runtime_eval_syntax_evidence(
        "general:gaps_in",
        "5",
        "6",
        vec![
            keyword_failure,
            assignment_eval_failure,
            lua_config_success.clone(),
        ],
    );
    assert_eq!(
        evidence.successful_syntax,
        Some("lua hl.config eval".to_string())
    );
    assert!(evidence.live_restore_proven);
    assert!(evidence.runtime_left_restored);
    assert!(!evidence.production_runtime_enabled);

    let proof = runtime_live_restore_attempt_review(
        RuntimeAction::Keyword {
            key: "general:gaps_in".to_string(),
            value: "6".to_string(),
        },
        true,
        Some("5"),
        Some("6"),
        Some(lua_config_success.command_pair.restore_command.as_str()),
        Some(lua_config_success.command_pair.mutation_command.as_str()),
        true,
        Some("6"),
        Some("5"),
    );
    assert_eq!(proof.status, RuntimeLiveRestoreStatus::LiveRestoreProven);
    assert!(proof.real_command_executed);
    assert!(proof.runtime_touched);
    assert!(proof.restored);
    assert!(!proof.production_runtime_enabled);
}

#[test]
fn runtime_live_restore_approval_review_consumes_proof_but_keeps_production_disabled() {
    let action = RuntimeAction::Keyword {
        key: "general:gaps_in".to_string(),
        value: "6".to_string(),
    };
    let syntax = proven_runtime_eval_syntax_evidence();
    let proof = proven_runtime_live_restore_proof();
    let command = "hyprctl eval 'hl.config({ general = { gaps_in = 6 } })'";

    let missing_proof =
        runtime_live_restore_approval_review(action.clone(), None, Some(&syntax), None, false);
    assert_eq!(
        missing_proof.status,
        RuntimeApprovalReviewStatus::MissingLiveRestoreProof
    );
    assert!(!missing_proof.production_runtime_enabled);

    let failed_proof = runtime_live_restore_attempt_review(
        action.clone(),
        true,
        Some("5"),
        Some("6"),
        Some("hyprctl eval 'hl.config({ general = { gaps_in = 5 } })'"),
        Some(command),
        true,
        Some("6"),
        Some("4"),
    );
    let failed_restore = runtime_live_restore_approval_review(
        action.clone(),
        Some(&failed_proof),
        Some(&syntax),
        None,
        false,
    );
    assert_eq!(
        failed_restore.status,
        RuntimeApprovalReviewStatus::FailedLiveRestoreProof
    );

    let mut wrong_setting = syntax.clone();
    wrong_setting.setting = "general:gaps_out".to_string();
    let wrong_setting_review = runtime_live_restore_approval_review(
        action.clone(),
        Some(&proof),
        Some(&wrong_setting),
        None,
        false,
    );
    assert_eq!(
        wrong_setting_review.status,
        RuntimeApprovalReviewStatus::WrongSetting
    );

    let mut wrong_restore = proof.clone();
    wrong_restore.restore_command = Some("hyprctl keyword general:gaps_in 5".to_string());
    let wrong_restore_review = runtime_live_restore_approval_review(
        action.clone(),
        Some(&wrong_restore),
        Some(&syntax),
        None,
        false,
    );
    assert_eq!(
        wrong_restore_review.status,
        RuntimeApprovalReviewStatus::RestoreCommandMismatch
    );

    let missing_approval = runtime_live_restore_approval_review(
        action.clone(),
        Some(&proof),
        Some(&syntax),
        None,
        false,
    );
    assert_eq!(
        missing_approval.status,
        RuntimeApprovalReviewStatus::MissingApproval
    );

    let wrong_scope = approval_request(
        ApprovalScope::DuplicateReplacement,
        None,
        Some(command),
        false,
        true,
    );
    let wrong_scope_review = runtime_live_restore_approval_review(
        action.clone(),
        Some(&proof),
        Some(&syntax),
        Some(&wrong_scope),
        false,
    );
    assert_eq!(
        wrong_scope_review.status,
        RuntimeApprovalReviewStatus::WrongApprovalScope
    );

    let mut expired = approval_request(
        ApprovalScope::RuntimeKeyword,
        None,
        Some(command),
        false,
        true,
    );
    expired.current_tick = 100;
    let expired_review = runtime_live_restore_approval_review(
        action.clone(),
        Some(&proof),
        Some(&syntax),
        Some(&expired),
        false,
    );
    assert_eq!(
        expired_review.status,
        RuntimeApprovalReviewStatus::ApprovalExpired
    );

    let approved = approval_request(
        ApprovalScope::RuntimeKeyword,
        None,
        Some(command),
        false,
        true,
    );
    let approved_review = runtime_live_restore_approval_review(
        action.clone(),
        Some(&proof),
        Some(&syntax),
        Some(&approved),
        false,
    );
    assert_eq!(
        approved_review.status,
        RuntimeApprovalReviewStatus::ApprovedButDefaultDisabled
    );
    assert!(approved_review
        .live_restore_evidence
        .as_ref()
        .is_some_and(|evidence| evidence.restoration_verified));
    assert!(!approved_review.production_runtime_enabled);

    let approved_with_flag = runtime_live_restore_approval_review(
        action,
        Some(&proof),
        Some(&syntax),
        Some(&approved),
        true,
    );
    assert_eq!(
        approved_with_flag.status,
        RuntimeApprovalReviewStatus::ApprovedButDefaultDisabled
    );
    assert!(!approved_with_flag.production_runtime_enabled);
    assert!(approved_with_flag
        .blockers
        .iter()
        .any(|blocker| blocker.contains("not wired")));
}

#[test]
fn runtime_approval_evidence_projection_includes_proof_without_enabling_production() {
    let evidence = proven_runtime_approval_evidence_summary();
    let lines = evidence.user_facing_lines();
    for expected in [
        "Runtime approval review",
        "Runtime changes are not enabled yet.",
        "This setting has a proven live-restore test.",
        "Production runtime/reload remains disabled.",
        "Setting: general:gaps_in",
        "Prior value: 5",
        "Temporary test value: 6",
        "Mutation command: hyprctl eval 'hl.config({ general = { gaps_in = 6 } })'",
        "Restore command: hyprctl eval 'hl.config({ general = { gaps_in = 5 } })'",
        "Post-mutation readback: css gap data: 6 6 6 6; set: true",
        "Post-restore readback: css gap data: 5 5 5 5; set: true",
        "Approval status: Approved but default-disabled",
        "Production runtime/reload: Disabled",
    ] {
        assert!(
            lines.iter().any(|line| line == expected),
            "missing runtime approval evidence line: {expected}"
        );
    }
    assert!(!evidence.production_runtime_enabled);
}

#[test]
fn disabled_future_approval_card_projections_cover_all_remaining_gates_without_enablement() {
    let cards = disabled_future_approval_card_projections();
    assert_eq!(cards.len(), 6);

    for expected_widget in [
        "hyprland-settings-source-include-approval-review-disabled",
        "hyprland-settings-duplicate-approval-review-disabled",
        "hyprland-settings-structured-approval-review-disabled",
        "hyprland-settings-profile-approval-review-disabled",
        "hyprland-settings-high-risk-approval-review-disabled",
        "hyprland-settings-0554-approval-review-disabled",
    ] {
        let card = cards
            .iter()
            .find(|card| card.widget_name == expected_widget)
            .unwrap_or_else(|| panic!("missing disabled approval card: {expected_widget}"));
        assert_eq!(card.production_status, "Disabled");
        assert!(!card.production_enabled);
        assert!(card.disabled_action_label.contains("(planned)"));
        assert!(!card.blockers.is_empty());
        assert!(card
            .user_facing_lines()
            .iter()
            .any(|line| line.contains("Production")));
    }

    let all_lines = cards
        .iter()
        .flat_map(|card| card.user_facing_lines())
        .collect::<Vec<_>>()
        .join("\n");
    for expected in [
        "Source/include approval review",
        "Production source/include insertion: Disabled",
        "Duplicate approval review",
        "Production duplicate writes: Disabled",
        "Structured hl.bind approval review",
        "Production structured writes: Disabled",
        "Profile/mode approval review",
        "Production profile switching: Disabled",
        "High-risk/display approval review",
        "Runtime live-restore proof is available for a low-risk setting.",
        "That proof is not enough to enable high-risk/display writes.",
        "Hyprland 0.55.4 migration review",
        "Current active app model: v0.55.2",
        "Migration status: Inactive",
        "Production migration activation: Disabled",
    ] {
        assert!(
            all_lines.contains(expected),
            "missing disabled approval projection line: {expected}"
        );
    }
}

#[test]
fn runtime_high_risk_and_hyprland_approval_flows_keep_production_disabled() {
    let runtime_gate = runtime_production_gate_review(
        RuntimeAction::Keyword {
            key: "general:gaps_in".to_string(),
            value: "6".to_string(),
        },
        true,
        Some("5"),
        None,
        true,
        false,
    );
    assert_eq!(
        runtime_gate.status,
        RuntimeProductionGateStatus::ReadyButDefaultDisabled
    );
    let wrong_runtime = approval_request(
        ApprovalScope::DuplicateReplacement,
        None,
        Some("hyprctl keyword general:gaps_in 6"),
        false,
        true,
    );
    assert_eq!(
        runtime_approval_flow(&runtime_gate, Some(&wrong_runtime)).status,
        ApprovalStatus::WrongScope
    );
    let runtime_request = approval_request(
        ApprovalScope::RuntimeKeyword,
        None,
        Some("hyprctl keyword general:gaps_in 6"),
        false,
        true,
    );
    let runtime_decision = runtime_approval_flow(&runtime_gate, Some(&runtime_request));
    assert_eq!(
        runtime_decision.status,
        ApprovalStatus::ReadyButDefaultDisabled
    );
    assert!(!runtime_decision.production_apply_enabled);

    let root = temp_root("high-risk-approval");
    let config = root.join("hyprland.conf");
    write_file(&config, "render:direct_scanout = false\n");
    let protocol = high_risk_live_recovery_protocol("render.direct_scanout", config, true, false);
    let high_risk_gate = high_risk_production_gate_review(
        Some(&protocol),
        true,
        true,
        true,
        true,
        true,
        true,
        false,
    );
    assert_eq!(
        high_risk_gate.status,
        HighRiskProductionGateStatus::ReadyButDefaultDisabled
    );
    let high_risk_missing = high_risk_approval_flow(&high_risk_gate, None);
    assert_eq!(high_risk_missing.status, ApprovalStatus::MissingEvidence);
    let high_risk_request = approval_request(
        ApprovalScope::HighRiskDisplayWrite,
        None,
        Some("render.direct_scanout"),
        false,
        true,
    );
    let high_risk_decision = high_risk_approval_flow(&high_risk_gate, Some(&high_risk_request));
    assert_eq!(
        high_risk_decision.status,
        ApprovalStatus::ReadyButDefaultDisabled
    );
    assert!(!high_risk_decision.production_apply_enabled);

    let evidence = local_hyprland_version_evidence(
        "0.55.4",
        Some("hyprland 0.55.4-1"),
        Some("Hyprland 0.55.4"),
        true,
        true,
        true,
        true,
        true,
    );
    let activation_gate = hyprland_version_activation_gate(&evidence, false);
    assert_eq!(
        activation_gate.status,
        HyprlandVersionActivationStatus::ReadyButDefaultDisabled
    );
    let migration_request = approval_request(
        ApprovalScope::Hyprland0554Migration,
        None,
        Some("hyprland_0554_migration"),
        false,
        true,
    );
    let migration_decision =
        hyprland_0554_approval_flow(&activation_gate, Some(&migration_request));
    assert_eq!(
        migration_decision.status,
        ApprovalStatus::ReadyButDefaultDisabled
    );
    assert!(!migration_decision.production_apply_enabled);

    let partial = local_hyprland_version_evidence(
        "0.55.4",
        Some("hyprland 0.55.4-1"),
        None,
        false,
        false,
        false,
        false,
        true,
    );
    let partial_gate = hyprland_version_activation_gate(&partial, false);
    assert_ne!(
        partial_gate.status,
        HyprlandVersionActivationStatus::ReadyButDefaultDisabled
    );
}

#[test]
fn high_risk_production_gate_requires_recovery_deadman_restore_backup_snapshot_and_approval() {
    let root = temp_root("high-risk-production-gate");
    let config = root.join("hyprland.conf");
    write_file(&config, "render:direct_scanout = false\n");
    let protocol = high_risk_live_recovery_protocol("render.direct_scanout", config, true, false);
    let missing_recovery = high_risk_production_gate_review(
        Some(&protocol),
        false,
        true,
        true,
        true,
        true,
        true,
        false,
    );
    assert_eq!(
        missing_recovery.status,
        HighRiskProductionGateStatus::RecoveryMissing
    );
    assert!(!missing_recovery.production_write_enabled);

    let missing_deadman = high_risk_production_gate_review(
        Some(&protocol),
        true,
        false,
        true,
        true,
        true,
        true,
        false,
    );
    assert_eq!(
        missing_deadman.status,
        HighRiskProductionGateStatus::DeadManTimeoutMissing
    );

    let ready = high_risk_production_gate_review(
        Some(&protocol),
        true,
        true,
        true,
        true,
        true,
        true,
        false,
    );
    assert_eq!(
        ready.status,
        HighRiskProductionGateStatus::ReadyButDefaultDisabled
    );
    assert!(!ready.production_write_enabled);
}

#[test]
fn hyprland_0554_activation_gate_keeps_0552_default_until_all_trusted_inputs_and_flag_exist() {
    let partial = local_hyprland_version_evidence(
        "0.55.4",
        Some("hyprland 0.55.4-1"),
        Some("Hyprland 0.55.4"),
        false,
        false,
        false,
        false,
        false,
    );
    let partial_gate = hyprland_version_activation_gate(&partial, false);
    assert_eq!(
        partial_gate.status,
        HyprlandVersionActivationStatus::MissingUserApproval
    );
    assert!(!partial_gate.migration_activated);
    assert!(!partial_gate.production_default_changed);
    assert_eq!(partial_gate.current_default_version, "0.55.2");

    let complete = local_hyprland_version_evidence(
        "0.55.4",
        Some("hyprland 0.55.4-1"),
        Some("Hyprland 0.55.4"),
        true,
        true,
        true,
        true,
        true,
    );
    let ready = hyprland_version_activation_gate(&complete, false);
    assert_eq!(
        ready.status,
        HyprlandVersionActivationStatus::ReadyButDefaultDisabled
    );
    assert!(!ready.migration_activated);
    assert_eq!(ready.current_default_version, "0.55.2");

    let current =
        local_hyprland_version_evidence("0.55.2", None, None, false, false, false, false, false);
    let current_gate = hyprland_version_activation_gate(&current, false);
    assert_eq!(
        current_gate.status,
        HyprlandVersionActivationStatus::Enabled
    );
    assert!(current_gate.migration_activated);
    assert_eq!(current_gate.current_default_version, "0.55.2");
}
