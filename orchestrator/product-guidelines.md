# Product Guidelines

## Tone And Voice

- Practical and utilitarian
- Focused on reliability over marketing language
- Clear about what matched, what failed, and what the tool is doing

## Visual Identity Notes

- This is currently a CLI-first tool, so output should optimize for scanability
- Prefer concise terminal messages grouped by workflow stage: load, fetch, enrich, sort, update
- Use visual markers sparingly and only when they improve triage of match success and failure

## UX Principles

- Keep the main workflow short: authenticate, fetch, enrich, sort, apply
- Favor deterministic behavior over hidden heuristics
- Surface unmatched tracks immediately so the local database can be corrected
- Preserve trust by making sorting rules easy to understand
- Accept both playlist IDs and playlist URLs when possible

## Accessibility And Usability

- Terminal output should stay readable without color
- Error messages should be actionable and specific
- Logs should distinguish between authentication failures, input errors, matching gaps, and Spotify update failures
- Configuration should rely on standard environment variables and documented local files

## Messaging

- Core promise: sort Spotify playlists for smoother harmonic mixing using local metadata
- Supporting message: local control, transparent matching, and no dependence on deprecated audio feature endpoints
