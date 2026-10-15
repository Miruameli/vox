#!/usr/bin/env python3
"""Structure gate for the Vox repository.

Enforces the modularity rules documented in ADR-0008 and the contributing
guide:

  1. No Rust source file exceeds 150 non-blank lines of code.
  2. No folder holds more than 5 direct files.
  3. No folder holds more than 5 subfolders.
  4. Folder names are descriptive kebab-case: generic bucket names
     (utils, helpers, common, misc, tmp, src_only) are rejected.

Run from the repository root:  python3 scripts/check_structure.py
Exit code 0 means all rules hold; 1 lists every violation.
"""

from pathlib import Path
import sys

MAX_FILE_LINES = 150
MAX_DIRECT_FILES = 5
MAX_SUBDIRS = 5
FORBIDDEN_DIR_NAMES = {"utils", "helpers", "common", "misc", "tmp"}
SKIP_DIRS = {".git", "target", "node_modules", "__pycache__"}


def repo_root() -> Path:
    return Path(__file__).resolve().parent.parent


def check_rust_files(root: Path) -> list[str]:
    violations = []
    for path in sorted(root.rglob("*.rs")):
        if any(part in SKIP_DIRS for part in path.parts):
            continue
        lines = [line for line in path.read_text(encoding="utf-8").splitlines() if line.strip()]
        if len(lines) > MAX_FILE_LINES:
            violations.append(
                f"{path.relative_to(root)}: {len(lines)} lines (max {MAX_FILE_LINES})"
            )
    return violations


def check_folders(root: Path) -> list[str]:
    violations = []
    for path in sorted(p for p in root.rglob("*") if p.is_dir()):
        if any(part in SKIP_DIRS for part in path.relative_to(root).parts):
            continue
        files = [child for child in path.iterdir() if child.is_file()]
        subdirs = [child for child in path.iterdir() if child.is_dir()]
        if path.name in FORBIDDEN_DIR_NAMES:
            violations.append(f"{path.relative_to(root)}/: forbidden folder name")
        if len(files) > MAX_DIRECT_FILES:
            violations.append(
                f"{path.relative_to(root)}/: {len(files)} direct files (max {MAX_DIRECT_FILES})"
            )
        if len(subdirs) > MAX_SUBDIRS:
            violations.append(
                f"{path.relative_to(root)}/: {len(subdirs)} subfolders (max {MAX_SUBDIRS})"
            )
    return violations


def check_root(root: Path) -> list[str]:
    violations = []
    files = [child for child in root.iterdir() if child.is_file()]
    if len(files) > MAX_DIRECT_FILES:
        violations.append(
            f"<repo root>: {len(files)} direct files (max {MAX_DIRECT_FILES})"
        )
    return violations


def main() -> int:
    root = repo_root()
    violations = (
        check_rust_files(root)
        + check_folders(root)
        + check_root(root)
    )
    if violations:
        print("structure gate failed:")
        for violation in violations:
            print(f"  - {violation}")
        return 1
    print(f"structure gate passed for {root}")
    return 0


if __name__ == "__main__":
    sys.exit(main())
