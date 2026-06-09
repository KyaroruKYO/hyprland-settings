use std::fs;
use std::io;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::Result;
use hyprland_settings::screen_shader_advisory::{
    run_screen_shader_advisory_check, select_vertex_profile, AdvisoryStatus, AdvisoryVertexProfile,
    ScreenShaderAdvisoryRequest,
};

const TEX300: &str = "/tmp/Hyprland-v0.55.2-full/src/render/shaders/glsl/tex300.vert";
const TEX320: &str = "/tmp/Hyprland-v0.55.2-full/src/render/shaders/glsl/tex320.vert";

fn temp_case(name: &str) -> Result<PathBuf> {
    let nanos = SystemTime::now().duration_since(UNIX_EPOCH)?.as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "hyprland-settings-screen-shader-advisory-{name}-{}-{nanos}",
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

fn base_request(dir: &Path, shader: &Path, tool: &Path) -> ScreenShaderAdvisoryRequest {
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

fn command_exists(command: &str) -> bool {
    std::process::Command::new("bash")
        .args(["-lc", &format!("command -v {command} >/dev/null 2>&1")])
        .status()
        .is_ok_and(|status| status.success())
}

#[test]
fn advisory_helper_refuses_missing_explicit_user_consent() -> Result<()> {
    let dir = temp_case("missing-consent")?;
    let shader = dir.join("selected.frag");
    let tool = dir.join("glslangValidator");
    write_shader(&shader, "#version 300 es\nvoid main() {}\n")?;
    write_fake_tool(&tool, "#!/bin/sh\nexit 0\n")?;

    let mut request = base_request(&dir, &shader, &tool);
    request.explicit_user_consent = false;
    let result = run_screen_shader_advisory_check(&request);

    assert_eq!(result.status, AdvisoryStatus::MissingConsent);
    assert_eq!(result.compiler_args.len(), 0);
    assert_eq!(result.production_write_decision_changed, false);
    assert_eq!(result.runtime_safety_claimed, false);
    assert_eq!(result.write_blocking, false);

    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn advisory_helper_copies_selected_shader_and_passes_only_temp_paths_to_compiler() -> Result<()> {
    let dir = temp_case("temp-copy")?;
    let shader = dir.join("selected.frag");
    let tool = dir.join("glslangValidator");
    let arg_log = dir.join("args.log");
    write_shader(
        &shader,
        "#version 300 es\nprecision mediump float;\nin vec2 v_texcoord;\nlayout(location = 0) out vec4 fragColor;\nuniform sampler2D tex;\nvoid main() { fragColor = texture(tex, v_texcoord); }\n",
    )?;
    write_fake_tool(
        &tool,
        &format!(
            "#!/bin/sh\nprintf '%s\\n' \"$@\" > '{}'\nexit 0\n",
            arg_log.display()
        ),
    )?;

    let request = base_request(&dir, &shader, &tool);
    let result = run_screen_shader_advisory_check(&request);

    assert_eq!(result.status, AdvisoryStatus::Passed);
    assert_eq!(
        result.selected_vertex_profile,
        Some(AdvisoryVertexProfile::Tex300)
    );
    assert_eq!(result.original_user_path_passed_to_compiler, false);
    assert_eq!(result.production_write_decision_changed, false);
    assert_eq!(result.runtime_safety_claimed, false);
    assert_eq!(result.write_blocking, false);
    assert!(result
        .compiler_args
        .iter()
        .all(|arg| !arg.contains(shader.to_str().unwrap())));
    assert!(
        result
            .compiler_args
            .iter()
            .filter(|arg| arg.starts_with("/tmp/"))
            .count()
            >= 2
    );
    let logged_args = fs::read_to_string(arg_log)?;
    assert!(!logged_args.contains(shader.to_str().unwrap()));
    assert!(logged_args.contains("selected-screen-shader.frag"));
    assert!(logged_args.contains("tex300.vert"));

    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn advisory_helper_uses_tex320_only_for_exact_source_backed_prefix() -> Result<()> {
    assert_eq!(
        select_vertex_profile(b"#version 320 es\nvoid main() {}\n"),
        AdvisoryVertexProfile::Tex320
    );
    assert_eq!(
        select_vertex_profile(b"\n#version 320 es\nvoid main() {}\n"),
        AdvisoryVertexProfile::Tex300
    );
    assert_eq!(
        select_vertex_profile(b"#version 300 es\nvoid main() {}\n"),
        AdvisoryVertexProfile::Tex300
    );
    Ok(())
}

#[test]
fn advisory_helper_reports_missing_tool_as_unavailable_without_blocking() -> Result<()> {
    let dir = temp_case("missing-tool")?;
    let shader = dir.join("selected.frag");
    write_shader(&shader, "#version 300 es\nvoid main() {}\n")?;
    let missing_tool = dir.join("missing-glslangValidator");

    let result = run_screen_shader_advisory_check(&base_request(&dir, &shader, &missing_tool));

    assert_eq!(result.status, AdvisoryStatus::Unavailable);
    assert_eq!(result.production_write_decision_changed, false);
    assert_eq!(result.runtime_safety_claimed, false);
    assert_eq!(result.write_blocking, false);

    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn advisory_helper_reports_timeout_as_non_blocking_inconclusive_result() -> Result<()> {
    let dir = temp_case("timeout")?;
    let shader = dir.join("selected.frag");
    let tool = dir.join("glslangValidator");
    write_shader(&shader, "#version 300 es\nvoid main() {}\n")?;
    write_fake_tool(&tool, "#!/bin/sh\nsleep 2\nexit 0\n")?;

    let mut request = base_request(&dir, &shader, &tool);
    request.timeout = Duration::from_millis(25);
    let result = run_screen_shader_advisory_check(&request);

    assert_eq!(result.status, AdvisoryStatus::TimedOut);
    assert_eq!(result.production_write_decision_changed, false);
    assert_eq!(result.runtime_safety_claimed, false);
    assert_eq!(result.write_blocking, false);

    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn advisory_helper_reports_failure_as_warning_only_not_write_blocking() -> Result<()> {
    let dir = temp_case("failure")?;
    let shader = dir.join("selected.frag");
    let tool = dir.join("glslangValidator");
    write_shader(&shader, "#version 300 es\nvoid main() {}\n")?;
    write_fake_tool(&tool, "#!/bin/sh\necho bad >&2\nexit 2\n")?;

    let result = run_screen_shader_advisory_check(&base_request(&dir, &shader, &tool));

    assert_eq!(result.status, AdvisoryStatus::Failed);
    assert_eq!(result.exit_code, Some(2));
    assert!(result.stderr.contains("bad"));
    assert_eq!(result.production_write_decision_changed, false);
    assert_eq!(result.runtime_safety_claimed, false);
    assert_eq!(result.write_blocking, false);

    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn advisory_helper_records_cleanup_warning_without_approving_or_blocking_write() -> Result<()> {
    let dir = temp_case("cleanup-warning")?;
    let shader = dir.join("selected.frag");
    let tool = dir.join("glslangValidator");
    write_shader(&shader, "#version 300 es\nvoid main() {}\n")?;
    write_fake_tool(&tool, "#!/bin/sh\nexit 0\n")?;

    let mut request = base_request(&dir, &shader, &tool);
    request.simulate_cleanup_failure = true;
    let result = run_screen_shader_advisory_check(&request);

    assert_eq!(result.status, AdvisoryStatus::CleanupWarning);
    assert!(result.cleanup_warning.is_some());
    assert_eq!(result.production_write_decision_changed, false);
    assert_eq!(result.runtime_safety_claimed, false);
    assert_eq!(result.write_blocking, false);
    if let Some(temp_dir) = result.temp_dir {
        let _ = fs::remove_dir_all(temp_dir);
    }
    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn advisory_helper_with_real_glslang_uses_temp_fixtures_when_available() -> Result<()> {
    let dir = temp_case("real-glslang")?;
    let shader = dir.join("selected.frag");
    write_shader(
        &shader,
        "#version 300 es\nprecision mediump float;\nin vec2 v_texcoord;\nlayout(location = 0) out vec4 fragColor;\nuniform sampler2D tex;\nvoid main() { fragColor = texture(tex, v_texcoord); }\n",
    )?;

    let request = base_request(&dir, &shader, Path::new("glslangValidator"));
    let result = run_screen_shader_advisory_check(&request);

    if command_exists("glslangValidator") {
        assert_eq!(result.status, AdvisoryStatus::Passed);
        assert_eq!(
            result.selected_vertex_profile,
            Some(AdvisoryVertexProfile::Tex300)
        );
    } else {
        assert_eq!(result.status, AdvisoryStatus::Unavailable);
    }
    assert_eq!(result.original_user_path_passed_to_compiler, false);
    assert_eq!(result.production_write_decision_changed, false);
    assert_eq!(result.runtime_safety_claimed, false);
    assert_eq!(result.write_blocking, false);

    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn advisory_helper_with_real_glslang_rejects_invalid_fixture_when_available() -> Result<()> {
    let dir = temp_case("real-glslang-invalid")?;
    let shader = dir.join("selected.frag");
    write_shader(
        &shader,
        "#version 300 es\nprecision mediump float;\nin vec2 v_texcoord;\nlayout(location = 0) out vec4 fragColor;\nuniform sampler2D tex;\nvoid main() { fragColor = vec4(texture(tex, v_texcoord).rgb, ); }\n",
    )?;

    let request = base_request(&dir, &shader, Path::new("glslangValidator"));
    let result = run_screen_shader_advisory_check(&request);

    if command_exists("glslangValidator") {
        assert_eq!(result.status, AdvisoryStatus::Failed);
        assert_ne!(result.exit_code, Some(0));
    } else {
        assert_eq!(result.status, AdvisoryStatus::Unavailable);
    }
    assert_eq!(result.production_write_decision_changed, false);
    assert_eq!(result.runtime_safety_claimed, false);
    assert_eq!(result.write_blocking, false);

    fs::remove_dir_all(dir)?;
    Ok(())
}

#[test]
fn advisory_helper_temp_copy_failure_is_non_blocking() -> Result<()> {
    let dir = temp_case("copy-failure")?;
    let missing_shader = dir.join("missing.frag");
    let tool = dir.join("glslangValidator");
    write_fake_tool(&tool, "#!/bin/sh\nexit 0\n")?;

    let result = run_screen_shader_advisory_check(&base_request(&dir, &missing_shader, &tool));

    assert_eq!(result.status, AdvisoryStatus::TempCopyFailed);
    assert_eq!(result.production_write_decision_changed, false);
    assert_eq!(result.runtime_safety_claimed, false);
    assert_eq!(result.write_blocking, false);

    fs::remove_dir_all(dir).or_else(|err| {
        if err.kind() == io::ErrorKind::NotFound {
            Ok(())
        } else {
            Err(err)
        }
    })?;
    Ok(())
}
