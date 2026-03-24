# Product Definition

## Product Name

MIX_SORTER

## Description

MIX_SORTER is a personal CLI tool for sorting Spotify playlists into a more DJ-friendly order. It enriches playlist tracks with locally stored BPM and Camelot key data, then reorders the playlist to support smoother harmonic mixing.

## Target Users

- Primary user: the project owner maintaining and curating personal Spotify playlists
- Secondary user profile: DJs or playlist curators who rely on key and tempo-aware ordering

## Core Features

- Authenticate with Spotify and load a target playlist by ID or URL
- Match Spotify tracks against `local_db.json` using normalization and fuzzy matching
- Enrich tracks with BPM and Camelot key metadata from local data
- Sort tracks by harmonic compatibility and tempo
- Push the updated track order back to Spotify

## Problem Statement

Spotify no longer exposes the audio metadata this workflow depends on in a reliable way. The tool needs a repeatable local-data-driven path to keep playlist sorting usable without depending on restricted APIs for key and BPM.

## Technical Constraints

- Must remain usable as a command-line workflow
- Must authenticate against Spotify with developer credentials
- Must tolerate imperfect track naming through fuzzy matching
- Must use `local_db.json` as the source of truth for enrichment metadata
- Should keep failure modes visible when tracks cannot be matched

## Success Metrics

- A playlist can be loaded, enriched, sorted, and written back in one run
- The majority of playlist tracks are matched correctly from the local database
- Missing or invalid metadata is surfaced clearly enough to fix the local database
- Sorting output is consistent and useful for harmonic mixing sessions
