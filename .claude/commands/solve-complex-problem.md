---
description: "Initiates a structured workflow to solve a complex problem or implement a difficult feature."
argument-hint: "[A detailed description of the problem to solve]"
---
This is a complex task that requires a structured approach. Follow these steps precisely. The workspace path will be provided to you by a hook.

**Phase 1: Understanding and Planning**

1. **Investigate:** Use the `investigator` agent to analyze the problem and the codebase. It will produce an `INVESTIGATION_REPORT.md` in the task's workspace.
    - Problem: $ARGUMENTS

2. **Map Flow:** Use the `code-flow-mapper` agent. It will read the investigation report and produce a `FLOW_REPORT.md` in the same workspace, detailing the execution path.

3. **Plan:** Use the `planner` agent. It will read both reports and produce a final, detailed `PLAN.md` in the workspace.

**Phase 2: Review and Execution**

4. **Present Plan:** Display the full contents of the `PLAN.md` to me for review and approval. **Wait for my confirmation before proceeding.**

5. **Execute:** Once I approve the plan, use the `backend-developer` and `qa-engineer` agents to execute the steps outlined in `PLAN.md`.
