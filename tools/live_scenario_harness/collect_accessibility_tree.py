#!/usr/bin/env python3
import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path


EXPECTED_TERMS = [
    "hyprland",
    "settings",
    "dashboard",
    "config",
    "appearance",
    "search",
    "apply",
    "safe batch",
    "blocked",
    "duplicate",
    "default",
    "generated",
    "script",
    "symlink",
    "high-risk",
    "display",
    "render",
    "profile",
    "mode",
]

APPROVAL_CARD_ASSERTIONS = {
    "sourceIncludeInsertion": {
        "heading": "Source/include approval review",
        "production": "Production source/include insertion",
        "disabled": "Disabled",
        "action": "Enable source/include insertion (planned)",
        "widget": "hyprland-settings-source-include-approval-review-disabled",
    },
    "duplicateReplacement": {
        "heading": "Duplicate approval review",
        "production": "Production duplicate writes",
        "disabled": "Disabled",
        "action": "Enable duplicate replacement (planned)",
        "widget": "hyprland-settings-duplicate-approval-review-disabled",
    },
    "structuredHlBindWrite": {
        "heading": "Structured hl.bind approval review",
        "production": "Production structured writes",
        "disabled": "Disabled",
        "action": "Enable structured write (planned)",
        "widget": "hyprland-settings-structured-approval-review-disabled",
    },
    "profileModeSwitch": {
        "heading": "Profile/mode approval review",
        "production": "Production profile switching",
        "disabled": "Disabled",
        "action": "Enable profile switching (planned)",
        "widget": "hyprland-settings-profile-approval-review-disabled",
    },
    "highRiskDisplayWrite": {
        "heading": "High-risk/display approval review",
        "production": "Production high-risk/display writes",
        "disabled": "Disabled",
        "action": "Enable high-risk/display writes (planned)",
        "widget": "hyprland-settings-high-risk-approval-review-disabled",
    },
    "hyprland0554Migration": {
        "heading": "Hyprland 0.55.4 migration review",
        "production": "Production migration activation",
        "disabled": "Disabled",
        "action": "Enable 0.55.4 migration (planned)",
        "widget": "hyprland-settings-0554-approval-review-disabled",
    },
}

ACTIVATION_DECISION_ASSERTIONS = {
    "sourceIncludeInsertion": {
        "heading": "Source/include production activation decision",
        "production": "Production source/include insertion",
        "disabled": "Disabled",
        "action": "Enable source/include production activation (planned)",
        "widget": "hyprland-settings-source-include-activation-decision-disabled",
    },
    "duplicateReplacement": {
        "heading": "Duplicate production activation decision",
        "production": "Production duplicate writes",
        "disabled": "Disabled",
        "action": "Enable duplicate production activation (planned)",
        "widget": "hyprland-settings-duplicate-activation-decision-disabled",
    },
}

ACTIVATION_PATH_ASSERTIONS = {
    "sourceIncludeInsertion": {
        "heading": "Source/include production activation path",
        "production": "Production source/include insertion",
        "disabled": "Disabled",
        "action": "Start source/include production activation (planned)",
        "widget": "hyprland-settings-source-include-activation-path-disabled",
    },
    "duplicateReplacement": {
        "heading": "Duplicate production activation path",
        "production": "Production duplicate writes",
        "disabled": "Disabled",
        "action": "Start duplicate production activation (planned)",
        "widget": "hyprland-settings-duplicate-activation-path-disabled",
    },
}

ACTIVATION_CONTROL_ASSERTIONS = {
    "sourceIncludeInsertion": {
        "heading": "Source/include production activation control",
        "production": "Production source/include insertion",
        "disabled": "Disabled",
        "executor": "Executor wiring: Unwired",
        "action": "Validate source/include activation request (planned)",
        "widget": "hyprland-settings-source-include-activation-control-disabled",
    },
    "duplicateReplacement": {
        "heading": "Duplicate production activation control",
        "production": "Production duplicate writes",
        "disabled": "Disabled",
        "executor": "Executor wiring: Unwired",
        "action": "Validate duplicate activation request (planned)",
        "widget": "hyprland-settings-duplicate-activation-control-disabled",
    },
}

ACTIVATION_FORM_ASSERTIONS = {
    "sourceIncludeInsertion": {
        "heading": "Source/include activation request form",
        "production": "Production source/include insertion",
        "disabled": "Disabled",
        "executor": "Executor wiring: Unwired",
        "action": "Validate source/include activation form (planned)",
        "widget": "hyprland-settings-source-include-activation-form-disabled",
        "fields": [
            "User-facing reason",
            "Explicit activation phrase/token",
            "Backup-before-write acknowledgement",
            "Restore-plan acknowledgement",
            "Post-write reread acknowledgement",
            "Final confirmation acknowledgement",
            "Backup-before-write plan",
            "Restore plan",
            "Post-write reread plan",
            "Post-restore verification plan",
            "Dry-run summary",
            "Files that would be touched",
        ],
    },
    "duplicateReplacement": {
        "heading": "Duplicate activation request form",
        "production": "Production duplicate writes",
        "disabled": "Disabled",
        "executor": "Executor wiring: Unwired",
        "action": "Validate duplicate activation form (planned)",
        "widget": "hyprland-settings-duplicate-activation-form-disabled",
        "fields": [
            "User-facing reason",
            "Explicit activation phrase/token",
            "Backup-before-write acknowledgement",
            "Restore-plan acknowledgement",
            "Post-write reread acknowledgement",
            "Final confirmation acknowledgement",
            "Backup-before-write plan",
            "Restore plan",
            "Post-write reread plan",
            "Post-restore verification plan",
            "Dry-run summary",
            "Files that would be touched",
        ],
    },
}

ACTIVATION_DRAFT_ASSERTIONS = {
    "sourceIncludeInsertion": {
        "heading": "Source/include activation draft",
        "production": "Production source/include insertion",
        "disabled": "Disabled",
        "executor": "Executor wiring: Unwired",
        "memory": "In-memory only",
        "update": "Update source/include activation draft (planned)",
        "reset": "Reset source/include activation draft (planned)",
        "widget": "hyprland-settings-source-include-activation-draft-disabled",
    },
    "duplicateReplacement": {
        "heading": "Duplicate activation draft",
        "production": "Production duplicate writes",
        "disabled": "Disabled",
        "executor": "Executor wiring: Unwired",
        "memory": "In-memory only",
        "update": "Update duplicate activation draft (planned)",
        "reset": "Reset duplicate activation draft (planned)",
        "widget": "hyprland-settings-duplicate-activation-draft-disabled",
    },
}

ACTIVATION_DRAFT_EDIT_ASSERTIONS = {
    "sourceIncludeInsertion": {
        "heading": "Source/include live activation draft editing",
        "production": "Production source/include insertion",
        "disabled": "Disabled",
        "executor": "Executor wiring: Unwired",
        "memory": "In-memory only",
        "mode": "Draft editing mode: memory-only",
        "validation": "Draft validation",
        "not_saved": "Not saved to disk",
        "update": "Update source/include activation draft (memory only)",
        "reset": "Reset source/include activation draft (memory only)",
        "widget": "hyprland-settings-source-include-activation-live-draft-edit-disabled",
    },
    "duplicateReplacement": {
        "heading": "Duplicate live activation draft editing",
        "production": "Production duplicate writes",
        "disabled": "Disabled",
        "executor": "Executor wiring: Unwired",
        "memory": "In-memory only",
        "mode": "Draft editing mode: memory-only",
        "validation": "Draft validation",
        "not_saved": "Not saved to disk",
        "update": "Update duplicate activation draft (memory only)",
        "reset": "Reset duplicate activation draft (memory only)",
        "widget": "hyprland-settings-duplicate-activation-live-draft-edit-disabled",
    },
}

