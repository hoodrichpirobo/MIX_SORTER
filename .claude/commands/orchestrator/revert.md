---
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion
description: Revert completed work using Git
argument-hint: [optional: track/phase/task to revert]
---

# Revert Protocol

You are reverting previously completed work. This protocol helps undo tracks, phases, or individual tasks using Git.

## Step 1: Verify Setup

```bash
ls orchestrator/tracks.md 2>/dev/null
git status 2>/dev/null
```

If tracks.md is missing or not in a git repo:
> "Orchestrator is not set up or this is not a Git repository."
> Halt execution.

## Step 2: Identify Revert Target

**If `$ARGUMENTS` is provided**: Parse to find the target.

**Otherwise**: Show interactive menu.

### Interactive Selection

Read all tracks and plans:
```bash
cat orchestrator/tracks.md
ls orchestrator/tracks/
```

Find revert candidates:
1. First, look for `[~]` (in-progress) items
2. If none, show recent `[x]` (completed) items

Present a hierarchical menu:

Use AskUserQuestion:
"What would you like to revert?"
- [List of in-progress or recently completed items]
- Specify a different track/phase/task

Options format:
- "Task: <task_description> (track: <track_id>)"
- "Phase: <phase_name> (track: <track_id>)"
- "Track: <track_description>"

## Step 3: Confirm Target

Once target is identified, confirm with user:
> "You want to revert: <target_description>
> This will undo commits and reset plan status.
> Proceed?"

If not confirmed, ask for different target or halt.

## Step 4: Find Associated Commits

Based on the target type:

### For a Task:
1. Read the plan.md
2. Find the task's commit SHA (if recorded)
3. If no SHA, search git log:
   ```bash
   git log --oneline --grep="<task keywords>" | head -5
   ```

### For a Phase:
1. Find all tasks in the phase
2. Collect all their SHAs
3. Also find the phase checkpoint commit

### For a Track:
1. Find all phases and tasks
2. Collect all SHAs
3. Find the track creation commit in tracks.md history

Also find plan-update commits:
```bash
git log --oneline -- orchestrator/tracks/<track_id>/plan.md | head -10
```

## Step 5: Handle Missing Commits

If a SHA from plan.md is not found in git history:
- The commit may have been rebased or amended
- Search for similar commit messages
- Ask user to confirm the replacement commit

```bash
git log --oneline --all | grep -i "<task keywords>"
```

## Step 6: Present Execution Plan

Show the user exactly what will happen:

```
═══════════════════════════════════════════════════════════════
                      REVERT PLAN
═══════════════════════════════════════════════════════════════

Target: <target_description>
Type: <Task | Phase | Track>

Commits to revert (in order):
1. <sha> - <commit message>
2. <sha> - <commit message>
...

Actions:
1. Run `git revert --no-edit <sha>` for each commit
2. Update plan.md to mark items as pending [ ]
3. Update tracks.md if reverting entire track

⚠️  This will create new revert commits, not delete history.

═══════════════════════════════════════════════════════════════
```

Use AskUserQuestion:
"Execute this revert plan?"
- Yes, proceed
- No, cancel

## Step 7: Execute Revert

If confirmed, execute in reverse order (newest first):

```bash
git revert --no-edit <sha1>
git revert --no-edit <sha2>
# ... etc
```

### Handle Conflicts

If any revert fails with conflicts:
> "Revert of <sha> caused conflicts in:
> <list of conflicted files>
>
> Please resolve conflicts manually, then run:
> `git add . && git revert --continue`
>
> After resolving, I'll continue updating the plan."

Halt and wait for user to resolve.

## Step 8: Update Plan Status

After successful reverts, update the plan.md:

For reverted tasks: `[x]` → `[ ]`
Remove SHA annotations.

For reverted phases: Reset all tasks in the phase.

For reverted tracks: Also update tracks.md: `[x]` → `[ ]` or remove entry.

Commit the plan update:
```bash
git add orchestrator/
git commit -m "orchestrator(revert): Reset <target_description> to pending"
```

## Step 9: Announce Completion

> "Revert complete!
>
> Reverted: <target_description>
> Commits reverted: <count>
> Plan status: Updated to pending
>
> You can now re-implement with `/orchestrator:implement`"

---

## Notes

- Reverts create new commits, preserving history
- Always revert in reverse chronological order
- Conflicts may occur if code has changed significantly
- Plan.md is updated to reflect the revert
- This is a complex operation - proceed carefully
