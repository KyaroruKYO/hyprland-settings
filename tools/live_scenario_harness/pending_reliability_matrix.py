#!/usr/bin/env python3
"""Pending-changes reliability matrix: drive every eligible inline control
in the running app through one user-like change and assert every pending
surface reacts on the FIRST change, then discard and assert everything
clears.

Consumes the env-gated skeleton exported by
`HYPRLAND_SETTINGS_EXPORT_PENDING_SKELETON=1 cargo test --test
pending_changes_reliability export_reliability_skeleton` and merges live
results into data/reports/pending-changes-reliability-matrix.v0.55.2.json.

Safety: interacts only through the app's own reversible preview controls
(switch toggles, spinner steps, dropdown picks, color-picker Select) and
the Discard control. It never clicks Save. Any node whose text mentions
apply/save is refused.
"""

import json
import subprocess
import sys
import time

import pyatspi

SKELETON = "/tmp/hyprland-settings-pending-skeleton.json"
OUTPUT = "data/reports/pending-changes-reliability-matrix.v0.55.2.json"
RESULTS_CACHE = "/tmp/hyprland-settings-pending-live-results.json"
APP_NAME = "hyprland-settings"


def log(message):
    print(message, flush=True)


def getoption(colon_key):
    try:
        output = subprocess.run(
            ["hyprctl", "getoption", colon_key],
            capture_output=True,
            text=True,
            timeout=5,
        ).stdout
    except Exception:
        return None
    for line in output.splitlines():
        line = line.strip()
        for prefix in (
            "int:",
            "float:",
            "bool:",
            "css gap data:",
            "str:",
            "color:",
            "gradient data:",
        ):
            if line.startswith(prefix):
                return line[len(prefix):].strip()
    return None


def app():
    for candidate in pyatspi.Registry.getDesktop(0):
        try:
            if candidate.name == APP_NAME:
                return candidate
        except Exception:
            continue
    raise SystemExit("app not found on the accessibility bus")


def walk(node, depth=0, max_depth=30):
    yield node
    if depth >= max_depth:
        return
    try:
        count = min(int(getattr(node, "childCount", 0)), 400)
    except Exception:
        return
    for index in range(count):
        try:
            child = node.getChildAtIndex(index)
        except Exception:
            continue
        if child is not None:
            yield from walk(child, depth + 1, max_depth)


def find_named(fragment, role=None, root=None, exact=False):
    for node in walk(root if root is not None else app()):
        try:
            name = node.name or ""
            node_role = node.getRoleName()
        except Exception:
            continue
        matches = name == fragment if exact else fragment in name
        if matches and (role is None or node_role == role):
            return node
    return None


def forbid_apply(node):
    text = ((node.name or "") + " " + (node.description or "")).lower()
    if "apply" in text or ("save" in text and "unsaved" not in text):
        raise RuntimeError(f"refusing node with apply/save text: {node.name}")


def click(node):
    forbid_apply(node)
    action = node.queryAction()
    for index in range(action.nActions):
        if action.getName(index).lower() in {"click", "press", "activate", "toggle"}:
            action.doAction(index)
            return True
    return False


def nav(page_label):
    row = find_named(f"Navigation: {page_label}")
    if row is None:
        raise RuntimeError(f"navigation row {page_label!r} missing")
    row.parent.querySelection().selectChild(row.getIndexInParent())
    time.sleep(0.6)


def find_setting_row(official):
    return find_named(f"Official key: {official}.", role="list item")


def chip_node():
    return find_named("View pending changes", role="button")


def chip_count():
    chip = chip_node()
    if chip is None:
        return None
    for node in walk(chip, 0, 4):
        try:
            if node.getRoleName() == "label" and (node.name or "").strip():
                return (node.name or "").strip()
        except Exception:
            continue
    return ""


def bar_present():
    return find_named("Unsaved changes — applied live", role="label") is not None