ACTIVATION_DRAFT_PERSISTENCE_ASSERTIONS = {
    "sourceIncludeInsertion": {
        "heading": "Source/include activation draft persistence boundary",
        "production": "Production source/include insertion",
        "disabled": "Disabled",
        "executor": "Executor wiring: Unwired",
        "status": "Persistence forbidden by default",
        "enabled": "Persistence enabled: false",
        "written": "Draft written to disk: false",
        "storage": "Storage path: none",
        "enable": "Enable source/include draft persistence (not available)",
        "clear": "Clear source/include persisted draft (not available)",
        "widget": "hyprland-settings-source-include-activation-draft-persistence-boundary-disabled",
    },
    "duplicateReplacement": {
        "heading": "Duplicate activation draft persistence boundary",
        "production": "Production duplicate writes",
        "disabled": "Disabled",
        "executor": "Executor wiring: Unwired",
        "status": "Persistence forbidden by default",
        "enabled": "Persistence enabled: false",
        "written": "Draft written to disk: false",
        "storage": "Storage path: none",
        "enable": "Enable duplicate draft persistence (not available)",
        "clear": "Clear duplicate persisted draft (not available)",
        "widget": "hyprland-settings-duplicate-activation-draft-persistence-boundary-disabled",
    },
}

PRODUCTION_ACTIVATION_SAFETY_GATE_ASSERTIONS = {
    "sourceIncludeInsertion": {
        "heading": "Source/include production activation safety gate",
        "production": "Production source/include insertion",
        "disabled": "Disabled",
        "executor": "Executor wiring: Unwired",
        "status": "Production activation proof partially satisfied but default-disabled",
        "backup": "Byte-exact backup",
        "write": "Write plan",
        "reread": "Reread plan",
        "restore": "Restore plan",
        "no_auto_apply": "No auto-apply proof",
        "persistence_auto_apply": "Persistence auto-apply proof",
        "final_approval": "Explicit final approval",
        "review": "Review source/include production activation gate (not available)",
        "enable": "Enable source/include production activation (not available)",
        "widget": "hyprland-settings-source-include-production-activation-safety-gate-disabled",
    },
    "duplicateReplacement": {
        "heading": "Duplicate production activation safety gate",
        "production": "Production duplicate writes",
        "disabled": "Disabled",
        "executor": "Executor wiring: Unwired",
        "status": "Production activation proof partially satisfied but default-disabled",
        "backup": "Byte-exact backup",
        "write": "Write plan",
        "reread": "Reread plan",
        "restore": "Restore plan",
        "no_auto_apply": "No auto-apply proof",
        "persistence_auto_apply": "Persistence auto-apply proof",
        "final_approval": "Explicit final approval",
        "review": "Review duplicate production activation gate (not available)",
        "enable": "Enable duplicate production activation (not available)",
        "widget": "hyprland-settings-duplicate-production-activation-safety-gate-disabled",
    },
}

PRODUCTION_ACTIVATION_SAFETY_PROOF_ASSERTIONS = {
    "sourceIncludeInsertion": {
        "heading": "Source/include production activation safety proof",
        "production": "Production source/include insertion",
        "disabled": "Disabled",
        "executor": "Executor wiring: Unwired",
        "status": "Production activation proof partially satisfied but default-disabled",
        "backup": "Byte-exact backup",
        "dry_run": "Dry-run write plan",
        "diff": "Diff preview",
        "reread": "Post-write reread",
        "restore": "Restore plan",
        "post_restore": "Post-restore verification",
        "no_auto_apply": "No auto-apply proof",
        "persistence_auto_apply": "Persisted-draft auto-apply proof",
        "final_approval": "Final approval still required",
        "run": "Run source/include production safety proof (fixture only, planned)",
        "enable": "Enable source/include production activation (not available)",
        "widget": "hyprland-settings-source-include-production-activation-safety-proof-disabled",
    },
    "duplicateReplacement": {
        "heading": "Duplicate production activation safety proof",
        "production": "Production duplicate writes",
        "disabled": "Disabled",
        "executor": "Executor wiring: Unwired",
        "status": "Production activation proof partially satisfied but default-disabled",
        "backup": "Byte-exact backup",
        "dry_run": "Dry-run write plan",
        "diff": "Diff preview",
        "reread": "Post-write reread",
        "restore": "Restore plan",
        "post_restore": "Post-restore verification",
        "no_auto_apply": "No auto-apply proof",
        "persistence_auto_apply": "Persisted-draft auto-apply proof",
        "final_approval": "Final approval still required",
        "run": "Run duplicate production safety proof (fixture only, planned)",
        "enable": "Enable duplicate production activation (not available)",
        "widget": "hyprland-settings-duplicate-production-activation-safety-proof-disabled",
    },
}

LEGACY_ACTIVATION_DRAFT_EDIT_ASSERTION_TEXT = [
    "Source/include activation draft editing",
    "Duplicate activation draft editing",
    "Editing mode",
    "Update source/include activation draft (planned)",
    "Reset source/include activation draft (planned)",
    "Update duplicate activation draft (planned)",
    "Reset duplicate activation draft (planned)",
]

SAFE_NAVIGATION_TARGETS = {
    "Dashboard",
    "Config",
    "Appearance",
    "Display",
    "Search",
    "FirstSafeSettingRow",
    "FirstBlockedSettingRow",
    "FirstDuplicateOrBlockedRow",
    "DuplicateConflictRow",
    "DuplicateConflictDetail",
    "MissingDefaultDetail",
    "GeneratedBlockedDetail",
    "ScriptManagedBlockedDetail",
    "SymlinkManagedBlockedDetail",
    "GeneratedConnectedFileDetail",
    "ScriptManagedConnectedFileDetail",
    "SymlinkConnectedFileDetail",
    "ProfileModeDetail",
    "HighRiskDetail",
    "DisplayRenderRiskDetail",
    "ProfileModeSwitchDetail",
    "DetailPane",
}

