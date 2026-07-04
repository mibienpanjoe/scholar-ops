# scholar-ops — Interface & Data Contract Specification

Version: v1.0, 2026-07-03

> **Adaptation note:** scholar-ops exposes no HTTP API. Its contracts are (1) file formats read/written by both the agent and the scripts, (2) script command-line interfaces, and (3) the router's dispatch contract. This document specifies them exactly — it plays the role an API spec plays in a client/server system: any two components built against it must interoperate without clarifying questions.

## Conventions

- **Encoding:** UTF-8, LF line endings, all files.
- **Dates:** `YYYY-MM-DD` (ISO 8601 date), always. Special deadline values: `rolling`, `unknown` (lowercase).
- **Slugs:** lowercase ASCII, hyphen-separated, from provider/name (`DAAD` → `daad`, `Mastercard Foundation Scholars` → `mastercard-foundation-scholars`).
- **UNKNOWN convention:** a datum that cannot be established from listing or profile is written exactly `UNKNOWN` (reports/blocks) or `unknown` (tracker deadline cell) — never guessed, never blank.
- **Tolerance rule:** parsers (scripts and modes) MUST tolerate whitespace variance and column padding in markdown tables; writers SHOULD pretty-align.

## 1. Router Contract (`/scholar-ops <arg>`)

| Input `$arg` | Dispatch |
|--------------|----------|
| *(empty)* | Discovery menu (list modes, one line each; note if profile missing) |
| `onboarding` | `modes/onboarding.md` |
| `evaluate <url\|text>` | `modes/evaluate.md` |
| URL or listing-like text (no keyword) | `modes/evaluate.md` (auto-detect) |
| `compare` | `modes/compare.md` |
| `pipeline` | `modes/pipeline.md` |
| `scan` | `modes/scan.md` |
| `tracker` | `modes/tracker.md` |
| anything else | Discovery menu |

Listing-like detection: contains a URL, or ≥ 2 of the keywords {scholarship, fellowship, grant, stipend, eligibility, deadline, applicants, award}.

Pre-dispatch guard: for `evaluate`, `compare`, `pipeline`, `scan` — if `config/profile.yml` is missing or unparseable, dispatch `onboarding` instead and state why (FR-014).

## 2. `config/profile.yml` — Profile Schema

```yaml
# scholar-ops profile — single source of personal truth (INV-01). Gitignored.
identity:
  full_name: "string, required"
  email: "string, required"
  nationality: ["string, required — one or more citizenships"]
  residence_country: "string, required"
  birth_year: 1999            # integer, required (age gates)

education:
  degrees:                    # required, may be empty list for current undergrads
    - level: "bachelor"       # enum: high-school | bachelor | masters | phd
      field: "string, required"
      institution: "string, required"
      gpa: "3.6/4.0"          # string "value/scale", required; "unknown" allowed
      year: 2025              # integer graduation year; null if in progress
  currently_enrolled: false   # boolean, required

target:
  levels: ["masters"]         # required, ≥1 of: bachelor | masters | phd | exchange
                              # ALWAYS from an explicit onboarding answer (FR-022)
  fields: ["string, required, ≥1"]
  countries: ["string — preferred host countries/regions; empty = anywhere"]
  earliest_start: "2027-09"   # YYYY-MM, required

languages:                    # required, ≥1
  - language: "english"
    proficiency: "fluent"     # enum: native | fluent | intermediate | basic
    certificate: "IELTS"      # certificate name, or null
    score: "7.5"              # string, or null
    status: "held"            # enum: held | planned | none

finances:
  funding_need: "full"        # enum: full | partial-ok — required
  application_fee_tolerance: 0   # integer USD; 0 = will not pay fees

constraints:
  bond_acceptable: true       # return-home/service bonds acceptable?
  study_mode: "on-campus"     # enum: on-campus | online | either
  relocation_limits: "string or null"

documents:                    # required map; keys fixed, values enum: ready | in-progress | missing
  passport: ready
  transcripts: ready
  reference_letters: in-progress
  cv: ready
  motivation_letter_base: missing

proof_points:                 # optional
  - name: "string"
    detail: "one line — achievement, project, publication, leadership"
```

Required-field validation (used by `doctor.mjs` and onboarding): `identity.full_name`, `identity.nationality` (≥1), `identity.birth_year`, `education.degrees` (key present), `target.levels` (≥1), `target.fields` (≥1), `languages` (≥1), `finances.funding_need`, `documents` (all five keys).

## 3. `portals.yml` — Scanner Configuration

```yaml
# scholar-ops portals — Scanner's complete world (INV-13). User layer.
search_queries:               # Level 1 (WebSearch)
  - query: 'masters scholarship computer science site:scholarshipportal.com'
    note: "optional free text"

tracked_portals:              # Level 2 (Playwright)
  - name: "DAAD Scholarship Database"
    url: "https://www2.daad.de/deutschland/stipendium/datenbank/en/21148-scholarship-database/"
    enabled: true

filters:                      # applied on top of profile-derived gates (FR-063)
  max_results_per_source: 10  # integer, default 10
  exclude_keywords: ["undergraduate only"]   # optional, drop matches containing these
```

## 4. `data/scholarships.md` — Tracker Row Contract

One markdown table, exactly these columns:

```markdown
| Name | Provider | Level | Country | Deadline | Score | Verdict | Status | Report | URL |
|------|----------|-------|---------|----------|-------|---------|--------|--------|-----|
| DAAD EPOS | DAAD | masters | Germany | 2026-10-31 | 4.20 | APPLY | preparing | reports/daad-epos.md | https://... |
```

