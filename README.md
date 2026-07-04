# scholar-ops

**An AI scholarship command center for your terminal.** Paste a scholarship link; scholar-ops checks it against your profile — hard eligibility gates first, then weighted scoring — and gives you a verdict, a report, and a deadline-aware tracker row. Spend your time applying to the few scholarships worth winning, not digging through hundreds.

> This is a **filter, not a spray-and-pray tool.** It recommends against applying below 4.0/5, and it never submits anything on your behalf — you always decide and apply.

Built on [Claude Code](https://claude.com/claude-code). Inspired by [santifer/career-ops](https://github.com/santifer/career-ops), which does the same for job listings — scholar-ops adapts the pattern to scholarships, where eligibility is more binary and deadlines rule everything.

## Why

Finding scholarships is easy; *filtering* them is the time sink. Most listings fail on a hard wall — nationality, age, degree level, GPA, a language certificate — that you only discover after reading the whole page. scholar-ops reads it for you, checks the walls first, and quotes the disqualifying line verbatim so you can trust the call.

## What it does

- **Evaluates** a scholarship URL: liveness check → hard eligibility gates (any fail = INELIGIBLE, stop) → weighted fit score across 8 dimensions → document checklist → application angle → scam/legitimacy check. Verdict: 🟢 APPLY · 🟡 MAYBE · 🔴 SKIP · ⛔ INELIGIBLE · 💀 DEAD.
- **Tracks** everything in `data/scholarships.md`, always sorted by deadline, with warnings under 14 days.
- **Scans** the portals you configure (DAAD, Campus France, Erasmus Mundus, Chevening, Mastercard Foundation, aggregators) for new matches, filtered to your profile.
- **Checks** your setup and deadlines with zero-token Node scripts.

## Quick start

```bash
# 1. Open this repo with your AI CLI
claude          # (or another agent-skill CLI)

# 2. Build your profile — an interview, nothing to hand-edit
/scholar-ops onboarding

# 3. Evaluate a scholarship
/scholar-ops https://www.daad.de/some-scholarship

# 4. See your pipeline
/scholar-ops tracker
```

On first launch the onboarding interview asks **which level(s) you're after — bachelor, masters, PhD, or exchange** (it never assumes), plus your nationality, education, target fields and countries, languages and certificates, finances, and which documents you have ready. It writes `config/profile.yml`, which stays on your machine.

## Commands

| Command | Does |
|---------|------|
| `/scholar-ops <url>` | Evaluate a scholarship → verdict + report + tracker row |
| `/scholar-ops onboarding` | Build or update your profile (start here) |
| `/scholar-ops scan` | Discover new scholarships from your configured portals |
| `/scholar-ops pipeline` | Evaluate the pending URLs in your inbox |
| `/scholar-ops compare` | Rank your APPLY/MAYBE scholarships by score + deadline |
| `/scholar-ops tracker` | View pipeline, update statuses, deadline warnings |

Zero-token diagnostics (plain Node, no dependencies):

```bash
node doctor.mjs           # validate setup (Node, profile, portals, dirs, .gitignore)
node deadline-check.mjs   # every deadline, sorted, flagged under 14 days
node tracker-check.mjs    # duplicate URLs, bad statuses, malformed rows (--fix normalizes status spelling)
```

## Configuration

- Copy `config/profile.example.yml` → `config/profile.yml` (or let onboarding do it).
- Copy `portals.example.yml` → `portals.yml` and tune the queries to your level, fields, and region.

Both instance files are **gitignored** — your personal data never enters version control (see `DATA_CONTRACT.md`).

## How it's built

Markdown "mode" instructions in `modes/` drive the agent; the shared rulebook (`modes/_shared.md`) defines the gates, scoring weights, verdict thresholds, and file contracts once, so nothing drifts. Deterministic Node scripts handle the exact-and-free work (deadline math, dedup) and independently verify what the agent wrote. No server, no API keys beyond your own CLI, no runtime dependencies.

Full engineering documentation — PRD, SRS, invariants, architecture, interface contracts — lives in [`docs/`](docs/).

## Requirements

- An agent-skill CLI (Claude Code or compatible)
- Node.js ≥ 18 (for the diagnostics scripts)

## License

MIT. Credit and thanks to [career-ops](https://github.com/santifer/career-ops) for the ops-style pattern this project adapts.