BLOCKED_CATEGORY_TARGETS = {
    "MissingDefaultDetail": {
        "category": "default_missing_line",
        "page": "Appearance",
        "terms": ["missing/default", "uses hyprland default", "default value"],
        "row_terms": ["missing/default setting row", "uses hyprland default"],
    },
    "GeneratedBlockedDetail": {
        "category": "generated_file",
        "page": "Config",
        "terms": ["generated", "do not edit"],
        "row_terms": ["generated", "do not edit"],
        "proof_surface": "config_page_text",
    },
    "GeneratedConnectedFileDetail": {
        "category": "generated_file",
        "page": "Config",
        "terms": [
            "generated file detail",
            "this file may be generated",
            "the app will not write it yet",
        ],
        "row_terms": [
            "hyprland-settings-connected-file-detail-generated",
            "generated connected-file blocker detail",
            "generated file detail",
        ],
        "proof_surface": "connected_file_detail",
    },
    "ScriptManagedBlockedDetail": {
        "category": "script_managed_file",
        "page": "Config",
        "terms": ["script", "changed by a script", "script-managed"],
        "row_terms": ["script", "changed by scripts", "changed by a script"],
        "proof_surface": "config_page_text",
    },
    "ScriptManagedConnectedFileDetail": {
        "category": "script_managed_file",
        "page": "Config",
        "terms": [
            "script-managed file detail",
            "this file may be changed by a script",
            "the app will not write it yet",
        ],
        "row_terms": [
            "hyprland-settings-connected-file-detail-script-managed",
            "script-managed connected-file blocker detail",
            "script-managed file detail",
        ],
        "proof_surface": "connected_file_detail",
    },
    "SymlinkManagedBlockedDetail": {
        "category": "symlink_current_profile",
        "page": "Config",
        "terms": ["symlink", "current-profile", "current profile"],
        "row_terms": ["symlink", "current-profile", "current profile"],
        "proof_surface": "config_page_text",
    },
    "SymlinkConnectedFileDetail": {
        "category": "symlink_current_profile",
        "page": "Config",
        "terms": [
            "symlink/current-profile detail",
            "this file may be a symlink or current-profile file",
            "the app will not write it yet",
        ],
        "row_terms": [
            "hyprland-settings-connected-file-detail-symlink-current-profile",
            "symlink current-profile connected-file blocker detail",
            "symlink/current-profile detail",
        ],
        "proof_surface": "connected_file_detail",
    },
    "HighRiskDetail": {
        "category": "high_risk",
        "page": "Display",
        "terms": ["high-risk", "extra care needed", "family-specific recovery"],
        "row_terms": ["high-risk setting row", "extra care needed"],
    },
    "DisplayRenderRiskDetail": {
        "category": "display_render_risk",
        "page": "Display",
        "terms": ["display/render", "screen shader", "render", "extra care needed"],
        "row_terms": ["display/render risk setting row", "screen shader", "render"],
    },
    "ProfileModeSwitchDetail": {
        "category": "profile_mode_switch",
        "page": "Config",
        "terms": ["profile", "mode", "current-profile", "symlink"],
        "row_terms": ["profile", "mode", "current-profile", "symlink"],
        "proof_surface": "config_page_text",
    },
    "ProfileModeDetail": {
        "category": "profile_mode_switch",
        "page": "Config",
        "terms": [
            "profile mode detail",
            "profile switching is not active yet",
            "the app will not change profile files or symlinks",
        ],
        "row_terms": [
            "hyprland-settings-profile-mode-detail",
            "profile mode detail",
            "profile switching is not active yet",
        ],
        "proof_surface": "profile_detail",
    },
}


def add_text(values, candidate):
    text = (candidate or "").strip()
    if text and text not in values:
        values.append(text)


def accessible_text(accessible):
    values = []
    add_text(values, getattr(accessible, "name", ""))
    add_text(values, getattr(accessible, "description", ""))
    try:
        text_iface = accessible.queryText()
        add_text(values, text_iface.getText(0, -1))
    except Exception:
        pass
    return values


def walk_accessible(accessible, values, max_nodes, seen):
    if len(seen) >= max_nodes:
        return
    try:
        key = hash(accessible)
    except Exception:
        key = id(accessible)
    if key in seen:
        return
    seen.add(key)

    for text in accessible_text(accessible):
        add_text(values, text)

    try:
        child_count = int(getattr(accessible, "childCount", 0))
    except Exception:
        child_count = 0
    for child_index in range(min(child_count, 500)):
        try:
            child = accessible.getChildAtIndex(child_index)
        except Exception:
            continue
        walk_accessible(child, values, max_nodes, seen)


def matches_expected_app(app, values):
    identity = "\n".join(accessible_text(app)).lower()
    if "firefox" in identity or "kitty" in identity or "steam" in identity:
        return False
    if "hyprland" in identity and ("settings" in identity or "config" in identity):
        return True
    lowered = "\n".join(values[:40]).lower()
    return "hyprland settings" in lowered or "hyprland-settings" in lowered


def found_terms(values):
    lowered = "\n".join(values).lower()
    return sorted(term for term in EXPECTED_TERMS if term in lowered)

def approval_card_assertions(values):
    text = "\n".join(values).lower()
    assertions = {}
    for key, spec in APPROVAL_CARD_ASSERTIONS.items():
        heading_found = spec["heading"].lower() in text
        production_found = spec["production"].lower() in text and spec["disabled"].lower() in text
        action_found = spec["action"].lower() in text
        widget_found = spec["widget"].lower() in text
        assertions[key] = {
            "heading": spec["heading"],
            "headingFound": heading_found,
            "productionDisabledText": spec["production"] + ": " + spec["disabled"],
            "productionDisabledFound": production_found,
            "disabledAction": spec["action"],
            "disabledActionFound": action_found,
            "widgetName": spec["widget"],
            "widgetNameFound": widget_found,
        }
    return assertions


def activation_decision_assertions(values):
    text = "\n".join(values).lower()
    assertions = {}
    for key, spec in ACTIVATION_DECISION_ASSERTIONS.items():
        heading_found = spec["heading"].lower() in text
        production_found = spec["production"].lower() in text and spec["disabled"].lower() in text
        action_found = spec["action"].lower() in text
        widget_found = spec["widget"].lower() in text
        assertions[key] = {
            "heading": spec["heading"],
            "headingFound": heading_found,
            "productionDisabledText": spec["production"] + ": " + spec["disabled"],
            "productionDisabledFound": production_found,
            "disabledAction": spec["action"],
            "disabledActionFound": action_found,
            "widgetName": spec["widget"],
            "widgetNameFound": widget_found,
        }
    return assertions


def activation_path_assertions(values):
    text = "\n".join(values).lower()
    assertions = {}
    for key, spec in ACTIVATION_PATH_ASSERTIONS.items():
        heading_found = spec["heading"].lower() in text
        production_found = spec["production"].lower() in text and spec["disabled"].lower() in text
        action_found = spec["action"].lower() in text
        widget_found = spec["widget"].lower() in text
        assertions[key] = {
            "heading": spec["heading"],
            "headingFound": heading_found,
            "productionDisabledText": spec["production"] + ": " + spec["disabled"],
            "productionDisabledFound": production_found,
            "disabledAction": spec["action"],
            "disabledActionFound": action_found,
            "widgetName": spec["widget"],
            "widgetNameFound": widget_found,
        }
    return assertions


def activation_control_assertions(values):
    text = "\n".join(values).lower()
    assertions = {}
    for key, spec in ACTIVATION_CONTROL_ASSERTIONS.items():
        heading_found = spec["heading"].lower() in text
        production_found = spec["production"].lower() in text and spec["disabled"].lower() in text
        executor_found = spec["executor"].lower() in text
        action_found = spec["action"].lower() in text
        widget_found = spec["widget"].lower() in text
        assertions[key] = {
            "heading": spec["heading"],
            "headingFound": heading_found,
            "productionDisabledText": spec["production"] + ": " + spec["disabled"],
            "productionDisabledFound": production_found,
            "executorWiring": spec["executor"],
            "executorWiringFound": executor_found,
            "disabledAction": spec["action"],
            "disabledActionFound": action_found,
            "widgetName": spec["widget"],
            "widgetNameFound": widget_found,
        }
    return assertions


def activation_form_assertions(values):
    text = "\n".join(values).lower()
    assertions = {}
    for key, spec in ACTIVATION_FORM_ASSERTIONS.items():
        heading_found = spec["heading"].lower() in text
        production_found = spec["production"].lower() in text and spec["disabled"].lower() in text
        executor_found = spec["executor"].lower() in text
        action_found = spec["action"].lower() in text
        widget_found = spec["widget"].lower() in text
        field_results = {
            field: field.lower() in text
            for field in spec.get("fields", [])
        }
        assertions[key] = {
            "heading": spec["heading"],
            "headingFound": heading_found,
            "productionDisabledText": spec["production"] + ": " + spec["disabled"],
            "productionDisabledFound": production_found,
            "executorWiring": spec["executor"],
            "executorWiringFound": executor_found,
            "disabledAction": spec["action"],
            "disabledActionFound": action_found,
            "widgetName": spec["widget"],
            "widgetNameFound": widget_found,
            "fieldLabels": spec.get("fields", []),
            "fieldLabelsFound": all(field_results.values()),
            "fieldLabelResults": field_results,
        }
    return assertions


