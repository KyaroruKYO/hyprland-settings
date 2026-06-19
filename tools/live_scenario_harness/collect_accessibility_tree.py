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
]


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


def click_named_card(app, target):
    if target.lower() == "apply":
        return False, "refused to navigate to Apply"
    if target not in {"Dashboard", "Config", "Appearance", "Display"}:
        return False, f"unsupported navigation target: {target}"

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
    try:
        action = button.queryAction()
        for action_index in range(action.nActions):
            if action.getName(action_index) == "click":
                action.doAction(action_index)
                return True, f"clicked {target} Open button"
    except Exception as error:
        return False, f"click action failed for {target}: {error}"
    return False, f"no click action available for {target}"


def main() -> int:
    output = Path(sys.argv[1]) if len(sys.argv) > 1 else Path("/tmp/hyprland-settings-atspi.json")
    result = {
        "attempted": True,
        "method": "gdbus-org.a11y.Bus",
        "pyatspiAvailable": False,
        "gdbusAvailable": shutil.which("gdbus") is not None,
        "succeeded": False,
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
            walk_accessible(app, texts, 1200, set())
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
        result["text"] = matching_apps[0]["text"] if matching_apps else []
        result["foundTerms"] = matching_apps[0]["foundTerms"] if matching_apps else []
        result["succeeded"] = bool(matching_apps and matching_apps[0]["text"])
        nav_target = os.environ.get("HYPR_SETTINGS_ATSPI_NAV_TARGET")
        result["navigationAttempted"] = bool(nav_target)
        result["navigationTarget"] = nav_target or None
        result["navigationSucceeded"] = False
        result["navigationMessage"] = None
        if nav_target and selected_app is not None:
            ok, message = click_named_card(selected_app, nav_target)
            result["navigationSucceeded"] = ok
            result["navigationMessage"] = message
            if ok:
                time.sleep(1)
                after = []
                walk_accessible(selected_app, after, 1200, set())
                result["textAfterNavigation"] = after
                result["foundTermsAfterNavigation"] = found_terms(after)
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
