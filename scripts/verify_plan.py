"""Build verify capture plans from feature-map and selected checks."""

from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
MAP = ROOT / ".cursor/skills/verify-superreader/feature-map.json"

WINDOW_PRESETS: dict[str, dict[str, str]] = {
    "pdf-highlight-1x": {"SPIKE_DRAG": "1"},
    "pdf-highlight-2x": {"SPIKE_DRAG": "1", "SPIKE_SCALE": "zoom-2x"},
    "epub-pride": {"SPIKE_MODE": "webkit", "SPIKE_SELECT": "1", "SPIKE_RELOAD": "1"},
    "epub-turned": {"SPIKE_MODE": "webkit", "SPIKE_TURN": "1"},
}

CHECK_TO_PRESET: dict[str, tuple[str, dict[str, str]]] = {
    "pdf-title": ("pdf-highlight-1x", {"SPIKE_DRAG": "1"}),
    "pdf-footer-nm": ("pdf-highlight-1x", {"SPIKE_DRAG": "1"}),
    "pdf-highlight-ink": ("pdf-highlight-1x", {"SPIKE_DRAG": "1"}),
    "pdf-highlight-ink-zoom": ("pdf-highlight-2x", {"SPIKE_DRAG": "1", "SPIKE_SCALE": "zoom-2x"}),
    "pdf-highlight-zoom-scale": ("pdf-highlight-2x", {"SPIKE_DRAG": "1", "SPIKE_SCALE": "zoom-2x"}),
    "epub-title": ("epub-pride", {"SPIKE_MODE": "webkit", "SPIKE_SELECT": "1", "SPIKE_RELOAD": "1"}),
    "epub-footer-nm": ("epub-pride", {"SPIKE_MODE": "webkit", "SPIKE_SELECT": "1", "SPIKE_RELOAD": "1"}),
    "epub-pages-left-in-chapter": ("epub-pride", {"SPIKE_MODE": "webkit", "SPIKE_SELECT": "1", "SPIKE_RELOAD": "1"}),
    "epub-highlight-on-sentence": ("epub-pride", {"SPIKE_MODE": "webkit", "SPIKE_SELECT": "1", "SPIKE_RELOAD": "1"}),
    "epub-turn": ("epub-turned", {"SPIKE_MODE": "webkit", "SPIKE_TURN": "1"}),
    "single-page": ("epub-pride", {"SPIKE_MODE": "webkit", "SPIKE_SELECT": "1", "SPIKE_RELOAD": "1"}),
}


def load_map() -> dict:
    return json.loads(MAP.read_text())


def relaunch_checks() -> set[str]:
    body = load_map()
    return set(body.get("relaunch") or [])


def dedupe_steps(check_ids: list[str]) -> list[tuple[str, dict[str, str]]]:
    seen: set[tuple[str, tuple[tuple[str, str], ...]]] = set()
    steps: list[tuple[str, dict[str, str]]] = []
    for check_id in check_ids:
        shot, preset = CHECK_TO_PRESET.get(check_id, (check_id, {}))
        key = (shot, tuple(sorted(preset.items())))
        if key in seen:
            continue
        seen.add(key)
        steps.append((shot, preset))
    return steps


def split_checks(check_ids: list[str]) -> tuple[list[str], list[str]]:
    relaunch = relaunch_checks()
    primary = [c for c in check_ids if c not in relaunch]
    second = [c for c in check_ids if c in relaunch]
    return primary, second