def activation_draft_assertions(values):
    text = "\n".join(values).lower()
    assertions = {}
    for key, spec in ACTIVATION_DRAFT_ASSERTIONS.items():
        heading_found = spec["heading"].lower() in text
        production_found = spec["production"].lower() in text and spec["disabled"].lower() in text
        executor_found = spec["executor"].lower() in text
        memory_found = spec["memory"].lower() in text
        update_found = spec["update"].lower() in text
        reset_found = spec["reset"].lower() in text
        widget_found = spec["widget"].lower() in text
        assertions[key] = {
            "heading": spec["heading"],
            "headingFound": heading_found,
            "productionDisabledText": spec["production"] + ": " + spec["disabled"],
            "productionDisabledFound": production_found,
            "executorWiring": spec["executor"],
            "executorWiringFound": executor_found,
            "memoryStatus": spec["memory"],
            "memoryStatusFound": memory_found,
            "disabledUpdate": spec["update"],
            "disabledUpdateFound": update_found,
            "disabledReset": spec["reset"],
            "disabledResetFound": reset_found,
            "widgetName": spec["widget"],
            "widgetNameFound": widget_found,
        }
    return assertions


def activation_draft_edit_assertions(values):
    text = "\n".join(values).lower()
    assertions = {}
    assertions["legacyDraftEditSurface"] = {
        "expectedText": LEGACY_ACTIVATION_DRAFT_EDIT_ASSERTION_TEXT,
        "expectedTextFound": {
            expected: expected.lower() in text
            for expected in LEGACY_ACTIVATION_DRAFT_EDIT_ASSERTION_TEXT
        },
    }
    for key, spec in ACTIVATION_DRAFT_EDIT_ASSERTIONS.items():
        heading_found = spec["heading"].lower() in text
        production_found = spec["production"].lower() in text and spec["disabled"].lower() in text
        executor_found = spec["executor"].lower() in text
        memory_found = spec["memory"].lower() in text
        mode_found = spec["mode"].lower() in text
        not_saved_found = spec["not_saved"].lower() in text
        validation_found = spec["validation"].lower() in text
        update_found = spec["update"].lower() in text
        reset_found = spec["reset"].lower() in text
        widget_found = spec["widget"].lower() in text
        assertions[key] = {
            "heading": spec["heading"],
            "headingFound": heading_found,
            "productionDisabledText": spec["production"] + ": " + spec["disabled"],
            "productionDisabledFound": production_found,
            "executorWiring": spec["executor"],
            "executorWiringFound": executor_found,
            "memoryStatus": spec["memory"],
            "memoryStatusFound": memory_found,
            "editingMode": spec["mode"],
            "editingModeFound": mode_found,
            "notSavedStatus": spec["not_saved"],
            "notSavedStatusFound": not_saved_found,
            "draftValidation": spec["validation"],
            "draftValidationFound": validation_found,
            "disabledUpdate": spec["update"],
            "disabledUpdateFound": update_found,
            "disabledReset": spec["reset"],
            "disabledResetFound": reset_found,
            "widgetName": spec["widget"],
            "widgetNameFound": widget_found,
        }
    return assertions


def activation_draft_persistence_assertions(values):
    text = "\n".join(values).lower()
    assertions = {}
    for key, spec in ACTIVATION_DRAFT_PERSISTENCE_ASSERTIONS.items():
        assertions[key] = {
            "heading": spec["heading"],
            "headingFound": spec["heading"].lower() in text,
            "productionDisabledText": spec["production"] + ": " + spec["disabled"],
            "productionDisabledFound": spec["production"].lower() in text
            and spec["disabled"].lower() in text,
            "executorWiring": spec["executor"],
            "executorWiringFound": spec["executor"].lower() in text,
            "persistenceStatus": spec["status"],
            "persistenceStatusFound": spec["status"].lower() in text,
            "persistenceEnabled": spec["enabled"],
            "persistenceEnabledFound": spec["enabled"].lower() in text,
            "draftWritten": spec["written"],
            "draftWrittenFound": spec["written"].lower() in text,
            "storagePath": spec["storage"],
            "storagePathFound": spec["storage"].lower() in text,
            "disabledEnable": spec["enable"],
            "disabledEnableFound": spec["enable"].lower() in text,
            "disabledClear": spec["clear"],
            "disabledClearFound": spec["clear"].lower() in text,
            "widgetName": spec["widget"],
            "widgetNameFound": spec["widget"].lower() in text,
        }
    return assertions


def production_activation_safety_gate_assertions(values):
    text = "\n".join(values).lower()
    assertions = {}
    for key, spec in PRODUCTION_ACTIVATION_SAFETY_GATE_ASSERTIONS.items():
        assertions[key] = {
            "heading": spec["heading"],
            "headingFound": spec["heading"].lower() in text,
            "productionDisabledText": spec["production"] + ": " + spec["disabled"],
            "productionDisabledFound": spec["production"].lower() in text
            and spec["disabled"].lower() in text,
            "executorWiring": spec["executor"],
            "executorWiringFound": spec["executor"].lower() in text,
            "gateStatus": spec["status"],
            "gateStatusFound": spec["status"].lower() in text,
            "byteExactBackup": spec["backup"],
            "byteExactBackupFound": spec["backup"].lower() in text,
            "writePlan": spec["write"],
            "writePlanFound": spec["write"].lower() in text,
            "rereadPlan": spec["reread"],
            "rereadPlanFound": spec["reread"].lower() in text,
            "restorePlan": spec["restore"],
            "restorePlanFound": spec["restore"].lower() in text,
            "noAutoApplyProof": spec["no_auto_apply"],
            "noAutoApplyProofFound": spec["no_auto_apply"].lower() in text,
            "persistenceAutoApplyProof": spec["persistence_auto_apply"],
            "persistenceAutoApplyProofFound": spec["persistence_auto_apply"].lower() in text,
            "finalApproval": spec["final_approval"],
            "finalApprovalFound": spec["final_approval"].lower() in text,
            "disabledReview": spec["review"],
            "disabledReviewFound": spec["review"].lower() in text,
            "disabledEnable": spec["enable"],
            "disabledEnableFound": spec["enable"].lower() in text,
            "widgetName": spec["widget"],
            "widgetNameFound": spec["widget"].lower() in text,
        }
    return assertions


def production_activation_safety_proof_assertions(values):
    text = "\n".join(values).lower()
    assertions = {}
    for key, spec in PRODUCTION_ACTIVATION_SAFETY_PROOF_ASSERTIONS.items():
        assertions[key] = {
            "heading": spec["heading"],
            "headingFound": spec["heading"].lower() in text,
            "productionDisabledText": spec["production"] + ": " + spec["disabled"],
            "productionDisabledFound": spec["production"].lower() in text
            and spec["disabled"].lower() in text,
            "executorWiring": spec["executor"],
            "executorWiringFound": spec["executor"].lower() in text,
            "proofStatus": spec["status"],
            "proofStatusFound": spec["status"].lower() in text,
            "byteExactBackup": spec["backup"],
            "byteExactBackupFound": spec["backup"].lower() in text,
            "dryRunWritePlan": spec["dry_run"],
            "dryRunWritePlanFound": spec["dry_run"].lower() in text,
            "diffPreview": spec["diff"],
            "diffPreviewFound": spec["diff"].lower() in text,
            "postWriteReread": spec["reread"],
            "postWriteRereadFound": spec["reread"].lower() in text,
            "restorePlan": spec["restore"],
            "restorePlanFound": spec["restore"].lower() in text,
            "postRestoreVerification": spec["post_restore"],
            "postRestoreVerificationFound": spec["post_restore"].lower() in text,
            "noAutoApplyProof": spec["no_auto_apply"],
            "noAutoApplyProofFound": spec["no_auto_apply"].lower() in text,
            "persistenceAutoApplyProof": spec["persistence_auto_apply"],
            "persistenceAutoApplyProofFound": spec["persistence_auto_apply"].lower() in text,
            "finalApproval": spec["final_approval"],
            "finalApprovalFound": spec["final_approval"].lower() in text,
            "disabledRun": spec["run"],
            "disabledRunFound": spec["run"].lower() in text,
            "disabledEnable": spec["enable"],
            "disabledEnableFound": spec["enable"].lower() in text,
            "widgetName": spec["widget"],
            "widgetNameFound": spec["widget"].lower() in text,
        }
    return assertions


