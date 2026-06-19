use crate::safe_batch_write::SafeBatchEligibility;
use crate::write_classification::high_risk_write_policy;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum HighRiskFamily {
    DisplayRenderPipeline,
    ShaderScreenShader,
    MonitorOutput,
    InputDevice,
    AnimationPerformance,
    WindowRuleWorkspaceBehavior,
    ExecScriptPath,
    EnvironmentSession,
    ProfileModeSwitch,
    UnknownHighRisk,
}

impl HighRiskFamily {
    pub fn label(self) -> &'static str {
        match self {
            Self::DisplayRenderPipeline => "display_render_pipeline_risk",
            Self::ShaderScreenShader => "shader_screen_shader_risk",
            Self::MonitorOutput => "monitor_output_risk",
            Self::InputDevice => "input_device_risk",
            Self::AnimationPerformance => "animation_performance_risk",
            Self::WindowRuleWorkspaceBehavior => "window_rule_workspace_behavior_risk",
            Self::ExecScriptPath => "exec_script_path_risk",
            Self::EnvironmentSession => "environment_session_risk",
            Self::ProfileModeSwitch => "profile_mode_switch_risk",
            Self::UnknownHighRisk => "unknown_high_risk_family",
        }
    }

    pub fn user_facing_blocked_copy(self) -> &'static str {
        match self {
            Self::DisplayRenderPipeline | Self::ShaderScreenShader => {
                "Blocked: display/render settings need a recovery path before the app can write them."
            }
            Self::MonitorOutput => {
                "Blocked: monitor/output settings need a display recovery path before the app can write them."
            }
            Self::InputDevice => {
                "Blocked: input settings need an input-safe recovery path before the app can write them."
            }
            Self::AnimationPerformance => {
                "Blocked: animation/performance settings need freeze-safe recovery before the app can write them."
            }
            Self::WindowRuleWorkspaceBehavior => {
                "Blocked: window/workspace behavior settings need extra review before the app can write them."
            }
            Self::ExecScriptPath => {
                "Blocked: script/path settings need extra review because the app must not execute scripts while checking them."
            }
            Self::EnvironmentSession => {
                "Blocked: session/environment settings need restart-boundary review before the app can write them."
            }
            Self::ProfileModeSwitch => {
                "Blocked: profile/mode settings need profile support before the app can write them."
            }
            Self::UnknownHighRisk => {
                "Blocked: this setting needs a specific safety plan before the app can write it."
            }
        }
    }

    pub fn required_proof(self) -> &'static str {
        match self {
            Self::DisplayRenderPipeline => {
                "visual-safe recovery, persisted backup, delayed apply review, rollback timer, and no reload unless separately proven"
            }
            Self::ShaderScreenShader => {
                "shader file validation, compile/readability advisory, display watchdog, persisted rollback, and timeout restore"
            }
            Self::MonitorOutput => {
                "display topology snapshot, visible confirmation, out-of-band rollback path, and timeout restore"
            }
            Self::InputDevice => {
                "input-independent confirmation, keyboard/token fallback, pointer-independent recovery, and timeout restore"
            }
            Self::AnimationPerformance => {
                "performance-safe bounds, fixture proof, freeze-risk warning, and backup rollback"
            }
            Self::WindowRuleWorkspaceBehavior => {
                "workspace/window behavior review, fixture proof, and manual confirmation path"
            }
            Self::ExecScriptPath => {
                "path existence/readability checks, no script execution during validation, and explicit user confirmation"
            }
            Self::EnvironmentSession => {
                "session boundary review, restart-only semantics, no runtime mutation, and clear pending-effect copy"
            }
            Self::ProfileModeSwitch => {
                "profile manager design, no symlink switching during validation, and separate profile recovery proof"
            }
            Self::UnknownHighRisk => {
                "family-specific risk analysis, fixture proof, recovery design, and explicit gate"
            }
        }
    }

    pub fn recovery_behavior(self) -> &'static str {
        match self {
            Self::DisplayRenderPipeline | Self::ShaderScreenShader | Self::MonitorOutput => {
                "dead-man confirm-or-revert with persisted backup and display-independent restore report"
            }
            Self::InputDevice => {
                "dead-man confirm-or-revert independent of pointer visibility, pointer focus, and normal mouse input"
            }
            Self::AnimationPerformance
            | Self::WindowRuleWorkspaceBehavior
            | Self::ExecScriptPath
            | Self::EnvironmentSession => {
                "persisted backup restore with reread verification and no Hyprland reload"
            }
            Self::ProfileModeSwitch => {
                "profile restore plan that does not switch symlinks during validation"
            }
            Self::UnknownHighRisk => "blocked until recovery behavior is specified",
        }
    }

    pub fn recommended_strategy(self) -> &'static str {
        match self {
            Self::DisplayRenderPipeline => "keep blocked; design visual watchdog batch gate",
            Self::ShaderScreenShader => {
                "keep blocked; use shader advisory plus render watchdog proof"
            }
            Self::MonitorOutput => "keep blocked; design display topology and rollback proof",
            Self::InputDevice => "keep blocked; use input-independent watchdog confirmation",
            Self::AnimationPerformance => "keep blocked; prove bounded fixture and freeze recovery",
            Self::WindowRuleWorkspaceBehavior => {
                "keep blocked; split window/workspace behavior review"
            }
            Self::ExecScriptPath => "keep blocked; validate paths without executing anything",
            Self::EnvironmentSession => "keep blocked or write only with restart-pending semantics",
            Self::ProfileModeSwitch => "keep blocked until profile manager support exists",
            Self::UnknownHighRisk => "keep blocked until classified by a narrower family",
        }
    }
}

