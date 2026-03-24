# Specification: First Real Playlist Run

## Overview

Prepare MIX_SORTER for its first real execution against a live Spotify playlist and close the remaining gaps between local code quality and real-world usability.

## Goals

- Validate that the current CLI works against a real Spotify playlist input
- Ensure the local metadata flow is understandable and dependable during an actual run
- Resolve any runtime issues that block a safe commit, push, or normal personal use
- Leave the repository in a state that is easy to continue from in the next session

## Requirements

- Accept a real playlist ID, Spotify URL, or Spotify playlist URI
- Authenticate successfully with the configured Spotify application
- Fetch playlist tracks and attempt enrichment from `local_db.json`
- Report matches, misses, and invalid Camelot entries clearly
- Update the playlist ordering on Spotify without losing tracks
- Document any runtime issues discovered during the live execution

## Acceptance Criteria

- A real playlist run is attempted end to end
- Any discovered runtime defects are either fixed or captured as explicit follow-up work
- The repository has a clear path to commit and push after validation
- The next session can resume immediately from the track plan

## Out Of Scope

- Building a UI
- Migrating metadata storage to Supabase or another database
- Large architectural rewrites unrelated to getting the current CLI production-usable
