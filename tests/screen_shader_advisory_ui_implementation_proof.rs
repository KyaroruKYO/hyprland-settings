use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use hyprland_settings::screen_shader_advisory::ScreenShaderAdvisoryRequest;
use hyprland_settings::ui::model::{
    initial_screen_shader_advisory_ui_action, run_screen_shader_advisory_ui_action,
    running_screen_shader_advisory_ui_action, ScreenShaderAdvisoryUiActionRequest,
    ScreenShaderAdvisoryUiResultState,
};
use hyprland_settings::write_classification::{is_safe_writable_setting, SAFE_WRITABLE_ROWS};
use serde_json::Value;

const TEX300: &str = "/tmp/Hyprland-v0.55.2-full/src/render/shaders/glsl/tex300.vert";
const TEX320: &str = "/tmp/Hyprland-v0.55.2-full/src/render/shaders/glsl/tex320.vert";

fn read_json(path: &str) -> Result<Value> {
    Ok(serde_json::from_str(&std::fs::read_to_string(path)?)?)
}

fn temp_case(name: &str) -> Result<PathBuf> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "hyprland-settings-screen-shader-ui-action-{name}-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir_all(&dir)?;
    Ok(dir)
}

fn write_shader(path: &Path, source: &str) -> Result<()> {
    fs::write(path, source)?;
    Ok(())
}

fn write_fake_tool(path: &Path, body: &str) -> Result<()> {
    fs::write(path, body)?;
    let mut permissions = fs::metadata(path)?.permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(path, permissions)?;
    Ok(())
}

fn base_helper_request(dir: &Path, shader: &Path, tool: &Path) -> ScreenShaderAdvisoryRequest {
    ScreenShaderAdvisoryRequest {
        selected_shader_path: shader.to_path_buf(),
        temp_root: dir.join("advisory-temp-root"),
        tex300_vertex_path: PathBuf::from(TEX300),
        tex320_vertex_path: PathBuf::from(TEX320),
        glslang_validator_path: tool.to_path_buf(),
        timeout: Duration::from_secs(2),
        explicit_user_consent: true,
        simulate_cleanup_failure: false,
    }
}

fn ui_request(helper_request: ScreenShaderAdvisoryRequest) -> ScreenShaderAdvisoryUiActionRequest {
    ScreenShaderAdvisoryUiActionRequest {
        row_id: "decoration.screen_shader".to_string(),
        explicit_user_trigger: true,
        helper_request: Some(helper_request),
    }
}

fn assert_non_authoritative(
    render: &hyprland_settings::ui::model::ScreenShaderAdvisoryUiActionRender,
) {
    assert!(!render.can_approve_write);
    assert!(!render.can_block_write);
    assert!(!render.can_bypass_production_gate);
    assert!(!render.production_write_decision_changed);
    assert!(!render.runtime_safety_claimed);
    assert!(!render.write_blocking);
}

#[test]
fn screen_shader_ui_implementation_report_records_option_a() -> Result<()> {
    let report =
        read_json("data/reports/screen-shader-advisory-ui-implementation-proof.v0.55.2.json")?;
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let approval = read_json("data/reports/screen-shader-production-gate-approval.v0.55.2.json")?;
    let exposure =
        read_json("data/reports/screen-shader-advisory-ui-exposure-design.v0.55.2.json")?;

    assert!(is_safe_writable_setting("decoration.screen_shader"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 278);
    assert_eq!(coverage["counts"]["readableRows"], 341);
    assert_eq!(coverage["counts"]["writableRows"], 278);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 63);

    assert_eq!(report["rowId"], "decoration.screen_shader");
    assert_eq!(report["officialSetting"], "decoration.screen_shader");
    assert_eq!(report["startingCommit"], "4a32d6b");
    assert_eq!(report["selectedUiImplementationProofOption"], "Option A");
    assert_eq!(report["currentWritableStatus"], "writable");
    assert_eq!(report["productionGateEnforced"], true);
    assert_eq!(report["productionGateChanged"], false);
    assert_eq!(report["watchdogMigrationProofStatus"], "complete");
    assert_eq!(report["advisoryCompilerFeasibilityStatus"], "complete");
    assert_eq!(
        report["advisoryCompilerIntegrationDesignStatus"],
        "complete"
    );
    assert_eq!(report["advisoryHelperImplementationStatus"], "complete");
    assert_eq!(report["advisoryUiExposureDesignStatus"], "complete");
    assert_eq!(report["chosenAdvisoryTool"], "glslangValidator");
    assert_eq!(report["advisoryUiActionImplemented"], true);
    assert_eq!(
        report["advisoryUiActionModule"],
        "src/ui/model.rs::run_screen_shader_advisory_ui_action"
    );
    assert_eq!(report["compileAwareValidationCurrentStatus"], "deferred");
    assert_eq!(report["compileAwareValidationChanged"], false);
    assert_eq!(report["compileAwareValidationImplemented"], false);
    assert_eq!(report["productionCompileAwareValidationImplemented"], false);
    assert_eq!(report["safeWritableRowsChanged"], false);
    assert_eq!(report["writeAllowlistChanged"], false);
    assert_eq!(report["rowsEnabledThisSprint"], 0);
    assert_eq!(report["readableRows"], 341);
    assert_eq!(report["writableRows"], 278);
    assert_eq!(report["blockedRows"], 63);
    assert_eq!(approval["productionGateEnforcedThisSprint"], true);
    assert_eq!(exposure["advisoryUiExposureImplemented"], false);

    Ok(())
}

