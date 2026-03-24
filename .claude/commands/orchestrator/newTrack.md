---
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion
description: Create a new development track (feature, bug fix, etc.)
argument-hint: [optional: track description]
---

# New Track Protocol

You are creating a new development track. A track is a focused unit of work (feature, bug fix, refactor, etc.) with its own specification and implementation plan.

## Step 1: Verify Setup

Check that Orchestrator is properly configured:

```bash
ls orchestrator/product.md orchestrator/workflow.md orchestrator/tech-stack.md 2>/dev/null
```

If any file is missing:
> "Orchestrator is not set up. Please run `/orchestrator:setup` first."
> Halt execution.

## Step 2: Get Track Description

**If `$ARGUMENTS` is provided**: Use it as the track description.

**Otherwise**: Ask the user:
> "What would you like to build or fix? Provide a brief description."

## Step 3: Determine Track Type

Analyze the description to infer the type:
- **feature**: New functionality
- **bug**: Fix for existing issue
- **refactor**: Code improvement without changing behavior
- **chore**: Maintenance task
- **docs**: Documentation update

Don't ask the user - infer from context.

## Step 4: Check for Duplicate Track

```bash
ls orchestrator/tracks/ 2>/dev/null
```

Extract short names from existing tracks. If the proposed track name would conflict, inform the user and suggest an alternative.

## Step 5: Load Project Context

Read these files to understand the project:
- `orchestrator/product.md`
- `orchestrator/tech-stack.md`
- `orchestrator/workflow.md`

This context will inform the spec and plan generation.

## Step 6: Generate Specification (spec.md)

Announce: "I'll ask a few questions to build the specification for this track."

Use AskUserQuestion to gather details. Tailor questions to track type:

**For FEATURES (3-5 questions)**:
- What specific functionality should this include?
- How should users interact with it?
- Are there any UI/UX requirements?
- What are the success criteria?
- Any edge cases to consider?

**For BUGS (2-3 questions)**:
- What is the expected behavior?
- What is the actual behavior?
- Steps to reproduce?

**For REFACTORS (2-3 questions)**:
- What code/area needs improvement?
- What's the goal of the refactor?
- Any constraints to maintain?

After gathering responses, generate the spec with sections:
- **Overview**: Brief description
- **Goals**: What this track achieves
- **Requirements**: Functional requirements
- **Acceptance Criteria**: How to verify completion
- **Out of Scope**: What this track does NOT include

Present the draft for approval. Revise if needed.

## Step 7: Generate Implementation Plan (plan.md)

Read `orchestrator/workflow.md` to understand the methodology.

Generate a hierarchical plan with:
- **Phases**: Major milestones
- **Tasks**: Individual work items
- **Sub-tasks**: Granular steps (if TDD: "Write tests" then "Implement")

Format:
```markdown
# Implementation Plan: [Track Description]

## Phase 1: [Phase Name]

- [ ] Task: [Task description]
  - [ ] Sub-task: Write tests for [component]
  - [ ] Sub-task: Implement [component]
- [ ] Task: [Next task]

- [ ] Task: Orchestrator - User Manual Verification 'Phase 1' (Protocol in workflow.md)

## Phase 2: [Phase Name]
...
```

**Important**:
- Follow TDD if specified in workflow (tests before implementation)
- Add verification task at end of each phase
- Use `[ ]` for pending, `[~]` for in-progress, `[x]` for complete

Present the draft for approval. Revise if needed.

## Step 8: Create Track Artifacts

Generate track ID:
```bash
date +"%Y%m%d"
```

Track ID format: `shortname_YYYYMMDD` (e.g., `user_auth_20251223`)

Create the track structure:
```bash
mkdir -p orchestrator/tracks/<track_id>
```

Create `metadata.json`:
```json
{
  "track_id": "<track_id>",
  "type": "feature",
  "status": "new",
  "created_at": "<ISO timestamp>",
  "updated_at": "<ISO timestamp>",
  "description": "<User's description>"
}
```

Write the approved `spec.md` and `plan.md` to the track folder.

## Step 9: Update Tracks File

Append to `orchestrator/tracks.md`:

```markdown

---

## [ ] Track: [Track Description]
*Link: [./orchestrator/tracks/<track_id>/](./orchestrator/tracks/<track_id>/)*
```

## Step 10: Announce Completion

> "Track '<track_id>' has been created!
>
> - Spec: orchestrator/tracks/<track_id>/spec.md
> - Plan: orchestrator/tracks/<track_id>/plan.md
>
> Start implementing with `/orchestrator:implement` or `/orchestrator:implement <track_id>`"

---

## Notes

- Maximum 5 questions for spec generation to avoid fatigue
- Plans should be detailed but not overwhelming (3-5 phases typical)
- Track IDs are unique by date - only one track with same shortname per day
- All questions use AskUserQuestion for better UX
