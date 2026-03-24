---
allowed-tools: Bash, Read, Write, Edit, Glob, Grep, AskUserQuestion
description: Initialize a new project with Orchestrator methodology
argument-hint: [optional: project description]
---

# Orchestrator Setup Protocol

You are initializing a project with the Orchestrator methodology. This wizard will guide you through creating the foundational documents for spec-driven development.

## Step 1: Check for Resume

Check if setup was previously started:

```bash
cat orchestrator/setup_state.json 2>/dev/null
```

**If file exists and has a `last_successful_step`**:
- Announce which step was completed
- Ask if user wants to resume or start fresh
- If resume: skip to the next incomplete step
- If start fresh: delete setup_state.json and continue

**If file doesn't exist or is empty**: This is a new setup, continue.

## Step 2: Detect Project Type

Determine if this is a new (greenfield) or existing (brownfield) project:

```bash
# Check for existing project indicators
ls -la .git package.json pom.xml requirements.txt go.mod Cargo.toml 2>/dev/null
git status --porcelain 2>/dev/null | head -5
ls src/ app/ lib/ 2>/dev/null | head -5
```

**Greenfield** (new project):
- No version control, no dependencies, no source code
- Or only a README.md exists

**Brownfield** (existing project):
- Has .git, dependency files, or source code
- If uncommitted changes exist, warn the user

For **brownfield projects**:
1. Analyze the codebase structure
2. Read README.md if exists
3. Infer tech stack from dependency files
4. Use this context for subsequent questions

## Step 3: Create Orchestrator Directory

```bash
mkdir -p orchestrator
echo '{"last_successful_step": ""}' > orchestrator/setup_state.json
```

## Step 4: Generate product.md

If `$ARGUMENTS` was provided, use it as the initial concept.

Otherwise, use AskUserQuestion to gather information:

**Questions to ask** (one at a time, max 5):
1. "What are you building? Describe the product in 1-2 sentences."
2. "Who are the target users?"
3. "What are the 3-5 core features?"
4. "What problem does this solve?"
5. "Any specific technical constraints?"

After gathering responses, generate `orchestrator/product.md` with:
- Product name and description
- Target users
- Core features
- Problem statement
- Technical constraints (if any)
- KPIs / Success metrics (inferred)

Present the draft to the user for approval. Revise if needed.

Once approved:
```bash
echo '{"last_successful_step": "product_guide"}' > orchestrator/setup_state.json
```

## Step 5: Generate product-guidelines.md

Use AskUserQuestion to gather brand/UX preferences:

**Questions** (max 3):
1. "What tone should the product have?" (Professional, Casual, Playful, etc.)
2. "Any visual or UX references/inspirations?"
3. "Key messaging or tagline?"

Generate `orchestrator/product-guidelines.md` with:
- Tone and voice guidelines
- Visual identity notes
- UX principles
- Accessibility considerations

Present draft for approval.

Once approved:
```bash
echo '{"last_successful_step": "product_guidelines"}' > orchestrator/setup_state.json
```

## Step 6: Generate tech-stack.md

For **brownfield**: Present the detected stack for confirmation.

For **greenfield**: Use AskUserQuestion:

**Questions** (max 4):
1. "Frontend framework?" (React, Vue, Angular, None, etc.)
2. "Backend language/framework?" (Node/Express, Python/FastAPI, Go, etc.)
3. "Database?" (PostgreSQL, MongoDB, SQLite, etc.)
4. "Any specific services?" (Auth, Cloud provider, etc.)

Generate `orchestrator/tech-stack.md` with:
- Frontend stack
- Backend stack
- Database
- Infrastructure/Services
- Development tools

Present draft for approval.

Once approved:
```bash
echo '{"last_successful_step": "tech_stack"}' > orchestrator/setup_state.json
```

## Step 7: Select Code Style Guides

List available templates:
```bash
ls orchestrator/templates/code_styleguides/
```

Based on the tech stack, recommend appropriate guides.

Use AskUserQuestion:
"Based on your stack, I recommend: [typescript.md, python.md]. Include these?"
- Yes, include recommended
- Let me choose different ones
- Skip style guides

Copy selected guides:
```bash
mkdir -p orchestrator/code_styleguides
cp orchestrator/templates/code_styleguides/[selected].md orchestrator/code_styleguides/
```

Update state:
```bash
echo '{"last_successful_step": "code_styleguides"}' > orchestrator/setup_state.json
```

## Step 8: Generate workflow.md

Use AskUserQuestion to configure the workflow:

**Questions**:
1. "Test coverage requirement?" (80% recommended, or custom)
2. "When to commit?" (After each task recommended, or after each phase)
3. "Use TDD methodology?" (Yes recommended, or No)

Generate `orchestrator/workflow.md` with:
- Development methodology (TDD if selected)
- Test coverage requirements
- Commit conventions
- Phase completion protocol
- Quality gates

Present draft for approval.

Once approved:
```bash
echo '{"last_successful_step": "workflow"}' > orchestrator/setup_state.json
```

## Step 9: Initialize Tracks and Decisions

Create `orchestrator/tracks.md`:

```markdown
# Project Tracks

This file tracks all major development tracks. Each track has its own spec and plan.

**Status Legend**:
- `[ ]` = New/Pending
- `[~]` = In Progress
- `[x]` = Completed

---

<!-- Tracks will be added here by /orchestrator:newTrack -->
```

Create `orchestrator/decisions.md`:

```markdown
# Architectural Decisions Record (ADR)

This file tracks important architectural and technical decisions made during development.

---

<!--
Template for new decisions:

## YYYY-MM-DD: Decision Title

**Context**: What situation or problem prompted this decision?

**Decision**: What was decided?

**Rationale**: Why was this the best choice?

**Alternatives Considered**:
- Alternative 1: Why not chosen

**Consequences**:
- Positive: Benefits
- Negative: Tradeoffs
-->
```

Create tracks directory:
```bash
mkdir -p orchestrator/tracks
```

## Step 10: Initialize Session System

Create the session directory and log file:
```bash
mkdir -p .claude/sessions
touch .claude/sessions/session-log.jsonl
```

## Step 11: Create First Track

Ask if user wants to create the first track now:
- Yes: Run the newTrack protocol (you can reference it or inline the logic)
- No: Finish setup, user will create tracks later

If creating first track:
1. Ask for track description
2. Generate spec.md with 3-5 questions
3. Generate plan.md based on spec and workflow
4. Create track folder with metadata.json, spec.md, plan.md
5. Update tracks.md

Update state:
```bash
echo '{"last_successful_step": "initial_track_generated"}' > orchestrator/setup_state.json
```

## Step 12: Finalize

1. Commit all orchestrator and session files:
```bash
git add orchestrator/ .claude/sessions/
git commit -m "orchestrator(setup): Initialize Orchestrator configuration"
```

2. Announce completion:
> "Orchestrator setup complete! Your project is now configured with:
> - Product definition (product.md)
> - Brand guidelines (product-guidelines.md)
> - Tech stack (tech-stack.md)
> - Code style guides
> - Development workflow (workflow.md)
> - Tracks file (tracks.md)
>
> Next steps:
> - Create a new track: `/orchestrator:newTrack`
> - Start implementing: `/orchestrator:implement`
> - Check status: `/orchestrator:status`"

---

## Notes

- This command works in the PROJECT directory, not the config template
- All questions use AskUserQuestion for better UX
- State is saved after each major step for resume capability
- For brownfield projects, infer as much as possible from existing code
