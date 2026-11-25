#!/usr/bin/env python3
"""Block direct commits on master/main, as in axum-best.

Use --branch <name> to check a branch without switching the working tree.
"""

import argparse
import subprocess
import sys

BLOCKED_BRANCHES = ("master", "main")


def current_branch() -> str:
    """Return the current branch, including before the first commit."""
    try:
        result = subprocess.run(
            ["git", "symbolic-ref", "--short", "-q", "HEAD"],
            check=True,
            capture_output=True,
            text=True,
        )
        return result.stdout.strip()
    except subprocess.CalledProcessError:
        return "HEAD"


def main() -> int:
    parser = argparse.ArgumentParser(description="Block commits on protected branches")
    parser.add_argument(
        "--branch",
        default=None,
        help="Branch name to check (default: current git branch)",
    )
    args = parser.parse_args()
    branch = args.branch or current_branch()

    if branch in BLOCKED_BRANCHES:
        print(
            f"check_branch.py: commits directly to '{branch}' are not allowed.\n"
            "  Create a feature branch first, e.g.:\n"
            "    git checkout -b codex/your-change\n"
            "  and open a pull request to merge your changes.",
            file=sys.stderr,
        )
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
