use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use hyprland_settings::ui::model::{
    run_screen_shader_advisory_selected_file_ui_action, run_screen_shader_advisory_ui_action,
    screen_shader_advisory_gtk_widget_projection, ScreenShaderAdvisorySelectedFileUiActionRequest,
    ScreenShaderAdvisoryUiActionRequest, ScreenShaderAdvisoryUiResultState,
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
        "hyprland-settings-screen-shader-file-chooser-{name}-{}-{nanos}",
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

fn selected_file_request(
    dir: &Path,
    selected_shader: Option<PathBuf>,
    tool: &Path,
    explicit_user_trigger: bool,
) -> ScreenShaderAdvisorySelectedFileUiActionRequest {
    ScreenShaderAdvisorySelectedFileUiActionRequest {
        row_id: "decoration.screen_shader".to_string(),
        explicit_user_trigger,
        selected_shader_path: selected_shader,
        temp_root: dir.join("advisory-temp-root"),
        tex300_vertex_path: PathBuf::from(TEX300),
        tex320_vertex_path: PathBuf::from(TEX320),
        glslang_validator_path: tool.to_path_buf(),
        timeout: Duration::from_secs(2),
        simulate_cleanup_failure: false,
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
fn file_chooser_execution_report_records_option_b_and_counts() -> Result<()> {
    let report =
        read_json("data/reports/screen-shader-advisory-file-chooser-execution-proof.v0.55.2.json")?;
    let coverage = read_json("data/reports/scalar-read-write-coverage.v0.55.2.json")?;
    let widget =
        read_json("data/reports/screen-shader-advisory-gtk-widget-wiring-proof.v0.55.2.json")?;
    let approval = read_json("data/reports/screen-shader-production-gate-approval.v0.55.2.json")?;

    assert!(is_safe_writable_setting("decoration.screen_shader"));
    assert_eq!(SAFE_WRITABLE_ROWS.len(), 278);
    assert_eq!(coverage["counts"]["readableRows"], 341);
    assert_eq!(coverage["counts"]["writableRows"], 278);
    assert_eq!(coverage["counts"]["blockedWriteRows"], 63);

    assert_eq!(report["rowId"], "decoration.screen_shader");
    assert_eq!(report["officialSetting"], "decoration.screen_shader");
    assert_eq!(report["startingCommit"], "e21ee10");
    assert_eq!(
        report["selectedFileChooserExecutionProofOption"],
        "Option B"
    );
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
    assert_eq!(report["advisoryUiImplementationStatus"], "complete");
    assert_eq!(report["gtkWidgetWiringStatus"], "complete");
    assert_eq!(report["visibleGtkWidgetImplemented"], true);
    assert_eq!(report["fileChooserExecutionImplemented"], false);
    assert_eq!(report["selectedFileActionModelImplemented"], true);
    assert_eq!(report["compileAwareValidationCurrentStatus"], "deferred");
    assert_eq!(report["safeWritableRowsChanged"], false);
    assert_eq!(report["writeAllowlistChanged"], false);
    assert_eq!(report["rowsEnabledThisSprint"], 0);
    assert_eq!(report["readableRows"], 341);
    assert_eq!(report["writableRows"], 278);
    assert_eq!(report["blockedRows"], 63);
    assert_eq!(widget["visibleGtkWidgetImplemented"], true);
    assert_eq!(approval["productionGateEnforcedThisSprint"], true);

    Ok(())
}

#[test]
fn selected_file_action_model_uses_generated_fixture_and_temp_compiler_paths() -> Result<()> {
    let dir = temp_case("selected-pass")?;
    let selected_shader = dir.join("selected-fixture.frag");
    let tool = dir.join("glslangValidator");
    let arg_log = dir.join("args.log");
    write_shader(
        &selected_shader,
        "#version 300 es\nprecision mediump float;\nin vec2 v_texcoord;\nlayout(location = 0) out vec4 fragColor;\nuniform sampler2D tex;\nvoid main() { fragColor = texture(tex, v_texcoord); }\n",
    )?;
    write_fake_tool(
        &tool,
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\nexit 0\n",
            arg_log.display()
        ),
    )?;

    let request = selected_file_request(&dir, Some(selected_shader.clone()), &tool, true);
    let temp_root = request.temp_root.clone();
    let render = run_screen_shader_advisory_selected_file_ui_action(request);

    assert_eq!(render.state, ScreenShaderAdvisoryUiResultState::Passed);
    assert!(render.helper_invoked);
    assert!(render.selected_shader_read);
    assert!(render.compiler_invoked);
    assert!(!render.compiler_args.is_empty());
    assert!(!render.original_user_path_passed_to_compiler);
    assert!(render
        .compiler_args
        .iter()
        .all(|arg| !arg.contains(selected_shader.to_str().unwrap())));
    for arg in render
        .compiler_args
        .iter()
        .filter(|arg| arg.as_str() != "-l")
    {
        assert!(Path::new(arg).starts_with(&temp_root));
    }
    let logged_args = fs::read_to_string(&arg_log)?;
    assert!(!logged_args.contains(selected_shader.to_str().unwrap()));
    assert!(logged_args.contains("selected-screen-shader.frag"));
    assert!(logged_args.contains("tex300.vert"));
    assert_non_authoritative(&render);

    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn selected_file_action_model_requires_selection_and_explicit_trigger() -> Result<()> {
    let dir = temp_case("selection-required")?;
    let tool = dir.join("glslangValidator");
    write_fake_tool(&tool, "#!/bin/sh\nexit 0\n")?;

    let missing_selection = run_screen_shader_advisory_selected_file_ui_action(
        selected_file_request(&dir, None, &tool, true),
    );
    assert_eq!(
        missing_selection.state,
        ScreenShaderAdvisoryUiResultState::NotRun
    );
    assert!(missing_selection.consent_required);
    assert!(!missing_selection.helper_invoked);
    assert!(!missing_selection.selected_shader_read);
    assert!(!missing_selection.compiler_invoked);
    assert!(missing_selection.compiler_args.is_empty());
    assert_non_authoritative(&missing_selection);

    let missing_consent_shader = dir.join("would-not-be-read.frag");
    let missing_consent = run_screen_shader_advisory_selected_file_ui_action(
        selected_file_request(&dir, Some(missing_consent_shader), &tool, false),
    );
    assert_eq!(
        missing_consent.state,
        ScreenShaderAdvisoryUiResultState::NotRun
    );
    assert!(missing_consent.consent_required);
    assert!(!missing_consent.helper_invoked);
    assert!(!missing_consent.selected_shader_read);
    assert!(!missing_consent.compiler_invoked);
    assert!(missing_consent.compiler_args.is_empty());
    assert_non_authoritative(&missing_consent);

    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn file_chooser_execution_report_keeps_action_out_of_write_safety() -> Result<()> {
    let report =
        read_json("data/reports/screen-shader-advisory-file-chooser-execution-proof.v0.55.2.json")?;

    assert_eq!(report["runsOnRowLoad"], false);
    assert_eq!(report["runsOnValueChange"], false);
    assert_eq!(report["runsDuringValidation"], false);
    assert_eq!(report["runsDuringPendingChange"], false);
    assert_eq!(report["runsDuringWritePlanning"], false);
    assert_eq!(report["runsDuringApplyFlow"], false);
    assert_eq!(report["advisoryResultCanApproveWrite"], false);
    assert_eq!(report["advisoryResultCanBlockWrite"], false);
    assert_eq!(report["advisoryResultCanBypassProductionGate"], false);
    assert_eq!(report["compilerChecksWiredIntoValidators"], false);
    assert_eq!(report["compilerChecksWiredIntoPendingChanges"], false);
    assert_eq!(report["compilerChecksWiredIntoWritePlanning"], false);
    assert_eq!(report["compilerChecksWiredIntoApplyFlow"], false);
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
    assert_eq!(report["generatedFixtureSelectedFilesUsedInTests"], true);
    assert_eq!(report["backgroundShaderScanningAllowed"], false);
    assert_eq!(report["arbitraryConfigPathReadsAllowed"], false);
    assert_eq!(report["originalSelectedPathPassedToCompiler"], false);
    assert_eq!(report["compilerReceivesOnlyTempPaths"], true);
    assert_eq!(report["tempCopyRequired"], true);
    assert_eq!(report["writesOutsideTempDirAllowed"], false);

    Ok(())
}

#[test]
fn selected_file_action_model_links_unified_pipeline() -> Result<()> {
    let report =
        read_json("data/reports/screen-shader-advisory-file-chooser-execution-proof.v0.55.2.json")?;
    let pipeline = read_json("data/reports/all-341-unified-pipeline.v0.55.2.json")?;

    let projection = screen_shader_advisory_gtk_widget_projection("decoration.screen_shader")
        .expect("screen shader widget projection expected");
    assert!(!projection.file_chooser_execution_implemented);
    assert!(projection.selected_file_action_model_implemented);
    assert_eq!(
        projection.selected_file_action_module,
        "src/ui/model.rs::run_screen_shader_advisory_selected_file_ui_action"
    );
    assert!(screen_shader_advisory_gtk_widget_projection("appearance.blur.size").is_none());

    assert_eq!(
        report["unifiedPipelineRepresentation"]["rowLinkField"],
        "advisoryFileChooserExecutionProofSource"
    );
    assert_eq!(
        report["recommendedValidationPolicy"]["requiredPreflight"],
        false
    );
    assert_eq!(
        report["recommendedValidationPolicy"]["applyFlowIntegration"],
        false
    );
    assert!(report["compatibilityGaps"]
        .as_array()
        .expect("compatibility gaps should be explicit")
        .iter()
        .any(|gap| gap.as_str().unwrap().contains("Direct GTK file chooser")));
    assert!(report["proofStillMissing"]
        .as_array()
        .expect("missing proof should be explicit")
        .iter()
        .any(|gap| gap.as_str().unwrap().contains("Direct GTK file chooser")));

    let screen_shader_row = pipeline["rows"]
        .as_array()
        .unwrap()
        .iter()
        .find(|row| row["rowId"] == "decoration.screen_shader")
        .expect("screen shader row should exist in all-341 pipeline");
    assert_eq!(
        screen_shader_row["advisoryFileChooserExecutionProofSource"],
        "screen-shader-advisory-file-chooser-execution-proof.v0.55.2.json"
    );
    assert_eq!(screen_shader_row["productionGateEnforcedThisSprint"], true);
    assert_eq!(screen_shader_row["countedAsEnabledHighRiskRow"], false);

    let unrelated = run_screen_shader_advisory_ui_action(ScreenShaderAdvisoryUiActionRequest {
        row_id: "appearance.blur.size".to_string(),
        explicit_user_trigger: true,
        helper_request: None,
    });
    assert_eq!(unrelated.state, ScreenShaderAdvisoryUiResultState::NotRun);
    assert!(!unrelated.helper_invoked);

    Ok(())
}
