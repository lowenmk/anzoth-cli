#!/usr/bin/env python3
"""End-to-end smoke test for the Windows Anzoth installer."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
import tempfile
from pathlib import Path


REPO_ROOT = Path(__file__).resolve().parents[2]
CODEx_RS = REPO_ROOT / "codex-rs"
PACKAGE_BUILDER = REPO_ROOT / "scripts" / "build_codex_package.py"


def parse_args() -> argparse.Namespace:
    parser = argparse.ArgumentParser()
    parser.add_argument("--installer", type=Path, required=True)
    parser.add_argument("--install-root", type=Path)
    parser.add_argument("--package-dir", type=Path)
    return parser.parse_args()


def run(cmd: list[str], *, env: dict[str, str] | None = None, cwd: Path | None = None) -> subprocess.CompletedProcess[str]:
    print("+", " ".join(cmd))
    return subprocess.run(
        cmd,
        cwd=str(cwd) if cwd is not None else None,
        env=env,
        check=True,
        text=True,
        capture_output=True,
    )


def build_package_dir(package_dir: Path) -> None:
    entrypoint = CODEx_RS / "target" / "debug" / "anzoth.exe"
    args = [
        sys.executable,
        str(PACKAGE_BUILDER),
        "--target",
        "x86_64-pc-windows-msvc",
        "--variant",
        "codex",
        "--entrypoint-bin",
        str(entrypoint),
        "--package-dir",
        str(package_dir),
        "--force",
    ]
    run(args, cwd=REPO_ROOT)


def user_path() -> str:
    completed = run(
        [
            "powershell",
            "-NoProfile",
            "-Command",
            "[Environment]::GetEnvironmentVariable('Path', 'User')",
        ],
        cwd=REPO_ROOT,
    )
    return completed.stdout.strip()


def main() -> int:
    args = parse_args()
    with tempfile.TemporaryDirectory(prefix="anzoth-installer-test-") as temp_root_str:
        temp_root = Path(temp_root_str)
        package_dir = args.package_dir or temp_root / "package"
        install_root = args.install_root or temp_root / "install"
        build_package_dir(package_dir)

        installer_env = os.environ.copy()
        installer_env["ANZOTH_PACKAGE_DIR"] = str(package_dir)
        installer_env["ANZOTH_INSTALL_DIR"] = str(install_root)

        install_result = run(
            [str(args.installer), "--release", "latest"],
            env=installer_env,
            cwd=REPO_ROOT,
        )
        print(install_result.stdout)
        if "Anzoth CLI installed successfully." not in install_result.stdout:
            raise AssertionError("Installer did not report success.")

        anzoth_exe = install_root / "bin" / "anzoth.exe"
        if not anzoth_exe.is_file():
            raise AssertionError(f"Missing installed entrypoint: {anzoth_exe}")

        version_result = run([str(anzoth_exe), "--version"], cwd=REPO_ROOT)
        if version_result.returncode != 0:
            raise AssertionError("Installed anzoth.exe failed --version.")

        user_path_value = user_path()
        if str(install_root / "bin") not in user_path_value:
            raise AssertionError("Installer did not add the bin directory to the user PATH.")

        uninstall_result = run(
            [str(args.installer), "--uninstall"],
            env=installer_env,
            cwd=REPO_ROOT,
        )
        print(uninstall_result.stdout)

        if install_root.exists():
            raise AssertionError("Installer uninstall did not remove the install root.")

        if str(install_root / "bin") in user_path():
            raise AssertionError("Installer uninstall did not remove the PATH entry.")

    return 0


if __name__ == "__main__":
    raise SystemExit(main())