def node_text(node):
    return "\n".join(accessible_text(node))


def node_text_lower(node):
    return node_text(node).lower()


def has_apply_text(node):
    return "apply" in node_text_lower(node)


def safe_click_action(node):
    if has_apply_text(node):
        return False, "refused to click node containing Apply"
    try:
        action = node.queryAction()
        for action_index in range(action.nActions):
            name = action.getName(action_index).lower()
            if name in {"click", "press", "activate"}:
                action.doAction(action_index)
                return True, f"performed {name} action"
    except Exception as error:
        return False, f"no safe click action: {error}"
    return False, "no click/press/activate action available"


def first_clickable_button(node):
    try:
        role = node.getRoleName()
    except Exception:
        role = ""
    if role in {"push button", "button"}:
        try:
            action = node.queryAction()
            if any(action.getName(i) == "click" for i in range(action.nActions)):
                return node
        except Exception:
            pass
    try:
        child_count = int(getattr(node, "childCount", 0))
    except Exception:
        child_count = 0
    for child_index in range(min(child_count, 200)):
        try:
            found = first_clickable_button(node.getChildAtIndex(child_index))
        except Exception:
            found = None
        if found is not None:
            return found
    return None


def find_first_node(app, predicate, max_nodes=1500):
    seen = set()

    def recurse(node):
        if len(seen) >= max_nodes:
            return None
        try:
            key = hash(node)
        except Exception:
            key = id(node)
        if key in seen:
            return None
        seen.add(key)
        try:
            if predicate(node):
                return node
        except Exception:
            pass
        try:
            child_count = int(getattr(node, "childCount", 0))
        except Exception:
            child_count = 0
        for child_index in range(min(child_count, 500)):
            try:
                found = recurse(node.getChildAtIndex(child_index))
            except Exception:
                found = None
            if found is not None:
                return found
        return None

    return recurse(app)


def find_first_node_with_parent(app, predicate, max_nodes=1500):
    seen = set()

    def recurse(node, parent):
        if len(seen) >= max_nodes:
            return None, None
        try:
            key = hash(node)
        except Exception:
            key = id(node)
        if key in seen:
            return None, None
        seen.add(key)
        try:
            if predicate(node):
                return node, parent
        except Exception:
            pass
        try:
            child_count = int(getattr(node, "childCount", 0))
        except Exception:
            child_count = 0
        for child_index in range(min(child_count, 500)):
            try:
                found, found_parent = recurse(node.getChildAtIndex(child_index), node)
            except Exception:
                found, found_parent = None, None
            if found is not None:
                return found, found_parent
        return None, None

    return recurse(app, None)


def safe_row_candidate(node, blocked):
    text = node_text_lower(node)
    if not text:
        return False
    if "apply" in text:
        return False
    if "setting row:" in text:
        has_blocked = any(
            marker in text
            for marker in [
                "uses hyprland default",
                "needs attention",
                "extra care needed",
                "blocked",
                "duplicate",
                "generated",
                "script",
                "symlink",
                "high-risk",
            ]
        )
        return has_blocked if blocked else not has_blocked
    return False


def safe_select_node(node, parent):
    ok, message = safe_click_action(node)
    if ok:
        return ok, message
    if parent is None:
        return False, f"{message}; no parent selection fallback available"
    if has_apply_text(parent):
        return False, "refused parent selection because parent contains Apply"
    try:
        selection = parent.querySelection()
        index = node.getIndexInParent()
        selection.selectChild(index)
        return True, f"selected child {index} through parent selection fallback"
    except Exception as error:
        return False, f"{message}; parent selection fallback failed: {error}"


def duplicate_row_candidate(node):
    text = node_text_lower(node)
    if not text or "apply" in text:
        return False
    return (
        "duplicate conflict setting row" in text
        or "this setting appears more than once in your config" in text
        or ("appearance blur enabled" in text and "needs attention" in text)
    )


def blocked_category_row_candidate(node, target):
    spec = BLOCKED_CATEGORY_TARGETS.get(target)
    if spec is None:
        return False
    text = node_text_lower(node)
    if not text or "apply" in text:
        return False
    return any(term in text for term in spec["row_terms"])


def blocked_category_text_collected(values, target):
    spec = BLOCKED_CATEGORY_TARGETS.get(target)
    if spec is None:
        return False
    text = "\n".join(values).lower()
    return any(term in text for term in spec["terms"])


def open_duplicate_conflict_detail(app):
    ok, message = click_named_target(app, "Appearance")
    if not ok:
        return False, f"could not open Appearance before duplicate detail: {message}"
    time.sleep(1)
    node, parent = find_first_node_with_parent(app, duplicate_row_candidate)
    if node is None:
        return False, "no duplicate-conflict setting row found"
    ok, message = safe_select_node(node, parent)
    if not ok:
        return False, f"duplicate-conflict row activation failed: {message}"
    return True, f"opened duplicate-conflict row detail: {message}"


def open_blocked_category_detail(app, target):
    spec = BLOCKED_CATEGORY_TARGETS[target]
    page = spec["page"]
    ok, message = click_named_target(app, page)
    if not ok:
        return False, f"could not open {page} before {target}: {message}"
    time.sleep(1)
    node, parent = find_first_node_with_parent(
        app, lambda current: blocked_category_row_candidate(current, target)
    )
    if node is None:
        values = []
        walk_accessible(app, values, 3000, set())
        if blocked_category_text_collected(values, target):
            return True, f"{target} blocker text found without row activation"
        return False, f"no allowlisted blocked-category row found for {target}"
    ok, message = safe_select_node(node, parent)
    if not ok:
        values = []
        walk_accessible(app, values, 3000, set())
        if blocked_category_text_collected(values, target):
            return True, f"{target} blocker text found after safe row activation failed: {message}"
        return False, f"{target} row activation failed: {message}"
    return True, f"opened {target} row detail: {message}"


