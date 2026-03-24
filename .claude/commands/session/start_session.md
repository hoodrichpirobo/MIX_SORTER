---
allowed-tools: Bash, Read, Write, Glob, Grep, Edit
description: Start a new development session with full context capture and Orchestrator integration (project)
argument-hint: [optional: session goal]
---

# Start Session Protocol

You are starting a new development session. This protocol ensures context continuity across sessions.

## Step 0: Verify Orchestrator Setup

Check if Orchestrator is configured:

```bash
ls orchestrator/product.md orchestrator/workflow.md 2>/dev/null
```

**If files don't exist**:
> "Orchestrator is not configured in this project. Would you like to:
> 1. Run `/orchestrator:setup` to initialize
> 2. Continue without Orchestrator context (session-only mode)"

If user chooses option 1: halt and let them run setup.
If user chooses option 2: skip Step 3 (Load Project Context) and continue with session-only mode.

Ensure session directory exists:
```bash
mkdir -p .claude/sessions
```

## Step 1: Check for Orphan Session

First, check if there's an unclosed session from a previous crash or interruption:

```bash
# Check if .session-active.json exists
cat .claude/sessions/.session-active.json 2>/dev/null
```

**If `.session-active.json` exists**, a previous session wasn't closed properly. Ask the user:

> "Found an orphan session from `<started_at>` on branch `<branch>`. What would you like to do?"
> 1. **Recover**: Attempt to reconstruct what happened and close it properly
> 2. **Discard**: Mark it as interrupted and start fresh
> 3. **Continue**: Resume that session instead of starting a new one

- **If Recover**: Run git log since that timestamp, gather modified files, generate summary, append to log as "interrupted", then continue with new session
- **If Discard**: Append minimal entry to log with status "abandoned", delete `.session-active.json`, continue
- **If Continue**: Skip to Step 6 (Report) using existing session data

## Step 2: Capture Git State

```bash
# Get current branch
git branch --show-current

# Get current HEAD SHA
git rev-parse --short HEAD

# Get git status
git status --porcelain

# Get recent commits (last 5)
git log --oneline -5
```

## Step 3: Load Full Project Context

Read these files to understand the project completely:

1. **Product Vision**: `orchestrator/product.md`
2. **UX Guidelines**: `orchestrator/product-guidelines.md`
3. **Workflow/Methodology**: `orchestrator/workflow.md`
4. **Decisions Log**: `orchestrator/decisions.md` (if exists)
5. **Tracks Overview**: `orchestrator/tracks.md`

For active tracks:
```bash
ls orchestrator/tracks/
```

For each track, read:
- `orchestrator/tracks/<track>/metadata.json` - check `status` field
- `orchestrator/tracks/<track>/spec.md` - understand what we're building
- `orchestrator/tracks/<track>/plan.md` - see pending `[ ]` and in-progress `[~]` tasks

## Step 4: Load Session History

Read the session log to understand recent work:

```bash
# Read last 10 sessions from log
tail -10 .claude/sessions/session-log.jsonl 2>/dev/null
```

Parse each JSON line and extract:
- What was accomplished recently
- Patterns in the work
- Any pending `next_steps` from the last session

Pay special attention to the **most recent session's `next_steps`** - this is what the user planned to do.

## Step 5: Create Active Session Marker

Generate session ID and create the active session file:

```bash
date +"%Y%m%d_%H%M%S"
```

Create `.claude/sessions/.session-active.json`:

```json
{
  "id": "session_<YYYYMMDD_HHMMSS>",
  "started_at": "<ISO 8601 timestamp>",
  "git": {
    "branch": "<current branch>",
    "initial_commit": "<HEAD SHA>"
  },
  "project": "<from orchestrator/product.md title or directory name>",
  "track": "<active track id or null>",
  "goal": "<from $ARGUMENTS or null>"
}
```

## Step 6: Report Session Context

Provide a comprehensive summary of what you now know:

```
## Session Started: session_<ID>

**Time**: <timestamp>
**Branch**: <branch>
**HEAD**: <commit SHA>
**Project**: <project name>

### Git Status
- Staged: <count> files
- Unstaged: <count> files
- Untracked: <count> files

### Active Track
**<track_id>**: <brief description from spec>
- Pending tasks: <count>
- In progress: <count>
- Current phase: <from plan.md>

### Context from Last Session
**Accomplished**: <summary>
**Next steps planned**: <from previous session's next_steps>

### Key Project Context
- **Stack**: <from product.md>
- **Current focus**: <inferred from track and recent sessions>
- **Recent decisions**: <from decisions.md if exists>
```

## Step 7: Confirm Start

If `$ARGUMENTS` was provided:
> "Session goal: **<goal>**. I've loaded full context. Ready to begin?"

Otherwise:
> "I've loaded the full project context. What would you like to focus on this session?"

---

## Important Notes

- **One session = one branch**. If you need to switch branches, close this session first with `/session:end_session`
- The active session marker (`.session-active.json`) ensures we can recover from crashes
- All context is loaded upfront so I can work with full awareness of the project
