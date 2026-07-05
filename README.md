# scholar-ops

**An AI scholarship command center for your terminal.** Paste a scholarship link; scholar-ops checks it against your profile — hard eligibility gates first, then weighted scoring — and gives you a verdict, a report, and a deadline-aware tracker row. Spend your time applying to the few scholarships worth winning, not digging through hundreds.

> This is a **filter, not a spray-and-pray tool.** It recommends against applying below 4.0/5, and it never submits anything on your behalf — you always decide and apply.

Built on [Claude Code](https://claude.com/claude-code). Scholarships live and die by eligibility walls and deadlines — so scholar-ops checks the walls first, quotes the disqualifying line back to you, and keeps everything sorted by deadline.

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

## Dashboard (optional)

A terminal dashboard over your tracker files, written in Rust + [ratatui](https://ratatui.rs), lives in [`tui/`](tui/). It's a **read-mostly viewer** — browse the tracker, sort by deadline, scroll reports, triage the scan inbox, all with **zero tokens**. Its only write is a row's `Status`; Claude still does every evaluation, gate, and score.

```bash
cd tui && cargo run --release     # or: cargo run --release --manifest-path tui/Cargo.toml
```

Run it from the repo root (it finds `data/` by walking up). Requires the Rust toolchain ([rustup](https://rustup.rs)).

| Key | Action |
|-----|--------|
| `↑`/`↓` or `j`/`k` | Move selection |
| `Tab` | Switch Tracker ↔ Pipeline |
| `PgUp`/`PgDn` | Scroll the report pane |
| `s` | Set the selected row's status (the only write) |
| `/` | Filter by name/provider · `u` urgent (<14d) · `v` cycle verdict |
| `r` | Reload from disk · `q` quit |

## Configuration

- Copy `config/profile.example.yml` → `config/profile.yml` (or let onboarding do it).
- Copy `portals.example.yml` → `portals.yml` and set the **sources** (sites + tracked portals) to sweep. Scan composes the search queries from your profile (level × field) — you don't hand-write them. Onboarding can seed this file for you.

Both instance files are **gitignored** — your personal data never enters version control (see `DATA_CONTRACT.md`).

## How it's built

Markdown "mode" instructions in `modes/` drive the agent; the shared rulebook (`modes/_shared.md`) defines the gates, scoring weights, verdict thresholds, and file contracts once, so nothing drifts. Deterministic Node scripts handle the exact-and-free work (deadline math, dedup) and independently verify what the agent wrote. No server, no API keys beyond your own CLI, no runtime dependencies.

Full engineering documentation — PRD, SRS, invariants, architecture, interface contracts — lives in [`docs/`](docs/).

## Requirements

- An agent-skill CLI (Claude Code or compatible)
- Node.js ≥ 18 (for the diagnostics scripts)
- Rust toolchain (optional — only for the `tui/` dashboard)

## License

MIT.
