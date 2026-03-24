# Daily Usage Guide

A practical guide for using this system day-to-day.

## Complete Workflow Overview

This diagram shows the full lifecycle of using Orchestrator with Claude Code:

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         FIRST TIME SETUP                                │
│                                                                         │
│   1. Use GitHub Template → Clone your new repo                          │
│   2. /orchestrator:setup → Answer questions → Generates all config      │
│                                                                         │
│   Creates: product.md, product-guidelines.md, tech-stack.md,            │
│            workflow.md, decisions.md, tracks.md, code_styleguides/      │
└─────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
┌─────────────────────────────────────────────────────────────────────────┐
│                         DAILY WORKFLOW                                  │
│                                                                         │
│   ┌──────────────────┐                                                  │
│   │ START OF DAY     │                                                  │
│   │                  │                                                  │
│   │ /session:start   │──→ Loads full context, shows last session        │
│   │ _session         │    summary, suggests next steps                  │
│   └────────┬─────────┘                                                  │
│            │                                                            │
│            ▼                                                            │
│   ┌──────────────────┐     ┌──────────────────┐                         │
│   │ NEED NEW FEATURE │     │ CONTINUE TRACK   │                         │
│   │                  │     │                  │                         │
│   │ /orchestrator:   │     │ /orchestrator:   │                         │
│   │ newTrack         │     │ implement        │                         │
│   │                  │     │                  │                         │
│   │ Creates spec.md  │     │ Executes tasks   │                         │
│   │ Creates plan.md  │     │ from plan.md     │                         │
│   └────────┬─────────┘     └────────┬─────────┘                         │
│            │                        │                                   │
│            └────────────┬───────────┘                                   │
│                         │                                               │
│                         ▼                                               │
│   ┌──────────────────────────────────────────────┐                      │
│   │              WORKING (collaborative)         │                      │
│   │                                              │                      │
│   │  For each task:                              │                      │
│   │  → Who implements? (Claude/You/Together)     │                      │
│   │  → Review changes before commit              │                      │
│   │  → Commit only when you approve              │                      │
│   │  → Continue or pause anytime                 │                      │
│   │                                              │                      │
│   │  /orchestrator:status → Check progress       │                      │
│   │  /orchestrator:revert → Undo if needed       │                      │
│   └──────────────────────┬───────────────────────┘                      │
│                          │                                              │
│                          ▼                                              │
│   ┌──────────────────┐                                                  │
│   │ END OF DAY       │                                                  │
│   │                  │                                                  │
│   │ /session:end     │──→ Auto-generates summary, logs to               │
│   │ _session         │    session-log.jsonl, suggests next steps        │
│   └──────────────────┘                                                  │
└─────────────────────────────────────────────────────────────────────────┘
```

## All Commands Reference

| Command | When to Use | What It Does |
|---------|-------------|--------------|
| `/orchestrator:setup` | Once per project | Wizard that creates all config files |
| `/orchestrator:newTrack` | Starting new feature/bug | Creates spec.md + plan.md for a unit of work |
| `/orchestrator:implement` | Ready to code | Executes tasks from plan.md step by step |
| `/orchestrator:status` | Anytime | Shows project and track progress |
| `/orchestrator:revert` | Something went wrong | Reverts commits/tracks using git |
| `/session:start_session` | Start of work session | Loads context, detects orphans |
| `/session:end_session` | End of work session | Generates summary, logs history |

## Starting Your Day

### 1. Open your project and start a session

```bash
cd your-project
claude  # or however you launch Claude Code

# First thing: start a session
/session:start_session
```

### 2. Review the context

Claude will show you:
- Current branch and git status
- Active track (if any)
- What was done in the last session
- What was planned for this session

### 3. Confirm your focus

Claude will ask what you want to work on. Either:
- Confirm the suggested next steps from last session
- Set a new goal

## Working with Tracks

Tracks are the core unit of work in Orchestrator. They provide structure for features that span multiple sessions.

### The Track Lifecycle

```
/orchestrator:newTrack "Add user authentication"
        │
        ▼
┌───────────────────────────┐
│ Claude asks 3-5 questions │
│ about the feature         │
└─────────────┬─────────────┘
              │
              ▼
┌───────────────────────────┐
│ Generates spec.md         │◄── WHAT to build (the contract)
│ - Overview                │
│ - Goals                   │
│ - Requirements            │
│ - Acceptance Criteria     │
└─────────────┬─────────────┘
              │
              ▼
┌───────────────────────────┐
│ Generates plan.md         │◄── HOW to build it (the tasks)
│ - Phase 1: [Tasks]        │
│ - Phase 2: [Tasks]        │
│ - Phase 3: [Tasks]        │
└─────────────┬─────────────┘
              │
              ▼
/orchestrator:implement
              │
              ▼