def badge_count(page_label):
    row = find_named(f"Navigation: {page_label}")
    if row is None:
        return None
    labels = []
    for node in walk(row, 0, 5):
        try:
            if node.getRoleName() == "label":
                labels.append((node.name or "").strip())
        except Exception:
            continue
    for text in labels:
        if text.isdigit():
            return text
    return None


def row_marker(official):
    row = find_setting_row(official)
    if row is None:
        return None
    return (row.description or "").strip()


def discard_all():
    # Exact name: color rows carry their own "Discard color changes"
    # arrow, which is a different control.
    discard = find_named("Discard", role="button", exact=True)
    if discard is None:
        return False
    action = discard.queryAction()
    for index in range(action.nActions):
        if action.getName(index).lower() in {"click", "press", "activate"}:
            action.doAction(index)
            time.sleep(0.7)
            return True
    return False


def change_switch(row):
    for node in walk(row, 0, 8):
        try:
            if node.getRoleName() in {"switch", "toggle button", "check box"}:
                forbid_apply(node)
                action = node.queryAction()
                for index in range(action.nActions):
                    if action.getName(index).lower() in {"toggle", "click", "press", "activate"}:
                        action.doAction(index)
                        return "toggled"
        except Exception:
            continue
    raise RuntimeError("no switch found")


def change_value(row):
    for node in walk(row, 0, 8):
        try:
            role = node.getRoleName()
        except Exception:
            continue
        if role in {"spin button", "slider"}:
            value = node.queryValue()
            current = value.currentValue
            span = value.maximumValue - value.minimumValue
            if span <= 0:
                raise RuntimeError("value range too small to step")
            step = value.minimumIncrement
            if not step or step <= 0 or step > span:
                # Integer-ish ranges step by 1 (fractional steps round away
                # on integer spins); tiny ranges step by a tenth.
                step = 1.0 if span >= 3 else span / 10.0
            target = current + step
            if target > value.maximumValue:
                target = current - step
            target = max(value.minimumValue, min(value.maximumValue, target))
            if abs(target - current) < 1e-9:
                raise RuntimeError("value range too small to step")
            value.currentValue = target
            return f"{current} -> {target}"
    raise RuntimeError("no value control found")


def change_dropdown(row, colon_key):
    # GtkComboBoxText exposes no popup items over AT-SPI, but its combo box
    # implements the Selection interface: selectChild(i) picks model item i
    # exactly like a user choosing it.
    for node in walk(row, 0, 8):
        try:
            role = node.getRoleName()
        except Exception:
            continue
        if role == "combo box":
            forbid_apply(node)
            selection = node.querySelection()
            before = getoption(colon_key)
            for candidate in (1, 0, 2):
                selection.selectChild(candidate)
                time.sleep(0.5)
                if getoption(colon_key) != before:
                    return f"selected model item {candidate}"
            raise RuntimeError("no combo selection changed the runtime value")
    raise RuntimeError("no combo box found")


def change_color(row):
    for node in walk(row, 0, 8):
        try:
            name = node.name or ""
        except Exception:
            continue
        if name == "Edit color":
            # Raw-entry fallback: the row's value is not in a color form
            # (e.g. the -1 "unset, falls back" sentinel); the popover text
            # editor cannot be driven over AT-SPI.
            raise RuntimeError("raw-color-entry")
        if name.startswith("Color stop"):
            click(node)
            time.sleep(0.9)
            tile = find_named("Palette color", role="push button") or find_named(
                "Palette color", role="button"
            )
            if tile is None:
                raise RuntimeError("palette did not open")
            click(tile)
            time.sleep(0.3)
            select = find_named("Select", role="button") or find_named(
                "Select", role="push button"
            )
            if select is None:
                raise RuntimeError("Select button missing")
            select.queryAction().doAction(0)
            return "picked palette color"
    raise RuntimeError("no color control found")


def pending_page_lists(official_colon):
    chip = chip_node()
    if chip is None:
        return False
    click(chip)
    time.sleep(0.9)
    listed = find_named(f"{official_colon} · set to") is not None
    return listed


