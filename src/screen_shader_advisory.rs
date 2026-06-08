use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvisoryVertexProfile {
    Tex300,
    Tex320,
}

impl AdvisoryVertexProfile {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Tex300 => "tex300",
            Self::Tex320 => "tex320",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AdvisoryStatus {
    Passed,
    Failed,
    Unavailable,
    TimedOut,
    TempCopyFailed,
    MissingConsent,
    CleanupWarning,
}

impl AdvisoryStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Unavailable => "unavailable",
            Self::TimedOut => "timed_out",
            Self::TempCopyFailed => "temp_copy_failed",
            Self::MissingConsent => "missing_consent",
            Self::CleanupWarning => "cleanup_warning",
        }
    }
}

#[derive(Debug, Clone)]
pub struct ScreenShaderAdvisoryRequest {
    pub selected_shader_path: PathBuf,
    pub temp_root: PathBuf,
    pub tex300_vertex_path: PathBuf,
    pub tex320_vertex_path: PathBuf,
    pub glslang_validator_path: PathBuf,
    pub timeout: Duration,
    pub explicit_user_consent: bool,
    pub simulate_cleanup_failure: bool,
}

#[derive(Debug, Clone)]
pub struct ScreenShaderAdvisoryResult {
    pub status: AdvisoryStatus,
    pub stdout: String,
    pub stderr: String,
    pub exit_code: Option<i32>,
    pub selected_vertex_profile: Option<AdvisoryVertexProfile>,
    pub temp_dir: Option<PathBuf>,
    pub temp_fragment_path: Option<PathBuf>,
    pub temp_vertex_path: Option<PathBuf>,
    pub compiler_program: PathBuf,
    pub compiler_args: Vec<String>,
    pub original_user_path_passed_to_compiler: bool,
    pub production_write_decision_changed: bool,
    pub runtime_safety_claimed: bool,
    pub write_blocking: bool,
    pub cleanup_warning: Option<String>,
    pub diagnostic: Option<String>,
}

impl ScreenShaderAdvisoryResult {
    fn early(
        status: AdvisoryStatus,
        compiler_program: PathBuf,
        diagnostic: impl Into<String>,
    ) -> Self {
        Self {
            status,
            stdout: String::new(),
            stderr: String::new(),
            exit_code: None,
            selected_vertex_profile: None,
            temp_dir: None,
            temp_fragment_path: None,
            temp_vertex_path: None,
            compiler_program,
            compiler_args: Vec::new(),
            original_user_path_passed_to_compiler: false,
            production_write_decision_changed: false,
            runtime_safety_claimed: false,
            write_blocking: false,
            cleanup_warning: None,
            diagnostic: Some(diagnostic.into()),
        }
    }
}