def click_named_target(app, target):
    if target.lower() == "apply":
        return False, "refused to navigate to Apply"
    if target not in SAFE_NAVIGATION_TARGETS:
        return False, f"unsupported navigation target: {target}"
    if target == "Search":
        node = find_first_node(
            app,
            lambda current: "search settings" in node_text_lower(current)
            or "hyprland-settings-search" in node_text_lower(current),
        )
        return (node is not None), (
            "Search field found" if node is not None else "Search field not found"
        )
    if target == "DetailPane":
        node = find_first_node(
            app,
            lambda current: "setting details" in node_text_lower(current)
            or "hyprland-settings-detail-pane" in node_text_lower(current),
        )
        return (node is not None), (
            "Detail pane found" if node is not None else "Detail pane not found"
        )
    if target == "DuplicateConflictDetail":
        return open_duplicate_conflict_detail(app)
    if target in BLOCKED_CATEGORY_TARGETS:
        return open_blocked_category_detail(app, target)
    if target == "DuplicateConflictRow":
        node, parent = find_first_node_with_parent(app, duplicate_row_candidate)
        if node is None:
            return False, "no duplicate-conflict setting row found"
        return safe_select_node(node, parent)
    if target in {
        "FirstSafeSettingRow",
        "FirstBlockedSettingRow",
        "FirstDuplicateOrBlockedRow",
    }:
        blocked = target != "FirstSafeSettingRow"
        if target == "FirstDuplicateOrBlockedRow":
            node, parent = find_first_node_with_parent(app, duplicate_row_candidate)
            if node is None:
                node, parent = find_first_node_with_parent(
                    app, lambda current: safe_row_candidate(current, True)
                )
        else:
            node, parent = find_first_node_with_parent(
                app, lambda current: safe_row_candidate(current, blocked)
            )
        if node is None:
            return False, f"no safe row target found for {target}"
        return safe_select_node(node, parent)

    def recurse(node):
        values = accessible_text(node)
        try:
            role = node.getRoleName()
        except Exception:
            role = ""
        if role == "table cell" and target in "\n".join(values):
            button = first_clickable_button(node)
            if button is not None:
                return button
        try:
            child_count = int(getattr(node, "childCount", 0))
        except Exception:
            child_count = 0
        for child_index in range(min(child_count, 500)):
            try:
                found = recurse(node.getChildAtIndex(child_index))
            except Exception:
                found = None
            if found is not None:
                return found
        return None

    button = recurse(app)
    if button is None:
        return False, f"no accessible Open button found for {target}"
    return safe_click_action(button)