#[test]
fn screen_shader_ui_action_models_all_required_states_without_auto_run() -> Result<()> {
    let report =
        read_json("data/reports/screen-shader-advisory-ui-implementation-proof.v0.55.2.json")?;
    let states = report["resultStatesModeled"]
        .as_array()
        .expect("result states should be explicit");
    for state in [
        "not_run",
        "running",
        "passed",
        "failed",
        "unavailable",
        "timed_out",
        "temp_copy_failed",
        "cleanup_warning",
    ] {
        assert!(states.iter().any(|value| value == state));
    }

    let initial = initial_screen_shader_advisory_ui_action("decoration.screen_shader")
        .expect("initial screen shader action state should exist");
    assert_eq!(initial.state, ScreenShaderAdvisoryUiResultState::NotRun);
    assert!(!initial.helper_invoked);
    assert!(!initial.compiler_invoked);
    assert!(!initial.selected_shader_read);
    assert_non_authoritative(&initial);

    let running = running_screen_shader_advisory_ui_action("decoration.screen_shader")
        .expect("running screen shader action state should exist");
    assert_eq!(running.state, ScreenShaderAdvisoryUiResultState::Running);
    assert!(!running.helper_invoked);
    assert!(!running.compiler_invoked);
    assert!(!running.selected_shader_read);
    assert_non_authoritative(&running);

    assert!(initial_screen_shader_advisory_ui_action("appearance.blur.size").is_none());

    assert_eq!(report["runsOnRowLoad"], false);
    assert_eq!(report["runsOnValueChange"], false);
    assert_eq!(report["runsDuringValidation"], false);
    assert_eq!(report["runsDuringPendingChange"], false);
    assert_eq!(report["runsDuringWritePlanning"], false);
    assert_eq!(report["runsDuringApplyFlow"], false);

    Ok(())
}

#[test]
fn screen_shader_ui_action_refuses_missing_consent_without_reading_shader() -> Result<()> {
    let request = ScreenShaderAdvisoryUiActionRequest {
        row_id: "decoration.screen_shader".to_string(),
        explicit_user_trigger: false,
        helper_request: Some(ScreenShaderAdvisoryRequest {
            selected_shader_path: PathBuf::from("/tmp/nonexistent-screen-shader-user-file.frag"),
            temp_root: std::env::temp_dir().join("unused-screen-shader-ui-consent-proof"),
            tex300_vertex_path: PathBuf::from(TEX300),
            tex320_vertex_path: PathBuf::from(TEX320),
            glslang_validator_path: PathBuf::from("glslangValidator"),
            timeout: Duration::from_millis(25),
            explicit_user_consent: false,
            simulate_cleanup_failure: false,
        }),
    };

    let render = run_screen_shader_advisory_ui_action(request);

    assert_eq!(render.state, ScreenShaderAdvisoryUiResultState::NotRun);
    assert!(render.consent_required);
    assert!(!render.helper_invoked);
    assert!(!render.selected_shader_read);
    assert!(!render.compiler_invoked);
    assert!(render.compiler_args.is_empty());
    assert_non_authoritative(&render);

    Ok(())
}

