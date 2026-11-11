"""One spike-shell process per verify session (primary + optional relaunch)."""

from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any, Callable

CTL = Path("/tmp/superreader-spike/verify-cmd.json")
ACK = Path("/tmp/superreader-spike/verify-ack.json")

LAUNCH_COUNT = 0


def launch_count() -> int:
    return LAUNCH_COUNT


def reset_launch_count() -> None:
    global LAUNCH_COUNT
    LAUNCH_COUNT = 0


class ShellSession:
    def __init__(
        self,
        out: Path,
        launch_fn: Callable[..., dict[str, Any]],
        finish_fn: Callable[[dict[str, Any]], None],
    ):
        self.out = out
        self._launch = launch_fn
        self._finish = finish_fn
        self._proc: dict[str, Any] | None = None

    def __enter__(self) -> ShellSession:
        return self

    def __exit__(self, *_exc: object) -> None:
        if self._proc:
            self._finish(self._proc)
            self._proc = None

    def ensure_running(self, env: dict[str, str] | None = None) -> dict[str, Any]:
        global LAUNCH_COUNT
        if self._proc and self._proc.get("alive"):
            return self._proc
        merged = {"SPIKE_VERIFY_SESSION": "1"}
        if env:
            merged.update(env)
        self._proc = self._launch(
            self.out,
            "verify-session",
            merged,
            lambda _names: True,
            120,
        )
        LAUNCH_COUNT += 1
        return self._proc

    def apply_preset(self, token: str, preset: dict[str, str], timeout: float = 30.0) -> None:
        self.ensure_running()
        CTL.parent.mkdir(parents=True, exist_ok=True)
        ACK.unlink(missing_ok=True)
        CTL.write_text(json.dumps({"token": token, "preset": preset}) + "\n")
        deadline = time.time() + timeout
        while time.time() < deadline:
            if ACK.exists():
                body = json.loads(ACK.read_text())
                if body.get("token") == token:
                    if not body.get("ok"):
                        raise RuntimeError(body.get("error") or "verify preset failed")
                    return
            time.sleep(0.15)
        raise TimeoutError(f"verify preset {token} ack")
