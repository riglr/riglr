#!/usr/bin/env python3
"""UserPromptSubmit hook for complex task directory preparation.
Automatically creates claude-instance-{id} directories for specific commands.
"""

import json
import os
import re
import sys
from pathlib import Path

# Commands that trigger this hook
TRIGGER_COMMANDS = ["/solve-complex-problem"]


def get_next_instance_id(base_dir: Path) -> int:
    """Find the next available instance ID."""
    if not base_dir.exists():
        return 1

    existing_dirs = [
        d
        for d in base_dir.iterdir()
        if d.is_dir() and d.name.startswith("claude-instance-")
    ]
    if not existing_dirs:
        return 1

    numbers = [
        int(re.search(r"(\d+)", d.name).group(1))
        for d in existing_dirs
        if re.search(r"(\d+)", d.name)
    ]
    return max(numbers) + 1 if numbers else 1


def main():
    """Main hook execution logic."""
    try:
        input_data = json.load(sys.stdin)
    except (json.JSONDecodeError, UnicodeDecodeError):
        sys.exit(0)  # Not a valid JSON input, exit silently

    prompt = input_data.get("prompt", "").strip()
    cwd = input_data.get("cwd", os.getcwd())

    if not any(prompt.startswith(cmd) for cmd in TRIGGER_COMMANDS):
        sys.exit(0)  # Not a trigger command, allow normal processing

    base_dir = Path(cwd) / "claude-code-storage"
    instance_id = get_next_instance_id(base_dir)
    instance_dir = base_dir / f"claude-instance-{instance_id}"

    try:
        base_dir.mkdir(exist_ok=True)
        instance_dir.mkdir(exist_ok=True)

        # Inject the workspace path into the prompt for the AI to use
        context_msg = f"A dedicated workspace has been created for this task at: `{instance_dir}`. All reports (INVESTIGATION_REPORT.md, FLOW_REPORT.md, PLAN.md) MUST be saved in this directory."
        print(context_msg)
        sys.exit(0)
    except Exception as e:
        print(f"Warning: Failed to create instance directory: {e}", file=sys.stderr)
        sys.exit(0)


if __name__ == "__main__":
    main()
