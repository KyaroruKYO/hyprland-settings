use std::fs;
use std::path::{Path, PathBuf};

use hyprland_settings::production_verification_contract::{
    fixture_reread_verify_expected_value, production_verification_contract_for_candidate,
    FixtureRereadVerificationError, ProductionVerificationStatus,
    PRODUCTION_VERIFICATION_CONTRACT_ENABLED,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;
use hyprland_settings::write_target_candidate::WriteTargetCandidate;
use hyprland_settings::write_target_fixture_proof::{
    prove_fixture_target_write, FixtureTargetWriteProofRequest,
};

fn temp_fixture(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "hyprland-settings-production-verification-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("fixture root should be created");
    root
}

fn write_file(path: &Path, content: &str) {
    fs::write(path, content).expect("fixture file should be written");
}

fn candidate(path: PathBuf) -> WriteTargetCandidate {
    WriteTargetCandidate {
        label: "Main config".to_string(),
        file_path: path,
        resolved_path: None,
        line_number: Some(2),
        safe: true,
        generated_or_script_managed: false,
        symlink_managed: false,
        requires_advanced_confirmation: false,
        backup_required: true,
        fixture_only: true,
    }
}

#[test]
fn fixture_reread_verification_passes_and_fails_without_real_files() {
    let root = temp_fixture("pass-fail");
    let config = root.join("hyprland.conf");
    write_file(&config, "# header\ngeneral:layout = dwindle\n");
    let candidate = candidate(config.clone());

    prove_fixture_target_write(&FixtureTargetWriteProofRequest {
        target: candidate.clone(),
        setting_id: "general.layout".to_string(),
        new_value: "master".to_string(),
        advanced_fixture_approval: false,
    })
    .expect("fixture write proof should write expected value");

    let pass_contract =
        production_verification_contract_for_candidate(&candidate, "general.layout", "master");
    let pass = fixture_reread_verify_expected_value(&pass_contract)
        .expect("fixture reread should parse temp file");
    assert_eq!(pass.status, ProductionVerificationStatus::PassedInFixture);
    assert_eq!(pass.observed_value.as_deref(), Some("master"));

    let fail_contract =
        production_verification_contract_for_candidate(&candidate, "general.layout", "dwindle");
    let fail = fixture_reread_verify_expected_value(&fail_contract)
        .expect("fixture reread should report mismatch as failed proof");
    assert_eq!(fail.status, ProductionVerificationStatus::FailedInFixture);
    assert_eq!(fail.observed_value.as_deref(), Some("master"));

    let rejected =
        fixture_reread_verify_expected_value(&production_verification_contract_for_candidate(
            &candidate.with_file_path(PathBuf::from("/home/kyo/.config/hypr/hyprland.conf")),
            "general.layout",
            "master",
        ))
        .expect_err("user config path must be rejected by fixture verification");
    assert_eq!(rejected, FixtureRereadVerificationError::NonFixturePath);

    assert!(!PRODUCTION_VERIFICATION_CONTRACT_ENABLED);
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 341);
}

trait CandidatePathExt {
    fn with_file_path(&self, file_path: PathBuf) -> WriteTargetCandidate;
}

impl CandidatePathExt for WriteTargetCandidate {
    fn with_file_path(&self, file_path: PathBuf) -> WriteTargetCandidate {
        WriteTargetCandidate {
            file_path,
            ..self.clone()
        }
    }
}
