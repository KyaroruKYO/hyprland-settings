pub const PRODUCTION_RECOVERY_CONTRACT_ENABLED: bool = false;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionRecoveryContract {
    pub backup_exists_required: bool,
    pub backup_verified_before_write_required: bool,
    pub restore_exact_backup_bytes_required: bool,
    pub reread_restored_file_required: bool,
    pub report_rollback_success_or_failure_required: bool,
    pub hyprland_reload_allowed: bool,
    pub production_enabled: bool,
}

impl ProductionRecoveryContract {
    pub fn user_facing_lines(&self) -> Vec<String> {
        vec![
            "Rollback/recovery must be implemented before real writes.".to_string(),
            "If verification fails, the app must restore the exact backup bytes.".to_string(),
            "This pilot must never reload Hyprland automatically.".to_string(),
            "Production recovery is not active yet.".to_string(),
        ]
    }
}

pub fn production_recovery_prerequisite_contract() -> ProductionRecoveryContract {
    ProductionRecoveryContract {
        backup_exists_required: true,
        backup_verified_before_write_required: true,
        restore_exact_backup_bytes_required: true,
        reread_restored_file_required: true,
        report_rollback_success_or_failure_required: true,
        hyprland_reload_allowed: false,
        production_enabled: PRODUCTION_RECOVERY_CONTRACT_ENABLED,
    }
}
