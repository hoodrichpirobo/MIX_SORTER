---
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion
description: Execute tasks from a track's implementation plan
argument-hint: [optional: track name or ID]
---

# Implement Track Protocol

You are implementing a development track collaboratively with the user. This protocol guides you through executing tasks from the track's plan.md following the project's workflow.

## Step 1: Verify Setup

```bash
ls orchestrator/product.md orchestrator/workflow.md orchestrator/tracks.md 2>/dev/null
```

If any file is missing:
> "Orchestrator is not set up. Please run `/orchestrator:setup` first."
> Halt execution.

## Step 2: Select Track

**If `$ARGUMENTS` is provided**: Search for matching track.

**Otherwise**: Find the next incomplete track automatically.

```bash
cat orchestrator/tracks.md
ls orchestrator/tracks/
```

Parse `tracks.md` to find tracks by status:
- `[ ]` = new/pending
- `[~]` = in progress
- `[x]` = completed

**Selection logic**:
1. If argument provided: match against track descriptions (case-insensitive)
2. If no argument: select first track that is NOT `[x]`
3. If no incomplete tracks: announce "All tracks completed!" and halt

Confirm selection with user:
> "Selected track: '<track_description>'. Proceed?"

## Step 3: Load Track Context

Read the track files:
- `orchestrator/tracks/<track_id>/spec.md`
- `orchestrator/tracks/<track_id>/plan.md`
- `orchestrator/workflow.md`

Understand what needs to be built and how.

## Step 4: Update Track Status

If track is `[ ]` (new), update to `[~]` (in progress) in `tracks.md`.

Also update `metadata.json`:
```json
{
  "status": "in_progress",
  "updated_at": "<current timestamp>"
}
```

## Step 5: Execute Tasks (Collaborative Loop)

Present the current plan status:
> "**Track: '<track_description>'**
>
> Current progress:
> - [x] Completed tasks...
> - [~] Current task (if any)
> - [ ] Pending tasks...
>
> Next task: '<next_task_description>'"

### For Each Task

**Step 5.1: Ask who implements**

Use AskUserQuestion:
"How do you want to handle this task?"
- **Claude implements** - I'll write the code following the spec
- **You implement** - I'll guide you and update the plan when done
- **Collaborate** - We work together, you tell me what to do
- **Skip for now** - Mark as skipped, move to next task

**Step 5.2: Execute based on choice**

**If Claude implements**:
1. Mark task as `[~]` in plan.md
2. Implement following workflow.md (TDD if specified)
3. Show the changes made (files modified, key code)
4. Ask: "Review the changes. Ready to commit, or want modifications?"
   - **Commit** → Proceed to Step 5.3
   - **Modify** → Make requested changes, ask again
   - **Discard** → Revert changes, ask how to proceed

**If User implements**:
1. Mark task as `[~]` in plan.md
2. Explain what needs to be done based on spec
3. Wait for user to implement
4. When user says done: review their changes, provide feedback if needed
5. Ask: "Ready to commit these changes?"

**If Collaborate**:
1. Mark task as `[~]` in plan.md
2. Work together - user gives instructions, Claude executes
3. Continue until user says the task is complete
4. Ask: "Ready to commit?"

**If Skip**:
1. Leave task as `[ ]`
2. Add note in plan.md: `<!-- Skipped: [reason if given] -->`
3. Move to next task

**Step 5.3: Commit (when ready)**

Only commit when user explicitly approves:

```bash
git add .
git status  # Show what will be committed
```

Ask: "These files will be committed. Confirm?"
- **Yes** → Commit with conventional message
- **No** → Let user adjust, ask again

```bash
git commit -m "<type>(<scope>): <description>"
```

**Step 5.4: Update plan.md**

Mark task complete with SHA:
```markdown
- [x] Task: Implement user login (SHA: abc1234)
```

**Step 5.5: Continue or pause**

Ask: "Continue to next task, or pause for now?"
- **Continue** → Go to next task
- **Pause** → Save progress, end implement session

### Phase Completion

When all tasks in a phase are complete:

1. Look for the "Orchestrator - User Manual Verification" task
2. Present verification checklist to user:
   > "Phase '<phase_name>' is complete. Please verify:
   > - [ ] All features work as specified
   > - [ ] Tests are passing
   > - [ ] No regressions introduced
   >
   > Confirm verification complete?"

3. After user confirms, mark verification task complete
4. Create checkpoint commit:
   ```bash
   git commit --allow-empty -m "orchestrator(checkpoint): Complete '<phase_name>'"
   ```

## Step 6: Track Completion

When all phases and tasks are complete:

1. Update track status in `tracks.md`: `[~]` → `[x]`

2. Update `metadata.json`:
   ```json
   {
     "status": "completed",
     "updated_at": "<current timestamp>"
   }
   ```

3. Announce completion:
   > "Track '<track_description>' is complete!"

## Step 7: Synchronize Documentation

After track completion, check if project docs need updates:

1. Read the completed track's `spec.md`
2. Compare against `product.md` and `tech-stack.md`
3. If significant changes detected, propose updates:
   > "Based on this track, I suggest updating product.md:
   > [proposed changes]
   > Approve these changes?"

Only update docs after user approval.

## Step 8: Track Cleanup

Ask user what to do with completed track:

Use AskUserQuestion:
"Track is complete. What would you like to do?"
- Archive (move to orchestrator/archive/)
- Delete permanently
- Keep in tracks (do nothing)

**If Archive**:
```bash
mkdir -p orchestrator/archive
mv orchestrator/tracks/<track_id> orchestrator/archive/
```
Remove entry from tracks.md.

**If Delete**:
Confirm with user (irreversible), then:
```bash
rm -rf orchestrator/tracks/<track_id>
```
Remove entry from tracks.md.

**If Keep**: Do nothing.

## Step 9: Next Steps

> "Implementation session complete!
>
> Next options:
> - Resume this track: `/orchestrator:implement <track_id>`
> - Create new track: `/orchestrator:newTrack`
> - Check status: `/orchestrator:status`"

---

## Notes

- **Collaborative by default**: Always ask before implementing or committing
- **User controls commits**: Never auto-commit without explicit approval
- **Pause anytime**: User can stop mid-track and resume later
- **Follow workflow.md**: TDD, coverage requirements still apply
- **Progress is saved**: plan.md tracks exactly where you left off
