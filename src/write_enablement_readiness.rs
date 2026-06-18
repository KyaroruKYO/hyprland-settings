pub const PRODUCTION_WRITE_TARGET_SELECTION_READY: bool = true;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ManualSmokeReviewChecklist {
    pub title: String,
    pub where_to_inspect: Vec<String>,
    pub expected_ui_copy: Vec<String>,
    pub disabled_controls: Vec<String>,
    pub must_not_happen: Vec<String>,
    pub screenshot_automation: String,
}

impl ManualSmokeReviewChecklist {
    pub fn user_facing_lines(&self) -> Vec<String> {
        let mut lines = vec![self.title.clone()];
        lines.extend(self.where_to_inspect.iter().cloned());
        lines.extend(
            self.expected_ui_copy
                .iter()
                .map(|copy| format!("Look for: {copy}")),
        );
        lines.extend(
            self.disabled_controls
                .iter()
                .map(|control| format!("Disabled: {control}")),
        );
        lines.extend(
            self.must_not_happen
                .iter()
                .map(|boundary| format!("Must not happen: {boundary}")),
        );
        lines.push(self.screenshot_automation.clone());
        lines
    }
}

pub fn disabled_walkthrough_manual_smoke_checklist() -> ManualSmokeReviewChecklist {
    ManualSmokeReviewChecklist {
        title: "Disabled write review walkthrough manual smoke checklist".to_string(),
        where_to_inspect: vec![
            "Launch the app.".to_string(),
            "Open a normal settings category.".to_string(),
            "Select a setting controlled in more than one place.".to_string(),
            "Inspect the setting detail pane.".to_string(),
        ],
        expected_ui_copy: vec![
            "Write review walkthrough".to_string(),
            "Shown when a setting is controlled in more than one place.".to_string(),
            "This walkthrough shows what the app would check before writing.".to_string(),
            "Recommended save location".to_string(),
            "Backup planned".to_string(),
            "Verification planned".to_string(),
            "Target decisions are preview-only right now.".to_string(),
            "Real save-location selection is not active yet.".to_string(),
            "Real writing is not active yet.".to_string(),
            "Apply behavior has not changed.".to_string(),
        ],
        disabled_controls: vec![
            "Target decisions are preview-only".to_string(),
            "Review save location".to_string(),
            "Production enablement is disabled".to_string(),
        ],
        must_not_happen: vec![
            "No config file is edited.".to_string(),
            "No Hyprland reload is run.".to_string(),
            "No mutating hyprctl command is run.".to_string(),
            "No real save-location selection becomes active.".to_string(),
            "Apply behavior does not change.".to_string(),
        ],
        screenshot_automation:
            "Automated screenshots were not required for this source-level smoke support sprint."
                .to_string(),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionWriteEnablementReadiness {
    pub status: ProductionWriteEnablementStatus,
    pub gates: Vec<ProductionWriteEnablementGate>,
    pub production_apply_integration_allowed: bool,
    pub real_write_target_selection_active: bool,
    pub real_layered_writes_active: bool,
}

impl ProductionWriteEnablementReadiness {
    pub fn user_facing_lines(&self) -> Vec<String> {
        let mut lines = vec![
            "Production write enablement".to_string(),
            "Status: Not ready".to_string(),
            "Target-selection approval is staged; real selection is still not active yet."
                .to_string(),
            "The app can preview the review flow, but cannot write through it.".to_string(),
            "Before enabling writes, exact backup, reread verification, recovery, and advanced confirmation must be complete.".to_string(),
        ];
        lines.extend(
            self.gates
                .iter()
                .map(|gate| format!("Required before enabling: {}", gate.label)),
        );
        lines.push("Real write-target selection is not active yet.".to_string());
        lines.push("Real writing is not active yet.".to_string());
        lines.push("Apply behavior has not changed.".to_string());
        lines
    }

    pub fn is_ready(&self) -> bool {
        self.status == ProductionWriteEnablementStatus::Ready
            && self.production_apply_integration_allowed
            && self.gates.iter().all(|gate| gate.satisfied)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProductionWriteEnablementStatus {
    NotReady,
    Ready,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProductionWriteEnablementGate {
    pub id: &'static str,
    pub label: &'static str,
    pub satisfied: bool,
    pub production_enabling: bool,
}

pub fn current_production_write_enablement_readiness() -> ProductionWriteEnablementReadiness {
    ProductionWriteEnablementReadiness {
        status: ProductionWriteEnablementStatus::NotReady,
        gates: vec![
            gate(
                "production_write_review_gate",
                "Production write review gate enabled",
            ),
            gate("target_selection_ui", "Target selection UI enabled"),
            gate("exact_backup", "Exact backup implementation"),
            gate("backup_path_policy", "Backup path policy finalized"),
            gate(
                "generated_script_confirmation",
                "Generated/script-managed confirmation",
            ),
            gate("symlink_target_policy", "Symlink target policy"),
            gate("reread_verification", "Reread verification"),
            gate("rollback_recovery", "Rollback/recovery plan"),
            gate("high_risk_policy", "High-risk policy integration"),
            gate("fixture_proof", "Fixture proof acceptance"),
            gate("manual_smoke_review", "Manual smoke review completed"),
            gate(
                "production_apply_integration",
                "Production Apply integration explicitly allowed",
            ),
        ],
        production_apply_integration_allowed: false,
        real_write_target_selection_active: false,
        real_layered_writes_active: false,
    }
}

fn gate(id: &'static str, label: &'static str) -> ProductionWriteEnablementGate {
    ProductionWriteEnablementGate {
        id,
        label,
        satisfied: false,
        production_enabling: true,
    }
}
