# Data Contract

Defines which files are **user layer** (your personal data — never touched by a scholar-ops update) and which are **system layer** (logic and templates that improve with each version). Enforces INV-03 (user-layer immunity) and INV-12 (data locality).

## User Layer — NEVER auto-updated, gitignored

| File | Purpose |
|------|---------|
| `config/profile.yml` | Your identity, nationality, education, targets — single source of personal truth (INV-01) |
| `portals.yml` | Your customized discovery source list |
| `data/scholarships.md` | Your application tracker — single source of application state (INV-02) |
| `data/pipeline.md` | Your URL inbox awaiting evaluation |
| `data/scan-history.tsv` | Your scan run log |
| `reports/*.md` | Your generated evaluation reports |

These contain highly personal data (nationality, birth year, GPA, finances). They are gitignored from the first commit and must never be committed, transmitted, or placed in a WebSearch query.

## System Layer — safe to auto-update

| File | Purpose |
|------|---------|
| `.claude/skills/scholar-ops/SKILL.md` | Router: mode dispatch + discovery menu |
| `CLAUDE.md` | Global agent rules and behavioral boundaries |
| `modes/_shared.md` | Scoring weights, gate rules, verdict thresholds, budgets, row/status contract |
| `modes/onboarding.md` | Profile interview (sole profile writer) |
| `modes/evaluate.md` | Evaluation workflow (Steps 0–7) |
| `modes/compare.md` | Ranking view over evaluated scholarships |
| `modes/pipeline.md` | Inbox processing |
| `modes/scan.md` | Portal scanner |
| `modes/tracker.md` | Tracker views and status updates |
| `config/profile.example.yml` | Profile schema template (seed for `config/profile.yml`) |
| `portals.example.yml` | Portals template (seed for `portals.yml`) |
| `doctor.mjs` | Setup validation |
| `deadline-check.mjs` | Deadline math and warnings |
| `tracker-check.mjs` | Tracker dedup + status vocabulary check |
| `package.json` | Script wiring |
| `docs/*` | Engineering documentation suite |
| `DATA_CONTRACT.md` | This file |

## Update procedure

To update scholar-ops: replace the system-layer files above with a newer version. Never touch the user-layer files. A future `.example.yml` change is applied by re-copying the template only if you have not yet created your instance file.