#[test]
fn screen_shader_ui_action_renders_pass_and_uses_only_temp_compiler_paths() -> Result<()> {
    let dir = temp_case("pass")?;
    let shader = dir.join("selected.frag");
    let tool = dir.join("glslangValidator");
    write_shader(
        &shader,
        "#version 300 es\nprecision mediump float;\nin vec2 v_texcoord;\nlayout(location = 0) out vec4 fragColor;\nuniform sampler2D tex;\nvoid main() { fragColor = texture(tex, v_texcoord); }\n",
    )?;
    write_fake_tool(&tool, "#!/bin/sh\nexit 0\n")?;

    let render =
        run_screen_shader_advisory_ui_action(ui_request(base_helper_request(&dir, &shader, &tool)));

    assert_eq!(render.state, ScreenShaderAdvisoryUiResultState::Passed);
    assert!(render.helper_invoked);
    assert!(render.compiler_invoked);
    assert!(!render.compiler_args.is_empty());
    assert!(!render.original_user_path_passed_to_compiler);
    assert!(render
        .compiler_args
        .iter()
        .all(|arg| !arg.contains(shader.to_str().unwrap())));
    assert!(render.message.contains("does not approve"));
    assert_non_authoritative(&render);

    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn screen_shader_ui_action_renders_failure_as_warning_only() -> Result<()> {
    let dir = temp_case("fail")?;
    let shader = dir.join("selected.frag");
    let tool = dir.join("glslangValidator");
    write_shader(&shader, "#version 300 es\nvoid main() {}\n")?;
    write_fake_tool(&tool, "#!/bin/sh\necho bad >&2\nexit 2\n")?;

    let render =
        run_screen_shader_advisory_ui_action(ui_request(base_helper_request(&dir, &shader, &tool)));

    assert_eq!(render.state, ScreenShaderAdvisoryUiResultState::Failed);
    assert!(render.compiler_invoked);
    assert!(render.message.contains("does not automatically block"));
    assert_non_authoritative(&render);

    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn screen_shader_ui_action_renders_missing_tool_timeout_temp_copy_and_cleanup_states() -> Result<()>
{
    let missing_tool_dir = temp_case("missing-tool")?;
    let missing_shader = missing_tool_dir.join("selected.frag");
    write_shader(&missing_shader, "#version 300 es\nvoid main() {}\n")?;
    let unavailable = run_screen_shader_advisory_ui_action(ui_request(base_helper_request(
        &missing_tool_dir,
        &missing_shader,
        &missing_tool_dir.join("missing-glslangValidator"),
    )));
    assert_eq!(
        unavailable.state,
        ScreenShaderAdvisoryUiResultState::Unavailable
    );
    assert!(unavailable.message.contains("does not approve or block"));
    assert_non_authoritative(&unavailable);
    fs::remove_dir_all(missing_tool_dir)?;

    let timeout_dir = temp_case("timeout")?;
    let timeout_shader = timeout_dir.join("selected.frag");
    let timeout_tool = timeout_dir.join("glslangValidator");
    write_shader(&timeout_shader, "#version 300 es\nvoid main() {}\n")?;
    write_fake_tool(&timeout_tool, "#!/bin/sh\nsleep 2\nexit 0\n")?;
    let mut timeout_request = base_helper_request(&timeout_dir, &timeout_shader, &timeout_tool);
    timeout_request.timeout = Duration::from_millis(25);
    let timed_out = run_screen_shader_advisory_ui_action(ui_request(timeout_request));
    assert_eq!(timed_out.state, ScreenShaderAdvisoryUiResultState::TimedOut);
    assert!(timed_out.message.contains("does not approve or block"));
    assert_non_authoritative(&timed_out);
    fs::remove_dir_all(timeout_dir)?;

    let temp_copy_dir = temp_case("temp-copy-failed")?;
    let temp_copy_tool = temp_copy_dir.join("glslangValidator");
    write_fake_tool(&temp_copy_tool, "#!/bin/sh\nexit 0\n")?;
    let temp_copy_failed = run_screen_shader_advisory_ui_action(ui_request(base_helper_request(
        &temp_copy_dir,
        &temp_copy_dir.join("missing-selected.frag"),
        &temp_copy_tool,
    )));
    assert_eq!(
        temp_copy_failed.state,
        ScreenShaderAdvisoryUiResultState::TempCopyFailed
    );
    assert!(!temp_copy_failed.compiler_invoked);
    assert!(temp_copy_failed
        .message
        .contains("does not approve or block"));
    assert_non_authoritative(&temp_copy_failed);
    fs::remove_dir_all(temp_copy_dir)?;

    let cleanup_dir = temp_case("cleanup-warning")?;
    let cleanup_shader = cleanup_dir.join("selected.frag");
    let cleanup_tool = cleanup_dir.join("glslangValidator");
    write_shader(&cleanup_shader, "#version 300 es\nvoid main() {}\n")?;
    write_fake_tool(&cleanup_tool, "#!/bin/sh\nexit 0\n")?;
    let mut cleanup_request = base_helper_request(&cleanup_dir, &cleanup_shader, &cleanup_tool);
    cleanup_request.simulate_cleanup_failure = true;
    let cleanup_warning = run_screen_shader_advisory_ui_action(ui_request(cleanup_request));
    assert_eq!(
        cleanup_warning.state,
        ScreenShaderAdvisoryUiResultState::CleanupWarning
    );
    assert!(cleanup_warning.cleanup_warning.is_some());
    assert!(cleanup_warning.message.contains("does not approve"));
    assert_non_authoritative(&cleanup_warning);
    if let Some(path) = cleanup_warning
        .temp_fragment_path
        .as_ref()
        .and_then(|path| path.parent())
    {
        let _ = fs::remove_dir_all(path);
    }
    fs::remove_dir_all(cleanup_dir)?;

    Ok(())
}

#[test]
fn screen_shader_ui_action_report_keeps_write_safety_disconnected() -> Result<()> {
    let report =
        read_json("data/reports/screen-shader-advisory-ui-implementation-proof.v0.55.2.json")?;
    let pipeline = read_json("data/reports/all-341-unified-pipeline.v0.55.2.json")?;

    assert_eq!(report["compilerChecksWiredIntoValidators"], false);
    assert_eq!(report["compilerChecksWiredIntoPendingChanges"], false);
    assert_eq!(report["compilerChecksWiredIntoWritePlanning"], false);
    assert_eq!(report["compilerChecksWiredIntoApplyFlow"], false);
    assert_eq!(report["advisoryResultCanApproveWrite"], false);
    assert_eq!(report["advisoryResultCanBlockWrite"], false);
    assert_eq!(report["advisoryResultCanBypassProductionGate"], false);
    assert_eq!(report["shaderCompilationThroughHyprlandRun"], false);
    assert_eq!(
        report["standaloneCompilerCommandsRunOnlyOnTempFixtures"],
        true
    );
    assert_eq!(report["liveShaderCompileUsed"], false);
    assert_eq!(report["liveDisplayRuntimeProofUsed"], false);
    assert_eq!(report["realConfigTouched"], false);
    assert_eq!(report["runtimeTouched"], false);
    assert_eq!(report["reloadEvalLuaUsed"], false);
    assert_eq!(report["realUserShaderFilesReadInTests"], false);
    assert_eq!(report["backgroundShaderScanningAllowed"], false);
    assert_eq!(report["originalUserPathPassedToCompiler"], false);
    assert_eq!(report["tempCopyRequired"], true);
    assert_eq!(report["writesOutsideTempDirAllowed"], false);

    let screen_shader_row = pipeline["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["rowId"] == "decoration.screen_shader")
        .expect("screen shader row should exist in all-341 pipeline");
    assert_eq!(
        screen_shader_row["advisoryUiImplementationProofSource"],
        "screen-shader-advisory-ui-implementation-proof.v0.55.2.json"
    );
    assert_eq!(screen_shader_row["productionGateEnforcedThisSprint"], true);
    assert_eq!(screen_shader_row["countedAsEnabledHighRiskRow"], false);
    assert!(screen_shader_row["nextRequiredWork"]
        .as_str()
        .unwrap()
        .contains("file chooser execution proof"));

    Ok(())
}