| Column | Domain |
|--------|--------|
| Name | free text |
| Provider | free text |
| Level | `bachelor \| masters \| phd \| exchange` (multi: comma-joined) |
| Country | host country, or `various` |
| Deadline | `YYYY-MM-DD \| rolling \| unknown` — required (FR-042) |
| Score | `0.00`–`5.00` two decimals, or `—` (INELIGIBLE/DEAD rows) |
| Verdict | `APPLY \| MAYBE \| SKIP \| INELIGIBLE \| DEAD` |
| Status | `found \| evaluated \| preparing \| applied \| awaiting \| interview \| won \| lost \| dead` (BR-06) |
| Report | repo-relative path, or `—` |
| URL | absolute URL — **unique key** (INV-02) |

Rules: rows kept deadline-ascending (`rolling`/`unknown` last); UNKNOWN gate flags appended to Verdict as ` ⚠` ; writers append/update exactly one row per evaluation (FR-03C).

## 5. `data/pipeline.md` — Inbox Entry Contract

```markdown
# Pipeline — pending scholarship URLs

- [ ] https://example.org/scholarship-x | DAAD scan 2026-07-03 | deadline 2026-10-31
- [x] https://example.org/scholarship-y | manual add | evaluated → tracker
- [x] ~~https://example.org/scholarship-z~~ | scan | dead link
```

- Entry: `- [ ] <url> | <source note> | <optional deadline/status note>`
- Processed-evaluated: `- [x]` + trailing `evaluated → tracker`
- Processed-dead: `- [x]` + struck-through URL + reason
- Append guard: URL absent from this file **and** from the tracker URL column (FR-053, INV-15).

## 6. `reports/{provider-slug}-{name-slug}.md` — Evaluation Report Contract

Required sections, in order:

```markdown
# {Name} — Evaluation
Evaluated: YYYY-MM-DD · Source: {url} · Liveness: live|text-only(UNVERIFIED)

## Verdict
{APPLY|MAYBE|SKIP|INELIGIBLE} · Score {n.nn}/5 ({A-F}) · Deadline {value} ({n} days)
{⚠ flags: unknown gates, legitimacy flags — one line each}

## A — Summary            (table: provider, level, fields, host, funding, duration, deadline, start)
## B — Eligibility Gates  (table: Gate | Requirement (verbatim quote) | Profile fact | PASS/FAIL/UNKNOWN)
## C — Fit Scores         (table: Dimension | Weight | Score 0–5 | Rationale)   — omitted if INELIGIBLE
## D — Documents          (table: Required | Status from profile | Gap action | Time vs deadline) — omitted if INELIGIBLE
## E — Application Angle  (prose bullets)                                        — omitted if INELIGIBLE
## F — Legitimacy         (table: Signal | Finding | Flag)
```

INELIGIBLE reports stop after B + F (stub report). DEAD listings get no report, only a tracker row.

## 7. `data/scan-history.tsv` — Scan Log Contract

Tab-separated, one line per portal per run, header row on creation:

```
timestamp	portal	found	added	duplicates	errors
2026-07-03T14:20:00Z	DAAD Scholarship Database	12	3	2	
2026-07-03T14:22:10Z	scholarshipportal.com	9	1	0	timeout on page 2
```

## 8. Script CLIs

All scripts: Node ≥ 18, zero dependencies, zero network, zero AI (FR-084). Output to stdout, errors to stderr.

### `node doctor.mjs`
Checks (one line each, `✓`/`✗` + reason): Node ≥ 18 · `config/profile.yml` exists, parses, required fields (§2) · `portals.yml` exists and parses (warn-only if missing) · `data/` and `reports/` exist · `.gitignore` covers user layer.
**Exit codes:** `0` all required checks pass · `1` any required check fails.

### `node deadline-check.mjs [--days N]`
Parses every tracker row; prints `deadline · days-remaining · name · status`, ascending; `rolling`/`unknown` listed last; flags `⚠` under N days (default 14, BR-03); flags `✗ PASSED` for past dates.
**Exit codes:** `0` ok · `1` tracker missing · `2` ≥1 unparseable deadline cell (FR-082).

### `node tracker-check.mjs [--fix]`
Reports: duplicate URLs · statuses outside BR-06 vocabulary · malformed rows (wrong column count). `--fix` normalizes status case/spelling only; never deletes or reorders rows (FR-083, EXC-08).
**Exit codes:** `0` clean · `1` findings reported (with or without `--fix`).

### `package.json` wiring
`npm run doctor` · `npm run deadlines` · `npm run tracker-check` → the three scripts above.

## 9. Outbound Calls (agent-side, not scripts)

| Call | Used by | Budget | Policy |
|------|---------|--------|--------|
| WebFetch(listing URL) | Evaluator Step 0 | 1/eval | fallback → Playwright |
| Playwright navigate | Evaluator Step 0; Scanner L2 | ≤2/eval; 1/portal | JS-rendered pages only |
| WebSearch | Evaluator Step 3 (selectivity); Scanner L1 | ≤3/eval; per configured query | MUST NOT contain profile data (INV-12) |

## 10. Interface Summary Table

| Interface | Producer | Consumer | Spec |
|-----------|----------|----------|------|
| `config/profile.yml` | onboarding | all modes, doctor | §2 |
| `portals.yml` | Seeker (from template) | scan, doctor | §3 |
| `data/scholarships.md` | evaluate | tracker/compare modes, scripts | §4 |
| `data/pipeline.md` | scan, Seeker | pipeline mode | §5 |
| `reports/*.md` | evaluate | Seeker, compare | §6 |
| `data/scan-history.tsv` | scan | Seeker, audits | §7 |
| Script CLIs | scripts | Seeker, npm | §8 |
| Router dispatch | SKILL.md | agent | §1 |
