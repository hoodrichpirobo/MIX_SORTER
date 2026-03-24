---
allowed-tools: Bash, Read, Glob, Grep
description: Display current progress of the project
argument-hint:
---

# Status Overview Protocol

You are providing a status overview of the project's development progress.

## Step 1: Verify Setup

```bash
ls orchestrator/tracks.md orchestrator/workflow.md 2>/dev/null
```

If files are missing:
> "Orchestrator is not set up or tracks.md is missing. Run `/orchestrator:setup` first."
> Halt execution.

## Step 2: Read Project State

```bash
# Read tracks overview
cat orchestrator/tracks.md

# List all tracks
ls orchestrator/tracks/
```

For each track directory, read:
- `orchestrator/tracks/<track_id>/metadata.json`
- `orchestrator/tracks/<track_id>/plan.md`

## Step 3: Parse and Count

For each track's `plan.md`, count:
- Total phases (## headers)
- Total tasks (`- [ ]`, `- [~]`, `- [x]` patterns)
- Completed tasks (`[x]`)
- In-progress tasks (`[~]`)
- Pending tasks (`[ ]`)

Calculate overall progress:
- `completion_percentage = (completed / total) * 100`

## Step 4: Identify Current Work

Find:
- **Current phase**: The phase containing `[~]` tasks
- **Current task**: The first `[~]` task
- **Next task**: The first `[ ]` task after current
- **Blockers**: Any tasks marked with "BLOCKED" or similar

## Step 5: Generate Status Report

Present a formatted report:

```
═══════════════════════════════════════════════════════════════
                    PROJECT STATUS REPORT
═══════════════════════════════════════════════════════════════

📅 Date: <current date/time>
📊 Overall Progress: <completed>/<total> tasks (<percentage>%)

───────────────────────────────────────────────────────────────
                         TRACKS
───────────────────────────────────────────────────────────────

<For each track>:
📁 <track_description>
   Status: <new | in_progress | completed>
   Progress: <completed>/<total> tasks (<percentage>%)
   Current: <current task or "N/A">

───────────────────────────────────────────────────────────────
                      CURRENT FOCUS
───────────────────────────────────────────────────────────────

🎯 Active Track: <track_description or "None">
📍 Current Phase: <phase_name or "N/A">
⚡ Current Task: <task_description or "None in progress">
➡️  Next Task: <next_task_description or "None">

───────────────────────────────────────────────────────────────
                        BLOCKERS
───────────────────────────────────────────────────────────────

<List any blocked items, or "None">

═══════════════════════════════════════════════════════════════
```

## Step 6: Recommendations

Based on the status, provide recommendations:

**If no tracks exist**:
> "No tracks found. Create one with `/orchestrator:newTrack`"

**If all tracks completed**:
> "All tracks completed! Create a new track or celebrate 🎉"

**If there's an in-progress track**:
> "Continue with `/orchestrator:implement` to resume '<current_task>'"

**If there are pending tracks but none in progress**:
> "Start the next track with `/orchestrator:implement`"

---

## Notes

- This is a read-only command, no modifications
- Provides quick snapshot of project health
- Useful for context after breaks or for team visibility