┌───────────────────────────────────────────┐
│ FOR EACH TASK (collaborative):            │
│                                           │
│ 1. "Who implements?"                      │
│    → Claude / You / Collaborate / Skip    │
│                                           │
│ 2. Implementation happens                 │
│                                           │
│ 3. "Review changes. Ready to commit?"     │
│    → Commit / Modify / Discard            │
│                                           │
│ 4. "Continue or pause?"                   │
│    → Next task / Save & exit              │
│                                           │
│ At phase end:                             │
│ → User verification                       │
│ → Checkpoint commit                       │
└─────────────┬─────────────────────────────┘
              │
              ▼
┌───────────────────────────┐
│ Track complete!           │
│ → Archive / Delete / Keep │
└───────────────────────────┘
```

### When to create a track

- Feature will take multiple sessions
- You need a spec and task breakdown
- You want to track progress formally

### When NOT to use tracks

- Quick bug fix
- Small change
- Exploration/research

### Track structure

```
orchestrator/tracks/user_auth_20251223/
├── metadata.json   # Status, dates
├── spec.md         # What we're building (the contract)
└── plan.md         # Task breakdown (the execution)
```

## Working

Just work normally. The session system doesn't interfere with your coding.

Some tips:
- **Commit frequently** - This helps the AI track progress
- **Mention decisions** - If you decide something important, say it out loud so it's captured
- **Stay on one branch** - If you need to switch, end the session first

## Switching Branches

If you need to switch branches (hotfix, different feature):

```bash
/session:end_session  # Close current session

# Switch branch
git checkout hotfix/critical-bug

# Start new session on new branch
/session:start_session
```

This keeps contexts separate.

## Ending Your Day

### 1. End the session

```bash
/session:end_session
```

### 2. Review the summary

Claude will show you:
- What was accomplished (auto-generated)
- Files modified
- Next steps

### 3. Handle uncommitted changes

If you have uncommitted changes, Claude will ask what to do:
- Commit them
- Note them for next session
- Leave them (they'll still be there)

## Handling Crashes

If your session ends abruptly (terminal crash, power outage, etc.):

### Next time you start:

```bash
/session:start_session
```

Claude will detect the orphan session and ask:
- **Recover**: Try to figure out what happened and log it
- **Discard**: Mark as abandoned, start fresh
- **Continue**: Resume the interrupted session

Usually "Recover" is the best choice - it preserves history.

## Recording Decisions

When you make an important architectural decision:

1. Tell Claude about it during the session
2. Ask Claude to add it to `decisions.md`

Or manually add to `orchestrator/decisions.md`:

```markdown
## 2024-12-22: Use Zustand for State Management

**Context**: Need state management for the app

**Decision**: Zustand

**Rationale**: Simpler than Redux, sufficient for our scale

**Alternatives Considered**:
- Redux: Too much boilerplate
- Context: Gets messy at scale
```

## Common Scenarios

### "I forgot to start a session"

No problem. Start one now:
```bash
/session:start_session
```

You'll get context loaded. Any work done before is in git history.

### "I forgot to end a session yesterday"

When you start today:
```bash
/session:start_session
```

Claude will detect the orphan and let you recover it.

### "I need to work on a quick hotfix"

```bash
/session:end_session  # If you have an active session
git checkout main
git checkout -b hotfix/thing
/session:start_session "Quick hotfix for X"
# ... fix it ...
/session:end_session
git checkout previous-branch
/session:start_session  # Back to your feature
```

### "I want to see what I did last week"

Check the session log:
```bash
cat .claude/sessions/session-log.jsonl | jq .
```

Or ask Claude:
> "Show me a summary of sessions from last week"

### "I'm starting a brand new project"

1. Use this template from GitHub ("Use this template" button)
2. Run `/orchestrator:setup` to configure the project
3. Run `/session:start_session` to begin your first session
4. Create tracks with `/orchestrator:newTrack` and implement with `/orchestrator:implement`


## Tips for Best Results

1. **Be consistent** - Always start/end sessions
2. **Commit often** - Helps tracking
3. **Speak decisions out loud** - "Let's use X because Y" gets captured
4. **One branch per session** - Keeps context clean
5. **Review next_steps** - They help future you

## Troubleshooting

### "Claude doesn't remember the project"

Did you run `/session:start_session`? It loads context.

### "The session log is getting huge"

JSON Lines files can be filtered. You could archive old entries:
```bash
# Keep last 100 sessions
tail -100 session-log.jsonl > session-log-new.jsonl
mv session-log-new.jsonl session-log.jsonl
```

### "I have multiple active sessions somehow"

This shouldn't happen, but if it does:
```bash
rm .claude/sessions/.session-active.json
/session:start_session
```

### "Claude is suggesting things that contradict our decisions"

Make sure decisions are in `decisions.md`. If they are, point Claude to it:
> "Check decisions.md - we decided to use X"
