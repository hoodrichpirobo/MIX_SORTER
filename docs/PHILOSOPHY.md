# Philosophy: Why This System Exists

This document explains the reasoning behind the design of this AI development configuration system.

## The Core Problem

AI assistants like Claude have no memory between sessions. Every time you start a new conversation, you're starting from zero. This creates several problems:

1. **Repeated explanations**: You explain the project, stack, and context every time
2. **Lost decisions**: Why did we choose X over Y? Nobody remembers
3. **Inconsistent work**: Without context, the AI might suggest things that contradict previous work
4. **Wasted time**: Re-establishing context eats into productive work time

## The Solution: Context Engineering

Instead of relying on the AI to "remember", we engineer the context:

- **Persist everything important** in files the AI can read
- **Structure information** so it's easy to load and understand
- **Automate capture** so you don't forget to document

## The Two Systems

Orchestrator provides two complementary systems:

### 1. Sessions (Continuity Between Conversations)

```
/session:start_session → Work → /session:end_session
```

Sessions solve the **memory problem**:
- Load all project context at start
- Auto-generate summary at end
- Detect and recover from crashes
- Maintain history in session-log.jsonl

### 2. Tracks (Structure for Complex Work)

```
/orchestrator:newTrack → /orchestrator:implement → Complete
```

Tracks solve the **planning problem**:
- Break features into spec (what) and plan (how)
- Execute tasks step by step with commits
- Pause at phase boundaries for verification
- Track progress with checkboxes and SHAs

### How They Work Together

```
┌─────────────────────────────────────────────────────────────────┐
│                      PROJECT LIFECYCLE                          │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  SETUP (once)                                                   │
│  ────────────                                                   │
│  /orchestrator:setup                                            │
│       │                                                         │
│       └──→ Creates: product.md, workflow.md, tech-stack.md,     │
│            product-guidelines.md, decisions.md, tracks.md       │
│                                                                 │
├─────────────────────────────────────────────────────────────────┤
│                                                                 │
│  DAILY WORK (repeated)                                          │
│  ─────────────────────                                          │
│                                                                 │
│  /session:start_session                                         │
│       │                                                         │
│       └──→ Loads: product.md, workflow.md, decisions.md,        │
│            tracks.md, session-log.jsonl, active track           │
│                     │                                           │
│                     ▼                                           │
│            ┌────────────────────┐                               │
│            │ New feature needed?│                               │
│            └─────────┬──────────┘                               │
│                      │                                          │
│        ┌─────────────┴─────────────┐                            │
│        │ YES                       │ NO                         │
│        ▼                           ▼                            │
│  /orchestrator:newTrack    /orchestrator:implement              │
│        │                           │                            │
│        └───────────┬───────────────┘                            │
│                    │                                            │
│                    ▼                                            │
│            ┌────────────────────┐                               │
│            │  Claude executes   │                               │
│            │  tasks from plan   │                               │
│            │  following workflow│                               │
│            └─────────┬──────────┘                               │
│                      │                                          │
│                      ▼                                          │
│  /session:end_session                                           │
│       │                                                         │
│       └──→ Generates summary, logs to session-log.jsonl         │
│                                                                 │
└─────────────────────────────────────────────────────────────────┘
```

## The File System

Each file serves a specific purpose. No duplication, no conflicts.

```
orchestrator/
├── product.md            ← What we're building (vision, users, features)
├── product-guidelines.md ← How it should feel (tone, UX, brand)
├── tech-stack.md         ← What technologies we use
├── workflow.md           ← How we work (TDD, coverage, commit style)
├── decisions.md          ← Why we made choices (ADR format)
├── tracks.md             ← Index of all tracks
├── tracks/               ← Individual tracks
│   └── feature_20251223/
│       ├── metadata.json ← Status, timestamps
│       ├── spec.md       ← What to build (contract)
│       └── plan.md       ← How to build (tasks)
└── code_styleguides/     ← Coding standards

.claude/sessions/
├── session-log.jsonl     ← Append-only history
└── .session-active.json  ← Current session marker (for crash recovery)
```

## Design Principles

### 1. The AI Reads, You Write (Minimally)

The system is designed so that:
- You execute simple commands (`/session:start_session`, `/session:end_session`)
- The AI does the heavy lifting (reading files, generating summaries)

You shouldn't need to manually write session logs or update documentation constantly.

### 2. Single Source of Truth

Each piece of information lives in one place:

| Information | Location |
|-------------|----------|
| What we're building | `product.md` |
| How we work | `workflow.md` |
| Why we made decisions | `decisions.md` |
| What was done | `session-log.jsonl` |
| Current task breakdown | `tracks/*/plan.md` |

No duplication, no conflicts.

### 3. Append-Only History

The session log (`session-log.jsonl`) is append-only:
- Entries are never modified after creation
- History is always preserved
- Easy to analyze patterns over time

### 4. Graceful Failure

Sessions can end abruptly (crashes, closed terminals). The system handles this:
- Active session marker (`.session-active.json`) detects orphans
- Recovery options let you salvage context
- No data is lost, just marked appropriately

### 5. Branch = Context Boundary

Git branches represent different contexts:
- `feature/auth` has different concerns than `hotfix/critical-bug`
- Mixing contexts in one session creates confusion
- One session = one branch keeps things clean

### 6. AI-Generated Summaries

Asking the user to summarize each session creates friction. Instead:
- The AI analyzes commits, files changed, and conversation
- Generates a summary automatically
- User just confirms or ends the session

This reduces the "ceremony" of closing sessions.

### 7. Full Context Loading

On session start, the AI loads *everything*:
- Product vision
- Technical decisions
- Recent session history
- Current track status

This might seem like overkill, but it ensures the AI never works with partial information. Modern AI can handle large contexts effectively.

### 8. Spec vs Plan Separation

Tracks separate **what** from **how**:

| spec.md | plan.md |
|---------|---------|
| The contract | The execution |
| Requirements, acceptance criteria | Phases, tasks, sub-tasks |
| Doesn't change during implementation | Updates as work progresses |
| Answers "what does done mean?" | Answers "what do I do now?" |

This separation prevents scope creep and keeps implementation focused.

### 9. Phase Verification

Large features are broken into phases. At each phase boundary:
- Claude pauses and asks user to verify
- User confirms feature works as specified
- Checkpoint commit is created

This catches issues early instead of at the end.

## Trade-offs

### More Files = More Overhead?

Yes, there are more files to manage. But:
- Most files rarely change (product.md, workflow.md)
- The session system is largely automated
- The payoff is consistent, context-aware AI assistance

### Slower Session Start?

Loading all context takes a moment. But:
- It happens once per session
- The time saved from not re-explaining things is much greater
- You get better quality assistance immediately

### Not Fully Automated?

We could theoretically auto-detect session start/end. But:
- Explicit commands give you control
- You decide when a "session" starts and ends
- Manual triggers are more reliable than heuristics

## What This System Is NOT

- **Not a project management tool**: It's for AI context, not task tracking (though tracks help)
- **Not a documentation system**: It documents *for the AI*, not for humans primarily
- **Not mandatory**: You can work without sessions, you just lose context benefits

## Evolution

This system is designed to evolve:
- Start simple (sessions only)
- Add tracks when doing bigger features
- Add decisions.md when patterns emerge
- Adjust workflow.md as your process matures

Don't try to use everything at once. Let the system grow with your needs.
