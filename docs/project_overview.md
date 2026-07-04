# scholar-ops — Project Overview

Version: v1.0, 2026-07-03

## Product Name & Mission

**scholar-ops** — an AI-powered scholarship evaluation and tracking system built on Claude Code (and compatible agent CLIs). Mission: shift a scholarship seeker's time from *digging through listings* to *applying to the few worth winning*.

## Problem

Finding scholarship listings is easy — aggregators, portals, and mailing lists overflow with them. Filtering is the time sink:

- Most listings fail on **hard eligibility walls** (nationality, age, degree level, GPA, language certificates) that are binary — reading 40 pages to discover 35 disqualifications wastes days.
- **Deadlines** govern everything: a perfect scholarship with 5 days left and a missing IELTS certificate is a wasted application.
- Scam and "ghost" scholarships (fee-to-apply, data harvesting) pollute the space.
- Tracking applications across cycles in spreadsheets decays fast.

## Users

- **Primary persona — the seeker:** a student or graduate (e.g., a West African graduate targeting funded Masters/PhD programs abroad) comfortable running a CLI, applying to scholarships for themselves in one or more academic cycles.
- **Secondary persona — the returning applicant:** the same user across cycles, maintaining a pipeline, learning from past outcomes.

## Solution Overview

A repository the user clones and opens with their AI coding CLI. The agent, guided by markdown "mode" instructions and the user's profile, does the following:

1. **Onboarding** — a chat interview builds `config/profile.yml`: identity, nationality, education, **target scholarship level (asked, never assumed)**, fields, countries, languages/certificates, finances, documents ready.
2. **Evaluate** — paste a scholarship URL (or text). The agent verifies liveness, extracts the listing, runs **hard eligibility gates first** (any fail → INELIGIBLE, stop), then weighted fit scoring (funding coverage, eligibility margin, field match, deadline feasibility, selectivity, effort, career value, constraints), a document-gap checklist, an application-angle brief, and a legitimacy check. Output: a report file + a tracker row. Verdicts: APPLY / MAYBE / SKIP / INELIGIBLE.
3. **Track** — `data/scholarships.md` is the single source of truth: deadline-sorted table with statuses `found → evaluated → preparing → applied → awaiting → interview → result`.
4. **Scan** — bounded discovery across configured portals (`portals.yml`: DAAD, Campus France, Erasmus Mundus catalogue, Chevening, Fulbright, Mastercard Foundation, aggregators), filtered by profile, feeding a pipeline inbox (`data/pipeline.md`) for later evaluation.
5. **Deterministic scripts** — zero-token Node utilities: `doctor.mjs` (setup validation), `deadline-check.mjs` (upcoming deadlines, warnings), `tracker-check.mjs` (dedup, normalization).

Philosophy: **this is a filter, not a spray-and-pray tool.** The system recommends against applying to anything below 4.0/5 and never submits an application itself — human in the loop, always.

## MVP Scope

- Skill router (`/scholar-ops`) with mode dispatch and discovery menu
- Onboarding interview → `config/profile.yml`
- Evaluation mode (liveness gate, blocks A–F, verdict, report, tracker entry)
- Tracker mode + compare mode (rank evaluated scholarships, deadline-aware)
- Pipeline inbox mode
- Portal scan mode + `portals.yml` configuration
- Diagnostics scripts: doctor, deadline-check, tracker-check

## Out of Scope (MVP)

- Batch parallel evaluation workers
- Motivation-letter / essay / PDF generation
- TUI dashboard
- Multi-CLI skill mirrors (Codex/OpenCode/etc.)
- Zero-token local portal parsers
- Multi-profile support (counselors managing several students)
- Publishing to GitHub / packaging as installer

## Tech Stack

- **Agent layer:** Claude Code skill (`.claude/skills/scholar-ops/SKILL.md`) + markdown mode instructions (`modes/*.md`)
- **Deterministic layer:** Node.js ≥ 18, plain `.mjs` scripts, zero runtime dependencies
- **Web access:** agent-native WebFetch/WebSearch; Playwright MCP fallback for JS-heavy pages
- **Data layer:** local files only — YAML config, markdown tracker/reports, TSV scan history
- **Deployment:** user's machine; no server, no API keys beyond the user's own CLI

## Success Criteria

- User evaluates a real scholarship URL end-to-end: liveness → gates → score → verdict → report + tracker row, in one command
- Ineligible scholarship short-circuits at gates without wasting a full evaluation
- Zero fabricated facts: every gate decision quotes the listing verbatim; unknowns marked UNKNOWN
- `deadline-check` surfaces every tracked deadline sorted, warning under 14 days
- A scan run produces only profile-relevant, deadline-valid, deduplicated entries in the pipeline inbox
- Personal data (profile, tracker, reports) never leaves the machine or enters version control