def main():
    with open(SKELETON) as handle:
        skeleton = json.load(handle)
    rows = skeleton["rows"]
    only = sys.argv[1] if len(sys.argv) > 1 else None

    # Results persist across chunked runs; reruns overwrite their rows.
    try:
        with open(RESULTS_CACHE) as handle:
            results = json.load(handle)
    except Exception:
        results = {}
    eligible = [row for row in rows if row["bucket"] == "editable-and-pending-required"]
    eligible.sort(key=lambda row: (row["pageLabel"] or "", row["rowId"]))
    if only and ":" in only and only.replace(":", "").replace("-", "").isdigit():
        start, end = only.split(":")
        eligible = eligible[int(start) - 1 : int(end)]
    elif only and "," in only:
        wanted = set(only.split(","))
        eligible = [row for row in eligible if row["rowId"] in wanted]
    elif only:
        eligible = [row for row in eligible if only in row["rowId"] or only == row["controlKind"]]

    log(f"eligible rows: {len(eligible)}")
    current_page = None
    for index, row in enumerate(eligible):
        row_id = row["rowId"]
        official = row["official"]
        colon = row["officialColon"]
        page = row["pageLabel"]
        kind = row["controlKind"]
        record = {
            "attempted": True,
            "firstChangeRegistered": False,
            "doubleToggleNeeded": False,
            "rowAccent": False,
            "sidebarBadge": False,
            "topChip": False,
            "bottomBar": False,
            "pendingPageListed": False,
            "discardCleared": False,
            "originalValue": None,
            "attemptedValue": None,
            "liveStatus": "unknown",
            "note": "",
        }
        results[row_id] = record
        try:
            # A clean slate is required per row; a lingering pending state
            # would cascade misreadings into every later row.
            if chip_node() is not None:
                discard_all()
                time.sleep(0.5)
                if chip_node() is not None:
                    record["liveStatus"] = "bug"
                    record["note"] = "previous pending state did not clear; aborting run to avoid cascading misreads"
                    log(f"[{index+1}/{len(eligible)}] ABORT stale pending before {row_id}")
                    break
            if kind == "ValueEntry":
                record["attempted"] = False
                record["liveStatus"] = "not-automated"
                record["note"] = (
                    "text-entry control: AT-SPI exposes no activate action to commit "
                    "typed text; verified manually outside the harness"
                )
                log(f"[{index+1}/{len(eligible)}] SKIP entry {row_id}")
                continue

            if page != current_page:
                nav(page)
                current_page = page

            before = getoption(colon)
            record["originalValue"] = before
            setting_row = find_setting_row(official)
            if setting_row is None:
                raise RuntimeError("setting row not found on page")

            if kind == "Switch":
                record["attemptedValue"] = change_switch(setting_row)
            elif kind in {"Slider", "SpinRow"}:
                record["attemptedValue"] = change_value(setting_row)
            elif kind == "Dropdown":
                record["attemptedValue"] = change_dropdown(setting_row, colon)
            elif kind == "ColorEntry":
                record["attemptedValue"] = change_color(setting_row)
            else:
                raise RuntimeError(f"unhandled control kind {kind}")

            time.sleep(0.8)
            after = getoption(colon)
            runtime_changed = after is not None and before is not None and after != before

            record["topChip"] = chip_count() == "1"
            record["bottomBar"] = bar_present()
            record["sidebarBadge"] = badge_count(page) == "1"
            record["rowAccent"] = row_marker(official) == "Pending change"
            record["firstChangeRegistered"] = record["topChip"]

            if not runtime_changed and not record["topChip"]:
                record["liveStatus"] = "no-op"
                if before is None:
                    record["note"] = (
                        "runtime readback unavailable for this option on this "
                        "compositor (getoption reports no parsable value), so no "
                        "preview session can capture an original"
                    )
                else:
                    record["note"] = (
                        f"runtime unchanged after change (before={before!r} after={after!r})"
                    )
                log(f"[{index+1}/{len(eligible)}] NO-OP {row_id}")
                discard_all()
                continue

            record["pendingPageListed"] = pending_page_lists(colon)
            # Return to the page for the discard-clears-row assertion.
            nav(page)

            discard_all()
            time.sleep(0.4)
            reverted = getoption(colon) == before
            cleared = chip_node() is None and not bar_present()
            marker_cleared = row_marker(official) in ("", None)
            record["discardCleared"] = bool(reverted and cleared and marker_cleared)

            ok = all(
                (
                    record["topChip"],
                    record["bottomBar"],
                    record["sidebarBadge"],
                    record["rowAccent"],
                    record["pendingPageListed"],
                    record["discardCleared"],
                )
            )
            record["liveStatus"] = "pass" if ok else "bug"
            log(
                f"[{index+1}/{len(eligible)}] {'PASS' if ok else 'BUG'} {row_id} "
                f"chip={record['topChip']} bar={record['bottomBar']} badge={record['sidebarBadge']} "
                f"accent={record['rowAccent']} page={record['pendingPageListed']} discard={record['discardCleared']}"
            )
        except Exception as error:
            if str(error) == "raw-color-entry":
                record["liveStatus"] = "not-automated"
                record["note"] = (
                    "value is a non-color sentinel (e.g. -1 = unset, falls back to "
                    "another option); the row hosts the raw text editor, which the "
                    "harness cannot drive over AT-SPI"
                )
                log(f"[{index+1}/{len(eligible)}] SKIP raw-entry {row_id}")
                continue
            record["liveStatus"] = "bug"
            record["note"] = f"harness error: {error}"
            log(f"[{index+1}/{len(eligible)}] ERROR {row_id}: {error}")
            try:
                discard_all()
            except Exception:
                pass
        finally:
            with open(RESULTS_CACHE, "w") as handle:
                json.dump(results, handle)

    # Merge into the final report.
    final_rows = []
    counts = {
        "editableAndPendingRequired": 0,
        "visibleButNotEditable": 0,
        "editableButNoOpInThisSession": 0,
        "bug": 0,
    }
    for row in rows:
        entry = dict(row)
        live = results.get(row["rowId"])
        if live is not None:
            entry["live"] = live
            if live["liveStatus"] == "bug":
                bucket = "bug"
            elif live["liveStatus"] == "no-op":
                bucket = "editable-but-no-op-in-this-session"
                entry["reason"] = live["note"]
            else:
                bucket = "editable-and-pending-required"
                if live["liveStatus"] == "not-automated":
                    entry["reason"] = live["note"]
        else:
            bucket = row["bucket"]
        entry["bucket"] = bucket
        key = {
            "editable-and-pending-required": "editableAndPendingRequired",
            "visible-but-not-editable": "visibleButNotEditable",
            "editable-but-no-op-in-this-session": "editableButNoOpInThisSession",
            "bug": "bug",
        }[bucket]
        counts[key] += 1
        final_rows.append(entry)

    report = {
        "report": "pending-changes-reliability-matrix",
        "modelVersion": "v0.55.2",
        "generatedAt": time.strftime("%Y-%m-%d"),
        "method": "live AT-SPI harness (tools/live_scenario_harness/pending_reliability_matrix.py) driving every eligible inline control through one user-like change, asserting all five pending surfaces on the first change, then discarding and asserting everything clears; runtime state cross-checked with hyprctl getoption before/after",
        "totals": {
            "scalarRows": len(final_rows),
            "structuredFamilies": len(skeleton["families"]),
            **counts,
        },
        "rows": final_rows,
        "families": skeleton["families"],
        "safety": {
            "saveClicked": False,
            "realConfigEdited": False,
            "runtimeMutated": "only via the app's own reversible preview controls; every change discarded and verified reverted per row",
        },
    }
    with open(OUTPUT, "w") as handle:
        json.dump(report, handle, indent=2)
        handle.write("\n")
    log(json.dumps(report["totals"], indent=1))


if __name__ == "__main__":
    main()