def main() -> int:
    output = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("/tmp/hyprland-settings-atspi.json")
    result = {
        "attempted": True,
        "method": "gdbus-org.a11y.Bus",
        "pyatspiAvailable": False,
        "gdbusAvailable": shutil.which("gdbus") is not None,
        "succeeded": False,
        "applicationMatched": False,
        "navigationAttempted": False,
        "navigationTarget": None,
        "navigationSucceeded": False,
        "navigationMessage": None,
        "applyRefused": True,
        "textAfterNavigation": [],
        "foundTerms": [],
        "foundTermsAfterNavigation": [],
        "detailPaneTextCollected": False,
        "blockedReasonTextCollected": False,
        "duplicateBlockedReasonTextCollected": False,
        "blockedCategory": None,
        "blockedCategoryDetailNavigationAttempted": False,
        "blockedCategoryDetailNavigationSucceeded": False,
        "blockedCategoryReasonTextCollected": False,
        "blockedCategoryExpectedTextCollected": False,
        "blockedCategorySelectionFallbackUsed": False,
        "connectedFileDetailNavigationAttempted": False,
        "connectedFileDetailNavigationSucceeded": False,
        "connectedFileGeneratedDetailCollected": False,
        "connectedFileScriptManagedDetailCollected": False,
        "connectedFileSymlinkDetailCollected": False,
        "profileModeDetailCollected": False,
        "proofSurface": None,
        "duplicateConflictDetailNavigationAttempted": False,
        "duplicateConflictDetailNavigationSucceeded": False,
        "forbiddenApplyActionSeen": False,
        "fallbackProofUsed": False,
        "approvalCardAssertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        "approvalCardAssertions": {},
        "approvalCardsAllHeadingsFound": False,
        "approvalCardsAllProductionDisabledFound": False,
        "approvalCardsAllDisabledActionsFound": False,
        "activationDecisionAssertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        "activationDecisionAssertions": {},
        "activationDecisionsAllHeadingsFound": False,
        "activationDecisionsAllProductionDisabledFound": False,
        "activationDecisionsAllDisabledActionsFound": False,
        "activationPathAssertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        "activationPathAssertions": {},
        "activationPathsAllHeadingsFound": False,
        "activationPathsAllProductionDisabledFound": False,
        "activationPathsAllDisabledActionsFound": False,
        "activationControlAssertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        "activationControlAssertions": {},
        "activationControlsAllHeadingsFound": False,
        "activationControlsAllProductionDisabledFound": False,
        "activationControlsAllExecutorUnwiredFound": False,
        "activationControlsAllDisabledActionsFound": False,
        "activationFormAssertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        "activationFormAssertions": {},
        "activationFormsAllHeadingsFound": False,
        "activationFormsAllProductionDisabledFound": False,
        "activationFormsAllExecutorUnwiredFound": False,
        "activationFormsAllDisabledActionsFound": False,
        "activationFormsAllFieldLabelsFound": False,
        "activationDraftAssertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        "activationDraftAssertions": {},
        "activationDraftsAllHeadingsFound": False,
        "activationDraftsAllProductionDisabledFound": False,
        "activationDraftsAllExecutorUnwiredFound": False,
        "activationDraftsAllInMemoryOnlyFound": False,
        "activationDraftsAllDisabledActionsFound": False,
        "activationDraftEditAssertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        "activationDraftEditAssertions": {},
        "activationDraftEditsAllHeadingsFound": False,
        "activationDraftEditsAllProductionDisabledFound": False,
        "activationDraftEditsAllExecutorUnwiredFound": False,
        "activationDraftEditsAllInMemoryOnlyFound": False,
        "activationDraftEditsAllModeFound": False,
        "activationDraftEditsAllValidationFound": False,
        "activationDraftEditsAllDisabledActionsFound": False,
        "activationDraftPersistenceAssertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        "activationDraftPersistenceAssertions": {},
        "activationDraftPersistenceAllHeadingsFound": False,
        "activationDraftPersistenceAllProductionDisabledFound": False,
        "activationDraftPersistenceAllExecutorUnwiredFound": False,
        "activationDraftPersistenceAllForbiddenFound": False,
        "activationDraftPersistenceAllStorageAbsentFound": False,
        "activationDraftPersistenceAllDisabledActionsFound": False,
        "productionActivationSafetyGateAssertionMethod": "screenshot_plus_accessibility_tree_text_not_ocr",
        "productionActivationSafetyGateAssertions": {},
        "productionActivationSafetyGatesAllHeadingsFound": False,
        "productionActivationSafetyGatesAllProductionDisabledFound": False,
        "productionActivationSafetyGatesAllExecutorUnwiredFound": False,
        "productionActivationSafetyGatesAllBlockedByDefaultFound": False,
        "productionActivationSafetyGatesAllRequiredProofFound": False,
        "productionActivationSafetyGatesAllDisabledActionsFound": False,
        "productionActivationSafetyProofAssertions": {},
        "productionActivationSafetyProofsAllHeadingsFound": False,
        "productionActivationSafetyProofsAllProductionDisabledFound": False,
        "productionActivationSafetyProofsAllExecutorUnwiredFound": False,
        "productionActivationSafetyProofsAllProofStatusFound": False,
        "productionActivationSafetyProofsAllCopiedFixtureProofFound": False,
        "productionActivationSafetyProofsAllNoAutoApplyFound": False,
        "productionActivationSafetyProofsAllFinalApprovalFound": False,
        "productionActivationSafetyProofsAllDisabledActionsFound": False,
        "text": [],
        "error": None,
    }
    try:
        import pyatspi  # type: ignore

        result["pyatspiAvailable"] = True
        desktop = pyatspi.Registry.getDesktop(0)
        matching_apps = []
        selected_app = None
        for app_index in range(desktop.childCount):
            app = desktop.getChildAtIndex(app_index)
            texts = []
            walk_accessible(app, texts, 3000, set())
            if matches_expected_app(app, texts):
                if selected_app is None:
                    selected_app = app
                matching_apps.append(
                    {
                        "applicationName": getattr(app, "name", "") or "",
                        "text": texts,
                        "foundTerms": found_terms(texts),
                    }
                )
        result["applicationsMatched"] = len(matching_apps)
        result["applicationMatched"] = bool(matching_apps)
        result["text"] = matching_apps[0]["text"] if matching_apps else []
        result["foundTerms"] = matching_apps[0]["foundTerms"] if matching_apps else []
        result["succeeded"] = bool(matching_apps and matching_apps[0]["text"])
        nav_target = os.environ.get("HYPR_SETTINGS_ATSPI_NAV_TARGET")
        result["navigationAttempted"] = bool(nav_target)
        result["navigationTarget"] = nav_target or None
        if nav_target and selected_app is not None:
            if nav_target in BLOCKED_CATEGORY_TARGETS:
                spec = BLOCKED_CATEGORY_TARGETS[nav_target]
                result["blockedCategory"] = spec["category"]
                result["proofSurface"] = spec.get("proof_surface")
                result["blockedCategoryDetailNavigationAttempted"] = True
                result["connectedFileDetailNavigationAttempted"] = spec.get("proof_surface") in {
                    "connected_file_detail",
                    "profile_detail",
                }
            result["duplicateConflictDetailNavigationAttempted"] = nav_target in {
                "DuplicateConflictDetail",
                "DuplicateConflictRow",
                "FirstDuplicateOrBlockedRow",
            }
            ok, message = click_named_target(selected_app, nav_target)
            result["navigationSucceeded"] = ok
            result["navigationMessage"] = message
            result["duplicateConflictDetailNavigationSucceeded"] = (
                ok and nav_target == "DuplicateConflictDetail"
            )
            result["blockedCategoryDetailNavigationSucceeded"] = (
                ok and nav_target in BLOCKED_CATEGORY_TARGETS
            )
            result["connectedFileDetailNavigationSucceeded"] = (
                ok
                and nav_target in BLOCKED_CATEGORY_TARGETS
                and BLOCKED_CATEGORY_TARGETS[nav_target].get("proof_surface")
                in {"connected_file_detail", "profile_detail"}
            )
            result["blockedCategorySelectionFallbackUsed"] = "selection fallback" in message
            if ok:
                time.sleep(1)
                after = []
                walk_accessible(selected_app, after, 3000, set())
                result["textAfterNavigation"] = after
                result["foundTermsAfterNavigation"] = found_terms(after)
        all_text = "\n".join(result["text"] + result["textAfterNavigation"]).lower()
        approval_assertions = approval_card_assertions(result["text"] + result["textAfterNavigation"])
        activation_assertions = activation_decision_assertions(
            result["text"] + result["textAfterNavigation"]
        )
        path_assertions = activation_path_assertions(result["text"] + result["textAfterNavigation"])
        control_assertions = activation_control_assertions(
            result["text"] + result["textAfterNavigation"]
        )
        form_assertions = activation_form_assertions(result["text"] + result["textAfterNavigation"])
        draft_assertions = activation_draft_assertions(
            result["text"] + result["textAfterNavigation"]
        )
        draft_edit_assertions = activation_draft_edit_assertions(
            result["text"] + result["textAfterNavigation"]
        )
        draft_persistence_assertions = activation_draft_persistence_assertions(
            result["text"] + result["textAfterNavigation"]
        )
        safety_gate_assertions = production_activation_safety_gate_assertions(
            result["text"] + result["textAfterNavigation"]
        )
        safety_proof_assertions = production_activation_safety_proof_assertions(
            result["text"] + result["textAfterNavigation"]
        )
        result["approvalCardAssertions"] = approval_assertions
        result["approvalCardsAllHeadingsFound"] = all(
            card["headingFound"] for card in approval_assertions.values()
        )
        result["approvalCardsAllProductionDisabledFound"] = all(
            card["productionDisabledFound"] for card in approval_assertions.values()
        )
        result["approvalCardsAllDisabledActionsFound"] = all(
            card["disabledActionFound"] for card in approval_assertions.values()
        )
        result["activationDecisionAssertions"] = activation_assertions
        result["activationDecisionsAllHeadingsFound"] = all(
            card["headingFound"] for card in activation_assertions.values()
        )
        result["activationDecisionsAllProductionDisabledFound"] = all(
            card["productionDisabledFound"] for card in activation_assertions.values()
        )
        result["activationDecisionsAllDisabledActionsFound"] = all(
            card["disabledActionFound"] for card in activation_assertions.values()
        )
        result["activationPathAssertions"] = path_assertions
        result["activationPathsAllHeadingsFound"] = all(
            card["headingFound"] for card in path_assertions.values()
        )
        result["activationPathsAllProductionDisabledFound"] = all(
            card["productionDisabledFound"] for card in path_assertions.values()
        )
        result["activationPathsAllDisabledActionsFound"] = all(
            card["disabledActionFound"] for card in path_assertions.values()
        )
        result["activationControlAssertions"] = control_assertions
        result["activationControlsAllHeadingsFound"] = all(
            card["headingFound"] for card in control_assertions.values()
        )
        result["activationControlsAllProductionDisabledFound"] = all(
            card["productionDisabledFound"] for card in control_assertions.values()
        )
        result["activationControlsAllExecutorUnwiredFound"] = all(
            card["executorWiringFound"] for card in control_assertions.values()
        )
        result["activationControlsAllDisabledActionsFound"] = all(
            card["disabledActionFound"] for card in control_assertions.values()
        )
        result["activationFormAssertions"] = form_assertions
        result["activationFormsAllHeadingsFound"] = all(
            card["headingFound"] for card in form_assertions.values()
        )
        result["activationFormsAllProductionDisabledFound"] = all(
            card["productionDisabledFound"] for card in form_assertions.values()
        )
        result["activationFormsAllExecutorUnwiredFound"] = all(
            card["executorWiringFound"] for card in form_assertions.values()
        )
        result["activationFormsAllDisabledActionsFound"] = all(
            card["disabledActionFound"] for card in form_assertions.values()
        )
        result["activationFormsAllFieldLabelsFound"] = all(
            card["fieldLabelsFound"] for card in form_assertions.values()
        )
        result["activationDraftAssertions"] = draft_assertions
        result["activationDraftsAllHeadingsFound"] = all(
            card["headingFound"] for card in draft_assertions.values()
        )
        result["activationDraftsAllProductionDisabledFound"] = all(
            card["productionDisabledFound"] for card in draft_assertions.values()
        )
        result["activationDraftsAllExecutorUnwiredFound"] = all(
            card["executorWiringFound"] for card in draft_assertions.values()
        )
        result["activationDraftsAllInMemoryOnlyFound"] = all(
            card["memoryStatusFound"] for card in draft_assertions.values()
        )
        result["activationDraftsAllDisabledActionsFound"] = all(
            card["disabledUpdateFound"] and card["disabledResetFound"]
            for card in draft_assertions.values()
        )
        result["activationDraftEditAssertions"] = draft_edit_assertions
        result["activationDraftEditsAllHeadingsFound"] = all(
            card.get("headingFound", True) for card in draft_edit_assertions.values()
        )
        result["activationDraftEditsAllProductionDisabledFound"] = all(
            card.get("productionDisabledFound", True)
            for card in draft_edit_assertions.values()
        )
        result["activationDraftEditsAllExecutorUnwiredFound"] = all(
            card.get("executorWiringFound", True) for card in draft_edit_assertions.values()
        )
        result["activationDraftEditsAllInMemoryOnlyFound"] = all(
            card.get("memoryStatusFound", True) for card in draft_edit_assertions.values()
        )
        result["activationDraftEditsAllModeFound"] = all(
            card.get("editingModeFound", True) for card in draft_edit_assertions.values()
        )
        result["activationDraftEditsAllValidationFound"] = all(
            card.get("draftValidationFound", True) for card in draft_edit_assertions.values()
        )
        result["activationDraftEditsAllDisabledActionsFound"] = all(
            card.get("disabledUpdateFound", True) and card.get("disabledResetFound", True)
            for card in draft_edit_assertions.values()
        )
        result["activationDraftPersistenceAssertions"] = draft_persistence_assertions
        result["activationDraftPersistenceAllHeadingsFound"] = all(
            card["headingFound"] for card in draft_persistence_assertions.values()
        )
        result["activationDraftPersistenceAllProductionDisabledFound"] = all(
            card["productionDisabledFound"] for card in draft_persistence_assertions.values()
        )
        result["activationDraftPersistenceAllExecutorUnwiredFound"] = all(
            card["executorWiringFound"] for card in draft_persistence_assertions.values()
        )
        result["activationDraftPersistenceAllForbiddenFound"] = all(
            card["persistenceStatusFound"] and card["persistenceEnabledFound"]
            for card in draft_persistence_assertions.values()
        )
        result["activationDraftPersistenceAllStorageAbsentFound"] = all(
            card["draftWrittenFound"] and card["storagePathFound"]
            for card in draft_persistence_assertions.values()
        )
        result["activationDraftPersistenceAllDisabledActionsFound"] = all(
            card["disabledEnableFound"] and card["disabledClearFound"]
            for card in draft_persistence_assertions.values()
        )
        result["productionActivationSafetyGateAssertions"] = safety_gate_assertions
        result["productionActivationSafetyGatesAllHeadingsFound"] = all(
            card["headingFound"] for card in safety_gate_assertions.values()
        )
        result["productionActivationSafetyGatesAllProductionDisabledFound"] = all(
            card["productionDisabledFound"] for card in safety_gate_assertions.values()
        )
        result["productionActivationSafetyGatesAllExecutorUnwiredFound"] = all(
            card["executorWiringFound"] for card in safety_gate_assertions.values()
        )
        result["productionActivationSafetyGatesAllBlockedByDefaultFound"] = all(
            card["gateStatusFound"] for card in safety_gate_assertions.values()
        )
        result["productionActivationSafetyGatesAllRequiredProofFound"] = all(
            card["byteExactBackupFound"]
            and card["writePlanFound"]
            and card["rereadPlanFound"]
            and card["restorePlanFound"]
            and card["noAutoApplyProofFound"]
            and card["persistenceAutoApplyProofFound"]
            and card["finalApprovalFound"]
            for card in safety_gate_assertions.values()
        )
        result["productionActivationSafetyGatesAllDisabledActionsFound"] = all(
            card["disabledReviewFound"] and card["disabledEnableFound"]
            for card in safety_gate_assertions.values()
        )
        result["productionActivationSafetyProofAssertions"] = safety_proof_assertions
        result["productionActivationSafetyProofsAllHeadingsFound"] = all(
            card["headingFound"] for card in safety_proof_assertions.values()
        )
        result["productionActivationSafetyProofsAllProductionDisabledFound"] = all(
            card["productionDisabledFound"] for card in safety_proof_assertions.values()
        )
        result["productionActivationSafetyProofsAllExecutorUnwiredFound"] = all(
            card["executorWiringFound"] for card in safety_proof_assertions.values()
        )
        result["productionActivationSafetyProofsAllProofStatusFound"] = all(
            card["proofStatusFound"] for card in safety_proof_assertions.values()
        )
        result["productionActivationSafetyProofsAllCopiedFixtureProofFound"] = all(
            card["byteExactBackupFound"]
            and card["dryRunWritePlanFound"]
            and card["diffPreviewFound"]
            and card["postWriteRereadFound"]
            and card["restorePlanFound"]
            and card["postRestoreVerificationFound"]
            for card in safety_proof_assertions.values()
        )
        result["productionActivationSafetyProofsAllNoAutoApplyFound"] = all(
            card["noAutoApplyProofFound"] and card["persistenceAutoApplyProofFound"]
            for card in safety_proof_assertions.values()
        )
        result["productionActivationSafetyProofsAllFinalApprovalFound"] = all(
            card["finalApprovalFound"] for card in safety_proof_assertions.values()
        )
        result["productionActivationSafetyProofsAllDisabledActionsFound"] = all(
            card["disabledRunFound"] and card["disabledEnableFound"]
            for card in safety_proof_assertions.values()
        )
        duplicate_text_collected = (
            "this setting appears more than once in your config" in all_text
            and "will not write this setting until the duplicate entries are resolved manually"
            in all_text
        )
        result["detailPaneTextCollected"] = "setting details" in all_text or duplicate_text_collected
        result["blockedReasonTextCollected"] = any(
            marker in all_text
            for marker in [
                "blocked",
                "uses hyprland default",
                "needs attention",
                "extra care needed",
                "duplicate",
                "generated",
                "script",
                "symlink",
                "high-risk",
            ]
        )
        result["duplicateBlockedReasonTextCollected"] = duplicate_text_collected
        if result["navigationTarget"] in BLOCKED_CATEGORY_TARGETS:
            result["blockedCategoryExpectedTextCollected"] = blocked_category_text_collected(
                result["text"] + result["textAfterNavigation"],
                result["navigationTarget"],
            )
            result["blockedCategoryReasonTextCollected"] = (
                result["blockedCategoryExpectedTextCollected"]
                or result["blockedReasonTextCollected"]
            )
            category = result["blockedCategory"]
            proof_surface = result["proofSurface"]
            if proof_surface == "connected_file_detail":
                result["connectedFileGeneratedDetailCollected"] = (
                    category == "generated_file"
                    and result["blockedCategoryExpectedTextCollected"]
                )
                result["connectedFileScriptManagedDetailCollected"] = (
                    category == "script_managed_file"
                    and result["blockedCategoryExpectedTextCollected"]
                )
                result["connectedFileSymlinkDetailCollected"] = (
                    category == "symlink_current_profile"
                    and result["blockedCategoryExpectedTextCollected"]
                )
            if proof_surface == "profile_detail":
                result["profileModeDetailCollected"] = result[
                    "blockedCategoryExpectedTextCollected"
                ]
    except Exception as error:
        result["error"] = f"pyatspi unavailable or inaccessible: {error}"
        if result["gdbusAvailable"]:
            try:
                probe = subprocess.run(
                    [
                        "gdbus",
                        "call",
                        "--session",
                        "--dest",
                        "org.a11y.Bus",
                        "--object-path",
                        "/org/a11y/bus",
                        "--method",
                        "org.a11y.Bus.GetAddress",
                    ],
                    check=False,
                    text=True,
                    stdout=subprocess.PIPE,
                    stderr=subprocess.PIPE,
                    timeout=3,
                )
                result["gdbusOrgA11yBusExitStatus"] = probe.returncode
                result["gdbusOrgA11yBusStdoutPresent"] = bool(probe.stdout.strip())
                result["gdbusOrgA11yBusStderr"] = probe.stderr.strip()
                result["atspiBusReachable"] = probe.returncode == 0 and bool(probe.stdout.strip())
                result["succeeded"] = False
            except Exception as gdbus_error:
                result["gdbusError"] = str(gdbus_error)

    output.parent.mkdir(parents=True, exist_ok=True)
    output.write_text(json.dumps(result, indent=2) + "\n")
    return 0 if result["succeeded"] else 1


if __name__ == "__main__":
    raise SystemExit(main())