pub fn high_risk_family_for_row(row_id: &str) -> Option<HighRiskFamily> {
    if row_id == "decoration.screen_shader" {
        return Some(HighRiskFamily::ShaderScreenShader);
    }
    if row_id.contains("profile") || row_id.contains("mode_switch") {
        return Some(HighRiskFamily::ProfileModeSwitch);
    }
    if row_id.starts_with("monitor.")
        || row_id.starts_with("workspace.")
        || row_id == "cursor.default_monitor"
    {
        return Some(HighRiskFamily::MonitorOutput);
    }
    if row_id.starts_with("input.") || row_id.starts_with("cursor.") {
        return Some(HighRiskFamily::InputDevice);
    }
    if row_id.starts_with("animation") || row_id.starts_with("animations.") {
        return Some(HighRiskFamily::AnimationPerformance);
    }
    if row_id.starts_with("windowrule.")
        || row_id.starts_with("window.")
        || row_id.contains("workspace")
    {
        return Some(HighRiskFamily::WindowRuleWorkspaceBehavior);
    }
    if row_id.contains("exec")
        || row_id.contains("script")
        || row_id.contains("path")
        || row_id.contains("shader")
    {
        return Some(HighRiskFamily::ExecScriptPath);
    }
    if row_id.starts_with("env.") || row_id.contains("session") || row_id.contains("reload") {
        return Some(HighRiskFamily::EnvironmentSession);
    }
    let policy = high_risk_write_policy(row_id)?;
    if policy.recovery_bucket.contains("display-render")
        || matches!(
            row_id.split('.').next(),
            Some("render" | "xwayland" | "opengl" | "experimental" | "quirks")
        )
    {
        return Some(HighRiskFamily::DisplayRenderPipeline);
    }
    if policy.recovery_bucket.contains("cursor-input") {
        return Some(HighRiskFamily::InputDevice);
    }
    Some(HighRiskFamily::UnknownHighRisk)
}

pub fn display_render_family_for_row(row_id: &str) -> Option<HighRiskFamily> {
    if row_id == "decoration.screen_shader" {
        return Some(HighRiskFamily::ShaderScreenShader);
    }
    if row_id.contains("profile") || row_id.contains("mode_switch") {
        return Some(HighRiskFamily::ProfileModeSwitch);
    }
    if high_risk_write_policy(row_id).is_some() {
        return high_risk_family_for_row(row_id);
    }
    matches!(
        row_id.split('.').next(),
        Some("render" | "xwayland" | "opengl" | "experimental" | "quirks")
    )
    .then_some(HighRiskFamily::DisplayRenderPipeline)
}

pub fn family_blocked_reason(row_id: &str) -> Option<SafeBatchEligibility> {
    let family = display_render_family_for_row(row_id)?;
    Some(match family {
        HighRiskFamily::DisplayRenderPipeline | HighRiskFamily::ShaderScreenShader => {
            SafeBatchEligibility::BlockedDisplayRenderRisk
        }
        HighRiskFamily::ProfileModeSwitch => SafeBatchEligibility::BlockedProfileModeSwitch,
        _ => SafeBatchEligibility::BlockedHighRisk,
    })
}
