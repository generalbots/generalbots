#!/usr/bin/env python3
"""Runtime measurement primitives for the #2531 load-stress lab.

Stdlib-only. No binary, no measurement, no fabricated numbers: the environment
probe reports *why* a runtime cannot be measured and the caller emits an
explicit SKIP. Two RSS sources are supported:

* ``proc`` — sample ``/proc/<pid>/status`` (VmRSS/VmHWM) of a running server.
  Linux exposes the peak directly as VmHWM, so no polling loop is needed.
* ``process`` — run the workload command under ``/usr/bin/time -v`` when present,
  otherwise use ``os.wait4`` to read that child's ``ru_maxrss``.
"""

from __future__ import annotations

import os
import platform
import re
import shlex
import shutil
import signal
import subprocess
from dataclasses import dataclass
from pathlib import Path
from typing import Any

TIME_BINARY = "/usr/bin/time"
TIME_PEAK_RE = re.compile(r"Maximum resident set size \(kbytes\):\s*(\d+)")
PROC_STATUS = "/proc/{pid}/status"
DEFAULT_TIMEOUT_S = 300
KB_PER_MB = 1024.0


@dataclass(frozen=True)
class Environment:
    """What this host can actually measure."""

    time_binary: str | None
    workload_template: str | None
    server_pid: int | None
    proc_readable: bool
    python: str

    def available(self) -> bool:
        if not self.workload_template:
            return False
        if self.server_pid is not None and not self.proc_readable:
            return False
        return True

    def reason(self) -> str:
        if not self.workload_template:
            return (
                "no --workload-cmd supplied; runtime measurement needs a command "
                "that drives an import/export (pass --workload-cmd, or run --self-test)"
            )
        if self.server_pid is not None and not self.proc_readable:
            return f"/proc/{self.server_pid}/status is not readable on this host"
        return ""


def detect_environment(workload_template: str | None, server_pid: int | None) -> Environment:
    """Probe the host without side effects."""
    readable = False
    if server_pid is not None and server_pid > 0:
        try:
            readable = Path(PROC_STATUS.format(pid=server_pid)).is_file()
        except OSError:
            readable = False
    return Environment(
        time_binary=TIME_BINARY if shutil.which("time") and os.path.isfile(TIME_BINARY) else None,
        workload_template=workload_template or None,
        server_pid=server_pid if server_pid and server_pid > 0 else None,
        proc_readable=readable,
        python=platform.python_version(),
    )


def read_proc_rss(pid: int) -> dict[str, float]:
    """Return ``{'rss_mb', 'peak_mb'}`` from /proc, or an empty dict on error."""
    values: dict[str, float] = {}
    try:
        text = Path(PROC_STATUS.format(pid=pid)).read_text(encoding="utf-8")
    except (OSError, UnicodeDecodeError):
        return values
    for line in text.splitlines():
        if line.startswith("VmRSS:") or line.startswith("VmHWM:"):
            parts = line.split()
            if len(parts) >= 2:
                try:
                    values[line.split(":", 1)[0]] = int(parts[1]) / KB_PER_MB
                except ValueError:
                    continue
    return {
        "rss_mb": values.get("VmRSS", 0.0),
        "peak_mb": values.get("VmHWM", values.get("VmRSS", 0.0)),
    }


def parse_time_stderr(text: str) -> float | None:
    """Extract the peak RSS (MB) from ``/usr/bin/time -v`` stderr."""
    match = TIME_PEAK_RE.search(text)
    if not match:
        return None
    return int(match.group(1)) / KB_PER_MB


class WorkloadError(RuntimeError):
    """The workload could not be executed (bad command, missing binary, ...)."""


def _timeout_handler(_signum: int, _frame: Any) -> None:
    raise TimeoutError("workload timed out")


def _format_command(template: str, fields: dict[str, Any]) -> list[str]:
    class _Fallback(dict[str, str]):
        def __missing__(self, key: str) -> str:
            return ""

    try:
        rendered = template.format_map(_Fallback({k: str(v) for k, v in fields.items()}))
    except (ValueError, IndexError) as error:
        raise WorkloadError(f"invalid --workload-cmd template: {error}") from error
    try:
        command = shlex.split(rendered)
    except ValueError as error:
        raise WorkloadError(f"--workload-cmd is not a valid shell-free command: {error}") from error
    if not command:
        raise WorkloadError("--workload-cmd rendered to an empty command")
    return command