pub fn run_screen_shader_advisory_check(
    request: &ScreenShaderAdvisoryRequest,
) -> ScreenShaderAdvisoryResult {
    if !request.explicit_user_consent {
        return ScreenShaderAdvisoryResult::early(
            AdvisoryStatus::MissingConsent,
            request.glslang_validator_path.clone(),
            "explicit user consent is required before reading the selected shader",
        );
    }

    let temp_dir = match create_private_temp_dir(&request.temp_root) {
        Ok(path) => path,
        Err(err) => {
            return ScreenShaderAdvisoryResult::early(
                AdvisoryStatus::TempCopyFailed,
                request.glslang_validator_path.clone(),
                format!("failed to create advisory temp directory: {err}"),
            );
        }
    };

    let temp_fragment_path = temp_dir.join("selected-screen-shader.frag");
    let fragment_source = match fs::read(&request.selected_shader_path) {
        Ok(bytes) => {
            if let Err(err) = fs::write(&temp_fragment_path, &bytes) {
                return finish_with_cleanup(
                    request,
                    temp_dir,
                    ScreenShaderAdvisoryResult::early(
                        AdvisoryStatus::TempCopyFailed,
                        request.glslang_validator_path.clone(),
                        format!("failed to copy selected shader into temp fixture: {err}"),
                    ),
                );
            }
            bytes
        }
        Err(err) => {
            return finish_with_cleanup(
                request,
                temp_dir,
                ScreenShaderAdvisoryResult::early(
                    AdvisoryStatus::TempCopyFailed,
                    request.glslang_validator_path.clone(),
                    format!("failed to read explicit selected shader: {err}"),
                ),
            );
        }
    };

    let profile = select_vertex_profile(&fragment_source);
    let source_vertex = match profile {
        AdvisoryVertexProfile::Tex300 => &request.tex300_vertex_path,
        AdvisoryVertexProfile::Tex320 => &request.tex320_vertex_path,
    };
    let temp_vertex_path = temp_dir.join(match profile {
        AdvisoryVertexProfile::Tex300 => "tex300.vert",
        AdvisoryVertexProfile::Tex320 => "tex320.vert",
    });

    if let Err(err) = fs::copy(source_vertex, &temp_vertex_path) {
        return finish_with_cleanup(
            request,
            temp_dir.clone(),
            ScreenShaderAdvisoryResult {
                status: AdvisoryStatus::TempCopyFailed,
                stdout: String::new(),
                stderr: String::new(),
                exit_code: None,
                selected_vertex_profile: Some(profile),
                temp_dir: Some(temp_dir),
                temp_fragment_path: Some(temp_fragment_path),
                temp_vertex_path: Some(temp_vertex_path),
                compiler_program: request.glslang_validator_path.clone(),
                compiler_args: Vec::new(),
                original_user_path_passed_to_compiler: false,
                production_write_decision_changed: false,
                runtime_safety_claimed: false,
                write_blocking: false,
                cleanup_warning: None,
                diagnostic: Some(format!(
                    "failed to copy source-backed Hyprland vertex shader into temp fixture: {err}"
                )),
            },
        );
    }

    let compiler_args = vec![
        "-l".to_string(),
        temp_vertex_path.to_string_lossy().to_string(),
        temp_fragment_path.to_string_lossy().to_string(),
    ];
    let original_user_path_passed_to_compiler = compiler_args
        .iter()
        .any(|arg| Path::new(arg) == request.selected_shader_path);

    let mut child = match Command::new(&request.glslang_validator_path)
        .args(&compiler_args)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
    {
        Ok(child) => child,
        Err(err) if err.kind() == io::ErrorKind::NotFound => {
            return finish_with_cleanup(
                request,
                temp_dir.clone(),
                ScreenShaderAdvisoryResult {
                    status: AdvisoryStatus::Unavailable,
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: None,
                    selected_vertex_profile: Some(profile),
                    temp_dir: Some(temp_dir),
                    temp_fragment_path: Some(temp_fragment_path),
                    temp_vertex_path: Some(temp_vertex_path),
                    compiler_program: request.glslang_validator_path.clone(),
                    compiler_args,
                    original_user_path_passed_to_compiler,
                    production_write_decision_changed: false,
                    runtime_safety_claimed: false,
                    write_blocking: false,
                    cleanup_warning: None,
                    diagnostic: Some("glslangValidator is unavailable".to_string()),
                },
            );
        }
        Err(err) => {
            return finish_with_cleanup(
                request,
                temp_dir.clone(),
                ScreenShaderAdvisoryResult {
                    status: AdvisoryStatus::Unavailable,
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: None,
                    selected_vertex_profile: Some(profile),
                    temp_dir: Some(temp_dir),
                    temp_fragment_path: Some(temp_fragment_path),
                    temp_vertex_path: Some(temp_vertex_path),
                    compiler_program: request.glslang_validator_path.clone(),
                    compiler_args,
                    original_user_path_passed_to_compiler,
                    production_write_decision_changed: false,
                    runtime_safety_claimed: false,
                    write_blocking: false,
                    cleanup_warning: None,
                    diagnostic: Some(format!("failed to start glslangValidator: {err}")),
                },
            );
        }
    };

    let started = Instant::now();
    loop {
        match child.try_wait() {
            Ok(Some(_)) => break,
            Ok(None) if started.elapsed() >= request.timeout => {
                let _ = child.kill();
                let output = child.wait_with_output().ok();
                return finish_with_cleanup(
                    request,
                    temp_dir.clone(),
                    ScreenShaderAdvisoryResult {
                        status: AdvisoryStatus::TimedOut,
                        stdout: output
                            .as_ref()
                            .map(|out| String::from_utf8_lossy(&out.stdout).to_string())
                            .unwrap_or_default(),
                        stderr: output
                            .as_ref()
                            .map(|out| String::from_utf8_lossy(&out.stderr).to_string())
                            .unwrap_or_default(),
                        exit_code: output.and_then(|out| out.status.code()),
                        selected_vertex_profile: Some(profile),
                        temp_dir: Some(temp_dir),
                        temp_fragment_path: Some(temp_fragment_path),
                        temp_vertex_path: Some(temp_vertex_path),
                        compiler_program: request.glslang_validator_path.clone(),
                        compiler_args,
                        original_user_path_passed_to_compiler,
                        production_write_decision_changed: false,
                        runtime_safety_claimed: false,
                        write_blocking: false,
                        cleanup_warning: None,
                        diagnostic: Some("glslangValidator advisory check timed out".to_string()),
                    },
                );
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(err) => {
                let _ = child.kill();
                return finish_with_cleanup(
                    request,
                    temp_dir.clone(),
                    ScreenShaderAdvisoryResult {
                        status: AdvisoryStatus::Unavailable,
                        stdout: String::new(),
                        stderr: String::new(),
                        exit_code: None,
                        selected_vertex_profile: Some(profile),
                        temp_dir: Some(temp_dir),
                        temp_fragment_path: Some(temp_fragment_path),
                        temp_vertex_path: Some(temp_vertex_path),
                        compiler_program: request.glslang_validator_path.clone(),
                        compiler_args,
                        original_user_path_passed_to_compiler,
                        production_write_decision_changed: false,
                        runtime_safety_claimed: false,
                        write_blocking: false,
                        cleanup_warning: None,
                        diagnostic: Some(format!("failed to poll glslangValidator: {err}")),
                    },
                );
            }
        }
    }

    let output = match child.wait_with_output() {
        Ok(output) => output,
        Err(err) => {
            return finish_with_cleanup(
                request,
                temp_dir.clone(),
                ScreenShaderAdvisoryResult {
                    status: AdvisoryStatus::Unavailable,
                    stdout: String::new(),
                    stderr: String::new(),
                    exit_code: None,
                    selected_vertex_profile: Some(profile),
                    temp_dir: Some(temp_dir),
                    temp_fragment_path: Some(temp_fragment_path),
                    temp_vertex_path: Some(temp_vertex_path),
                    compiler_program: request.glslang_validator_path.clone(),
                    compiler_args,
                    original_user_path_passed_to_compiler,
                    production_write_decision_changed: false,
                    runtime_safety_claimed: false,
                    write_blocking: false,
                    cleanup_warning: None,
                    diagnostic: Some(format!("failed to collect glslangValidator output: {err}")),
                },
            );
        }
    };

    let status = if output.status.success() {
        AdvisoryStatus::Passed
    } else {
        AdvisoryStatus::Failed
    };

    finish_with_cleanup(
        request,
        temp_dir.clone(),
        ScreenShaderAdvisoryResult {
            status,
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            exit_code: output.status.code(),
            selected_vertex_profile: Some(profile),
            temp_dir: Some(temp_dir),
            temp_fragment_path: Some(temp_fragment_path),
            temp_vertex_path: Some(temp_vertex_path),
            compiler_program: request.glslang_validator_path.clone(),
            compiler_args,
            original_user_path_passed_to_compiler,
            production_write_decision_changed: false,
            runtime_safety_claimed: false,
            write_blocking: false,
            cleanup_warning: None,
            diagnostic: None,
        },
    )
}

pub fn select_vertex_profile(fragment_source: &[u8]) -> AdvisoryVertexProfile {
    if fragment_source.starts_with(b"#version 320 es") {
        AdvisoryVertexProfile::Tex320
    } else {
        AdvisoryVertexProfile::Tex300
    }
}

fn create_private_temp_dir(temp_root: &Path) -> io::Result<PathBuf> {
    fs::create_dir_all(temp_root)?;
    let nanos = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let path = temp_root.join(format!(
        "screen-shader-advisory-{}-{nanos}",
        std::process::id()
    ));
    fs::create_dir(&path)?;
    Ok(path)
}

fn finish_with_cleanup(
    request: &ScreenShaderAdvisoryRequest,
    temp_dir: PathBuf,
    mut result: ScreenShaderAdvisoryResult,
) -> ScreenShaderAdvisoryResult {
    result.temp_dir.get_or_insert_with(|| temp_dir.clone());

    let cleanup_result = if request.simulate_cleanup_failure {
        Err(io::Error::other(
            "simulated cleanup failure for advisory proof",
        ))
    } else {
        fs::remove_dir_all(&temp_dir)
    };

    if let Err(err) = cleanup_result {
        result.cleanup_warning = Some(format!("failed to clean advisory temp directory: {err}"));
        if matches!(
            result.status,
            AdvisoryStatus::Passed | AdvisoryStatus::Failed
        ) {
            result.status = AdvisoryStatus::CleanupWarning;
        }
    }

    result.production_write_decision_changed = false;
    result.runtime_safety_claimed = false;
    result.write_blocking = false;
    result
}
