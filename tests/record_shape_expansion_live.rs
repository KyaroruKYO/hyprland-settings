//! Env-gated live proofs for record-shape expansion: the enabled flag and
//! the bezier reference of explicitly overridden animation records. Runtime
//! mutation only (reversible, readback-verified, zero residue) — no config
//! writes happen in the runtime proofs. The save-flow proof (added only
//! after the runtime proofs passed and the shapes were promoted) writes the
//! REAL active config once with a byte-exact backup and restores it.
//! Normal `cargo test` never runs these bodies.
//!
//! Run (serially — the proofs mutate shared runtime state):
//!   HYPRLAND_SETTINGS_RUN_RECORD_SHAPE_EXPANSION_LIVE=1 \
//!   HYPRLAND_SETTINGS_RECORD_SHAPE_TARGET=all \
//!   cargo test --test record_shape_expansion_live -- --ignored --nocapture --test-threads=1

use hyprland_settings::family_record_picker::render_animation_preview_expression;
use hyprland_settings::runtime_preview_executor::{
    HyprctlRuntimePreviewRunner, RuntimePreviewRunner,
};
use hyprland_settings::structured_family_runtime_preview::{
    parse_animation_records, parse_bezier_records, AnimationRuntimeRecord,
};

fn env_gate(target: &str) -> bool {
    if std::env::var("HYPRLAND_SETTINGS_RUN_RECORD_SHAPE_EXPANSION_LIVE").as_deref() != Ok("1") {
        eprintln!("skipping: HYPRLAND_SETTINGS_RUN_RECORD_SHAPE_EXPANSION_LIVE not set");
        return false;
    }
    if std::env::var("HYPRLAND_INSTANCE_SIGNATURE").is_err() {
        eprintln!("skipping: no live Hyprland session");
        return false;
    }
    let selected = std::env::var("HYPRLAND_SETTINGS_RECORD_SHAPE_TARGET")
        .unwrap_or_else(|_| "all".to_string());
    if selected != "all" && selected != target {
        eprintln!("skipping: target {selected} does not select {target}");
        return false;
    }
    true
}

fn read_record(runner: &mut dyn RuntimePreviewRunner, name: &str) -> AnimationRuntimeRecord {
    let listing = runner
        .run("hyprctl", &["animations".to_string()])
        .expect("animations listing reads");
    parse_animation_records(&listing)
        .into_iter()
        .find(|record| record.name == name)
        .unwrap_or_else(|| panic!("record {name} present in readback"))
}

fn speed_of(record: &AnimationRuntimeRecord) -> f64 {
    record.speed.parse().expect("speed parses")
}

/// Apply desired enabled/speed/bezier through the same fixed-shape
/// expression the picker preview uses (the record clone carries the desired
/// enabled/bezier; other fields come from the readback).
fn apply_fields(
    runner: &mut dyn RuntimePreviewRunner,
    record: &AnimationRuntimeRecord,
    enabled: &str,
    speed: f64,
    bezier: &str,
) {
    let mut desired = record.clone();
    desired.enabled = enabled.to_string();
    desired.bezier = bezier.to_string();
    let expression =
        render_animation_preview_expression(&desired, speed).expect("expression renders");
    runner
        .run("hyprctl", &["eval".to_string(), expression])
        .expect("eval runs");
}

fn assert_full_record(
    observed: &AnimationRuntimeRecord,
    enabled: &str,
    speed: f64,
    bezier: &str,
    style: &str,
    context: &str,
) {
    assert_eq!(observed.enabled, enabled, "{context}: enabled");
    assert!(
        (speed_of(observed) - speed).abs() < 1e-3,
        "{context}: expected speed {speed}, observed {}",
        observed.speed
    );
    assert_eq!(observed.bezier, bezier, "{context}: bezier");
    assert_eq!(observed.style, style, "{context}: style");
    assert!(observed.overridden, "{context}: still overridden");
}

#[test]
#[ignore = "mutates the running compositor (reversibly); run only with HYPRLAND_SETTINGS_RUN_RECORD_SHAPE_EXPANSION_LIVE=1"]
fn record_shape_proof_animation_record_enabled_round_trip() {
    if !env_gate("animation-enabled") {
        return;
    }
    let mut runner = HyprctlRuntimePreviewRunner;
    let listing = runner
        .run("hyprctl", &["animations".to_string()])
        .expect("animations listing reads");

    // Proof record: the FIRST (sorted) explicitly overridden, currently
    // ENABLED, style-free, non-internal record that is NOT the original
    // family-proof record (global) and NOT the speed-shape proof record
    // (fade), proving the generalization on fresh ground. Sorted-first here
    // and sorted-last in the bezier proof keeps the two proofs on distinct
    // records.
    let mut eligible: Vec<String> = parse_animation_records(&listing)
        .into_iter()
        .filter(|record| {
            record.overridden
                && record.enabled == "1"
                && record.style.is_empty()
                && !record.name.starts_with("__")
                && record.name != "global"
                && record.name != "fade"
        })
        .map(|record| record.name)
        .collect();
    eligible.sort();
    let record_name = eligible.first().expect("an eligible record exists").clone();
    let original = read_record(&mut runner, &record_name);
    let original_speed = speed_of(&original);
    println!(
        "ENABLED PROOF record {record_name}: original enabled={} speed={} bezier={}",
        original.enabled, original.speed, original.bezier
    );

    // Apply enabled=0 with every other field passed unchanged. LIVE FINDING:
    // the compositor RESETS the speed/bezier readback of a record while it
    // is disabled (observed: speed -> 1.00, bezier -> default), so only the
    // enabled flag itself is verifiable here; the revert below proves the
    // original fields restore exactly.
    apply_fields(
        &mut runner,
        &original,
        "0",
        original_speed,
        &original.bezier,
    );
    let observed = read_record(&mut runner, &record_name);
    assert_eq!(observed.enabled, "0", "apply enabled=0: enabled");
    assert!(observed.overridden, "apply enabled=0: still overridden");
    assert_eq!(observed.style, original.style, "apply enabled=0: style");
    println!(
        "ENABLED PROOF apply verified: {record_name} enabled 1 -> 0 (disabled readback reports reset speed={} bezier={})",
        observed.speed, observed.bezier
    );

    // Exact revert.
    apply_fields(
        &mut runner,
        &original,
        "1",
        original_speed,
        &original.bezier,
    );
    let restored = read_record(&mut runner, &record_name);
    assert_eq!(restored, original, "zero residue: full record restored");
    println!("ENABLED PROOF PASSED: {record_name} restored byte-identically; zero residue");
}

