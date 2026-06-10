use crate::write_target_candidate::WriteTargetCandidate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteAdvancedConfirmation {
    pub requires_confirmation: bool,
    pub reasons: Vec<String>,
    pub generated_file_warning: bool,
    pub script_managed_warning: bool,
    pub symlink_managed_warning: bool,
    pub advanced_mode_required: bool,
    pub confirmed: bool,
    pub production_disabled: bool,
}

impl WriteAdvancedConfirmation {
    pub fn user_facing_lines(&self) -> Vec<String> {
        let mut lines = Vec::new();
        if self.script_managed_warning {
            lines.push("This file may be changed by scripts.".to_string());
        }
        if self.generated_file_warning {
            lines.push("This file appears to be generated.".to_string());
        }
        if self.symlink_managed_warning {
            lines.push("This file is symlinked.".to_string());
        }
        if self.requires_confirmation {
            lines.push("Advanced confirmation would be required before writing here.".to_string());
        }
        lines.push("Advanced confirmation is not active yet.".to_string());
        lines
    }
}

pub fn advanced_confirmation_for_candidate(
    candidate: &WriteTargetCandidate,
) -> WriteAdvancedConfirmation {
    let generated_or_script = candidate.generated_or_script_managed;
    let symlink = candidate.symlink_managed;
    let mut reasons = Vec::new();
    if generated_or_script {
        reasons.push("generated or script-managed file".to_string());
    }
    if symlink {
        reasons.push("symlink-managed file".to_string());
    }

    WriteAdvancedConfirmation {
        requires_confirmation: candidate.requires_advanced_confirmation,
        reasons,
        generated_file_warning: generated_or_script,
        script_managed_warning: generated_or_script,
        symlink_managed_warning: symlink,
        advanced_mode_required: candidate.requires_advanced_confirmation,
        confirmed: false,
        production_disabled: true,
    }
}
