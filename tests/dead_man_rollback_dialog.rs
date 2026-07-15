//! Backend guards for the in-window dead-man rollback dialog (Scope D).
//!
//! The GTK modal itself is exercised by the live GUI harness; these prove
//! the pure text/countdown contract it renders and the controller behavior
//! it drives — timeout/Escape/close revert, Keep prevents the timeout
//! revert, and the countdown starts at the reference's 15 seconds.

use hyprland_settings::runtime_preview_dead_man::{
    dead_man_dialog_message, dead_man_dialog_title, RuntimePreviewDeadManController,
    RuntimePreviewDeadManUiPhase, DEAD_MAN_COUNTDOWN_SECONDS,
};
use hyprland_settings::runtime_preview_executor::{
    RuntimePreviewRunner, RUNTIME_PREVIEW_DEAD_MAN_TIMEOUT_MS,
};
use hyprland_settings::write_classification::SAFE_WRITABLE_ROWS;

struct RecordingRunner {
    log: std::rc::Rc<std::cell::RefCell<Vec<Vec<String>>>>,
}

impl RuntimePreviewRunner for RecordingRunner {
    fn run(&mut self, _program: &str, args: &[String]) -> Result<String, String> {
        self.log.borrow_mut().push(args.to_vec());
        if args.first().map(String::as_str) == Some("getoption") {
            Ok("bool: true\nset: true".to_string())
        } else {
            Ok("ok".to_string())
        }
    }
}

fn armed_applied_controller() -> (
    RuntimePreviewDeadManController,
    std::rc::Rc<std::cell::RefCell<Vec<Vec<String>>>>,
) {
    let row_id = SAFE_WRITABLE_ROWS
        .iter()
        .find(|row| row.official_setting == "animations.enabled")
        .expect("animations.enabled exists")
        .row_id;
    let log = std::rc::Rc::new(std::cell::RefCell::new(Vec::new()));
    let mut controller = RuntimePreviewDeadManController::new(
        row_id,
        Box::new(RecordingRunner { log: log.clone() }),
    )
    .expect("controller builds");
    controller.arm().expect("arm");
    controller.apply("false").expect("apply");
    (controller, log)
}

#[test]
fn countdown_is_fifteen_seconds_and_bound_to_the_executor_timeout() {
    assert_eq!(DEAD_MAN_COUNTDOWN_SECONDS, 15);
    assert_eq!(
        DEAD_MAN_COUNTDOWN_SECONDS * 1000,
        RUNTIME_PREVIEW_DEAD_MAN_TIMEOUT_MS,
        "the displayed countdown must equal the real auto-revert timeout"
    );
}

#[test]
fn dialog_title_matches_the_reference() {
    assert_eq!(dead_man_dialog_title(true), "Keep these display settings?");
    assert_eq!(dead_man_dialog_title(false), "Keep this setting?");
}

#[test]
fn dialog_message_counts_down_with_correct_wording() {
    assert_eq!(
        dead_man_dialog_message(true, 15),
        "Reverting to previous display settings in 15 seconds."
    );
    assert_eq!(
        dead_man_dialog_message(true, 14),
        "Reverting to previous display settings in 14 seconds."
    );
    assert_eq!(
        dead_man_dialog_message(false, 15),
        "Reverting to the previous setting in 15 seconds."
    );
    // Singular at one second.
    assert_eq!(
        dead_man_dialog_message(false, 1),
        "Reverting to the previous setting in 1 second."
    );
}

#[test]
fn countdown_starts_at_fifteen_after_apply() {
    let (controller, _log) = armed_applied_controller();
    assert_eq!(controller.remaining_seconds(), 15);
    assert_eq!(
        controller.phase(),
        RuntimePreviewDeadManUiPhase::CountingDown
    );
}

#[test]
fn timeout_reverts_the_change() {
    let (mut controller, log) = armed_applied_controller();
    // One tick short of timeout: still counting.
    assert!(controller
        .tick(RUNTIME_PREVIEW_DEAD_MAN_TIMEOUT_MS - 1)
        .expect("tick")
        .is_none());
    // Crossing the timeout auto-reverts to the captured original.
    let receipt = controller
        .tick(1)
        .expect("tick")
        .expect("timeout fires a receipt");
    assert_eq!(
        receipt.phase,
        RuntimePreviewDeadManUiPhase::TimedOutReverted
    );
    // The last runtime call restored the original value.
    assert!(log
        .borrow()
        .last()
        .expect("a revert call was recorded")
        .join(" ")
        .contains("enabled = true"));
}

#[test]
fn revert_now_reverts_immediately() {
    let (mut controller, _log) = armed_applied_controller();
    let receipt = controller.revert_now().expect("revert now");
    assert_eq!(receipt.phase, RuntimePreviewDeadManUiPhase::Reverted);
}

#[test]
fn keep_prevents_the_timeout_revert() {
    let (mut controller, _log) = armed_applied_controller();
    controller.confirm_keep().expect("keep");
    // After Keep the countdown is over; ticking past the timeout does
    // nothing (no auto-revert of a confirmed change).
    let outcome = controller
        .tick(RUNTIME_PREVIEW_DEAD_MAN_TIMEOUT_MS * 2)
        .expect("tick after keep");
    assert!(
        outcome.is_none(),
        "a kept change must not be auto-reverted by the countdown"
    );
    assert_ne!(
        controller.phase(),
        RuntimePreviewDeadManUiPhase::TimedOutReverted
    );
}

#[test]
fn closing_unconfirmed_reverts() {
    // Escape / dialog close route through revert_if_unconfirmed.
    let (mut controller, _log) = armed_applied_controller();
    let receipt = controller.revert_if_unconfirmed();
    assert!(receipt.is_some(), "an unconfirmed session reverts on close");
}

#[test]
fn closing_after_keep_does_not_revert() {
    let (mut controller, _log) = armed_applied_controller();
    controller.confirm_keep().expect("keep");
    assert!(
        controller.revert_if_unconfirmed().is_none(),
        "a kept change is not reverted on close"
    );
}
