---
name: scholar-ops
description: AI scholarship command center — evaluate scholarship links against your profile with hard eligibility gates and weighted scoring, track applications by deadline, scan portals for new matches. Use when the user pastes a scholarship URL, asks to evaluate/compare/track scholarships or fellowships or grants, or runs /scholar-ops.
arguments: mode
user_invocable: true
user-invocable: true
argument-hint: "[<url> | onboarding | evaluate | compare | pipeline | scan | tracker]"
license: MIT
---

# scholar-ops — Router

scholar-ops filters scholarship listings against the Seeker's profile so they spend time applying to winners, not digging. This file routes an invocation to exactly one mode. Global behavioral boundaries are in `CLAUDE.md`; shared rules (gates, weights, verdicts, budgets, file contracts) are in `modes/_shared.md`.

## Pre-dispatch guard (INV-01, FR-014)

Before dispatching to `evaluate`, `compare`, `pipeline`, or `scan`: check that `config/profile.yml` exists and parses. If it does not, dispatch **`onboarding`** instead and tell the Seeker why. (`onboarding` and `tracker` do not require a profile.)

## Mode routing

Determine the mode from the argument `$mode`:

| Input | Mode file |
|-------|-----------|
| *(empty / unrecognized)* | Discovery menu (below) |
| `onboarding` | `modes/onboarding.md` |
| `evaluate <url\|text>` | `modes/evaluate.md` |
| a URL, or scholarship-listing text (no keyword) | `modes/evaluate.md` |
| `compare` | `modes/compare.md` |
| `pipeline` | `modes/pipeline.md` |
| `scan` | `modes/scan.md` |
| `tracker` | `modes/tracker.md` |

**Auto-detect evaluate:** if `$mode` is not a known keyword AND it contains a URL, or contains ≥ 2 of {scholarship, fellowship, grant, stipend, eligibility, deadline, applicants, award}, run `evaluate`. Otherwise show the discovery menu.

## Discovery menu (no / unrecognized argument)

Show this menu. If `config/profile.yml` is missing, add a note that the Seeker should start with `onboarding`.

```
scholar-ops — Scholarship Command Center

  /scholar-ops <url>        → EVALUATE: gates + score + verdict + report + tracker row
  /scholar-ops onboarding   → Build your profile (start here on first run)
  /scholar-ops scan         → Discover new scholarships from configured portals
  /scholar-ops pipeline     → Evaluate pending URLs in your inbox (data/pipeline.md)
  /scholar-ops compare      → Rank your APPLY/MAYBE scholarships by score + deadline
  /scholar-ops tracker      → View pipeline state, update statuses, deadline warnings

  Zero-token checks:  node doctor.mjs · node deadline-check.mjs · node tracker-check.mjs
```

This is a filter, not a spray-and-pray tool: it recommends against applying below 4.0/5 and never submits anything on the Seeker's behalf.