#[test]
#[ignore = "mutates the running compositor (reversibly); run only with HYPRLAND_SETTINGS_RUN_RECORD_SHAPE_EXPANSION_LIVE=1"]
fn record_shape_proof_animation_record_enabled_from_disabled_round_trip() {
    if !env_gate("animation-enabled") {
        return;
    }
    let mut runner = HyprctlRuntimePreviewRunner;
    let listing = runner
        .run("hyprctl", &["animations".to_string()])
        .expect("animations listing reads");

    // Proof record: an explicitly overridden, currently DISABLED record.
    // The earlier live finding was that the compositor ignores SPEED changes
    // on disabled records; this proof answers whether the enabled flag
    // itself can be toggled on (and exactly back off) with verification.
    let Some(original) = parse_animation_records(&listing)
        .into_iter()
        .find(|record| {
            record.overridden
                && record.enabled == "0"
                && record.style.is_empty()
                && !record.name.starts_with("__")
        })
    else {
        eprintln!("skipping: no explicitly overridden disabled record exists in this session");
        return;
    };
    let record_name = original.name.clone();
    let original_speed = speed_of(&original);
    println!(
        "ENABLED-FROM-DISABLED PROOF record {record_name}: original enabled={} speed={} bezier={}",
        original.enabled, original.speed, original.bezier
    );

    // Apply enabled=1 with every other field unchanged.
    apply_fields(
        &mut runner,
        &original,
        "1",
        original_speed,
        &original.bezier,
    );
    let observed = read_record(&mut runner, &record_name);
    assert_full_record(
        &observed,
        "1",
        original_speed,
        &original.bezier,
        &original.style,
        "apply enabled=1 from disabled",
    );
    println!(
        "ENABLED-FROM-DISABLED PROOF apply verified: {record_name} enabled 0 -> 1, other fields unchanged"
    );

    // Exact revert.
    apply_fields(
        &mut runner,
        &original,
        "0",
        original_speed,
        &original.bezier,
    );
    let restored = read_record(&mut runner, &record_name);
    assert_eq!(restored, original, "zero residue: full record restored");
    println!(
        "ENABLED-FROM-DISABLED PROOF PASSED: {record_name} restored byte-identically; zero residue"
    );
}

#[test]
#[ignore = "mutates the running compositor (reversibly); run only with HYPRLAND_SETTINGS_RUN_RECORD_SHAPE_EXPANSION_LIVE=1"]
fn record_shape_proof_animation_record_bezier_round_trip() {
    if !env_gate("animation-bezier") {
        return;
    }
    let mut runner = HyprctlRuntimePreviewRunner;
    let listing = runner
        .run("hyprctl", &["animations".to_string()])
        .expect("animations listing reads");
    let curves: Vec<String> = parse_bezier_records(&listing)
        .into_iter()
        .map(|curve| curve.name)
        .collect();
    assert!(
        curves.len() >= 2,
        "at least two curves exist to swap between"
    );

    // Proof record: the LAST (sorted) explicitly overridden, enabled,
    // style-free, non-internal eligible record — distinct from the enabled
    // proof's sorted-first pick.
    let mut eligible: Vec<AnimationRuntimeRecord> = parse_animation_records(&listing)
        .into_iter()
        .filter(|record| {
            record.overridden
                && record.enabled == "1"
                && record.style.is_empty()
                && !record.name.starts_with("__")
                && record.name != "global"
                && record.name != "fade"
                && !record.bezier.is_empty()
                && curves.contains(&record.bezier)
        })
        .collect();
    eligible.sort_by(|left, right| left.name.cmp(&right.name));
    let original = eligible
        .pop()
        .expect("an eligible bezier proof record exists");
    let record_name = original.name.clone();
    let original_speed = speed_of(&original);
    // The replacement is a DIFFERENT existing curve from the readback —
    // only existing curve names are ever referenced.
    let replacement = curves
        .iter()
        .find(|curve| **curve != original.bezier)
        .expect("a different existing curve exists")
        .clone();
    println!(
        "BEZIER PROOF record {record_name}: original bezier={} -> replacement {replacement}",
        original.bezier
    );

    // Apply the replacement curve with every other field unchanged.
    apply_fields(&mut runner, &original, "1", original_speed, &replacement);
    let observed = read_record(&mut runner, &record_name);
    assert_full_record(
        &observed,
        "1",
        original_speed,
        &replacement,
        &original.style,
        "apply replacement bezier",
    );
    println!(
        "BEZIER PROOF apply verified: {record_name} bezier {} -> {replacement}, other fields unchanged",
        original.bezier
    );

    // Exact revert.
    apply_fields(
        &mut runner,
        &original,
        "1",
        original_speed,
        &original.bezier,
    );
    let restored = read_record(&mut runner, &record_name);
    assert_eq!(restored, original, "zero residue: full record restored");
    println!("BEZIER PROOF PASSED: {record_name} restored byte-identically; zero residue");
}
