use crate::write_target_candidate::WriteTargetCandidate;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WriteTargetRecommendation {
    pub recommended_target: Option<WriteTargetCandidate>,
    pub other_possible_targets: Vec<WriteTargetCandidate>,
    pub blocked_targets: Vec<BlockedWriteTarget>,
    pub reason: String,
    pub advanced_confirmation_required: bool,
    pub backup_required: bool,
    pub fixture_only: bool,
    pub production_disabled: bool,
}

impl WriteTargetRecommendation {
    pub fn user_facing_lines(&self) -> Vec<String> {
        let mut lines = vec![
            "Save location".to_string(),
            "Real write-target selection is not active yet.".to_string(),
            "The app will back up the exact file before saving changes in a future version."
                .to_string(),
        ];

        if let Some(target) = &self.recommended_target {
            lines.push(format!("Recommended save location: {}", target.label));
            lines.push(format!("Reason: {}", self.reason));
        } else {
            lines.push("Recommended save location: not available yet".to_string());
        }

        if !self.other_possible_targets.is_empty() {
            lines.push(format!(
                "Other possible locations: {}",
                self.other_possible_targets
                    .iter()
                    .map(|target| target.label.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
        }

        if !self.blocked_targets.is_empty() {
            lines.push(format!(
                "Blocked locations: {}",
                self.blocked_targets
                    .iter()
                    .map(|target| target.label.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ));
            lines.push(
                "Generated or script-managed files may require advanced confirmation.".to_string(),
            );
        }

        lines.push("Apply behavior has not changed.".to_string());
        lines
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BlockedWriteTarget {
    pub label: String,
    pub reason: String,
    pub requires_advanced_confirmation: bool,
}

pub fn recommend_write_targets(candidates: &[WriteTargetCandidate]) -> WriteTargetRecommendation {
    let mut safe = Vec::new();
    let mut blocked = Vec::new();

    for candidate in candidates {
        if candidate.safe {
            safe.push(candidate.clone());
        } else {
            blocked.push(BlockedWriteTarget {
                label: candidate.label.clone(),
                reason: if candidate.generated_or_script_managed {
                    "This file may be changed by scripts or generated tooling.".to_string()
                } else {
                    "This location is not safe for automatic writes yet.".to_string()
                },
                requires_advanced_confirmation: candidate.requires_advanced_confirmation,
            });
        }
    }

    let recommended_target = safe.first().cloned();
    let other_possible_targets = safe.into_iter().skip(1).collect::<Vec<_>>();
    let advanced_confirmation_required = blocked
        .iter()
        .any(|target| target.requires_advanced_confirmation);

    WriteTargetRecommendation {
        recommended_target,
        other_possible_targets,
        blocked_targets: blocked,
        reason: "This is the first safe read-only candidate from the detected layered setting."
            .to_string(),
        advanced_confirmation_required,
        backup_required: true,
        fixture_only: true,
        production_disabled: true,
    }
}