def _run_process(command: list[str], timeout: int) -> tuple[int, float, str]:
    """Run one command, returning (exit_code, peak_rss_mb, source).

    Output goes to DEVNULL: a workload may print document content, which must
    never be captured, and a full pipe would deadlock before ``wait4`` returns.
    """
    try:
        proc = subprocess.Popen(command, stdout=subprocess.DEVNULL, stderr=subprocess.DEVNULL)
    except OSError as error:
        raise WorkloadError(f"cannot execute {command[0]!r}: {error}") from error

    previous = signal.signal(signal.SIGALRM, _timeout_handler)
    if timeout > 0:
        signal.setitimer(signal.ITIMER_REAL, timeout)
    try:
        try:
            _, status, usage = os.wait4(proc.pid, 0)
        except TimeoutError as error:
            proc.kill()
            os.wait4(proc.pid, 0)
            raise WorkloadError(f"workload {command[0]!r} exceeded {timeout}s") from error
        except ChildProcessError as error:
            raise WorkloadError(f"lost child process for {command[0]!r}: {error}") from error
    finally:
        signal.setitimer(signal.ITIMER_REAL, 0)
        signal.signal(signal.SIGALRM, previous)
    code = os.waitstatus_to_exitcode(status)
    peak = usage.ru_maxrss / KB_PER_MB
    return code, peak, "os.wait4(ru_maxrss)"


def _run_timed(
    time_binary: str, command: list[str], timeout: int
) -> tuple[int, float | None, str]:
    """Run one command under ``/usr/bin/time -v``."""
    full = [time_binary, "-v", *command]
    try:
        completed = subprocess.run(
            full, stdout=subprocess.PIPE, stderr=subprocess.PIPE, timeout=timeout, check=False
        )
    except OSError as error:
        raise WorkloadError(f"cannot execute {time_binary!r}: {error}") from error
    except subprocess.TimeoutExpired as error:
        raise WorkloadError(f"timed workload exceeded {timeout}s: {error}") from error
    stderr = completed.stderr.decode("utf-8", errors="replace")
    return completed.returncode, parse_time_stderr(stderr), "/usr/bin/time -v"


def run_once(
    env: Environment,
    fields: dict[str, Any],
    timeout: int = DEFAULT_TIMEOUT_S,
    prefer_time: bool = True,
) -> dict[str, Any]:
    """Execute one workload and return its measurement (never document content)."""
    command = _format_command(env.workload_template or "", fields)
    before = read_proc_rss(env.server_pid) if env.server_pid else {}
    source = "proc:/proc/<pid>/status"
    exit_code = 0
    process_peak: float | None = None

    if env.server_pid:
        exit_code, _, _ = _run_process(command, timeout)
        after = read_proc_rss(env.server_pid)
        if not after:
            raise WorkloadError(f"server PID {env.server_pid} vanished during the workload")
        peak = after["peak_mb"]
        baseline = before.get("rss_mb", 0.0)
        returned = after["rss_mb"]
    else:
        if prefer_time and env.time_binary:
            exit_code, process_peak, source = _run_timed(env.time_binary, command, timeout)
        else:
            exit_code, process_peak, source = _run_process(command, timeout)
        if process_peak is None:
            raise WorkloadError(f"/usr/bin/time -v produced no peak RSS for {command[0]!r}")
        peak = process_peak
        baseline = process_peak
        returned = process_peak

    return {
        "exit_code": exit_code,
        "peak_rss_mb": round(peak, 2),
        "baseline_rss_mb": round(baseline, 2),
        "return_rss_mb": round(returned, 2),
        "rss_source": source,
    }


def run_soak(
    env: Environment,
    fields: dict[str, Any],
    cycles: int,
    timeout: int = DEFAULT_TIMEOUT_S,
    prefer_time: bool = True,
) -> dict[str, Any]:
    """Bounded open/edit/save/close soak; returns one aggregated measurement."""
    samples: list[dict[str, Any]] = []
    for index in range(cycles):
        iteration = dict(fields)
        iteration["mode"] = "soak"
        iteration["iteration"] = index + 1
        samples.append(run_once(env, iteration, timeout=timeout, prefer_time=prefer_time))

    first, last = samples[0], samples[-1]
    peak = max(sample["peak_rss_mb"] for sample in samples)
    growth = last["peak_rss_mb"] - first["peak_rss_mb"]
    return {
        "exit_code": max(int(sample["exit_code"]) for sample in samples),
        "peak_rss_mb": round(peak, 2),
        "baseline_rss_mb": first["baseline_rss_mb"],
        "return_rss_mb": last["return_rss_mb"],
        "session_cache_growth_mb": round(max(0.0, growth), 2),
        "rss_source": last["rss_source"],
        "cycles": cycles,
    }
