# scholar-ops — System Architecture

Version: v1.0, 2026-07-03

## Architectural Style

**Skill-routed agent workflows over a file-based store.** Three layers in one local repository:

1. **Probabilistic layer** — an AI agent (Claude Code or compatible CLI) executing markdown mode instructions. Handles everything requiring judgment: reading listings, applying gates, scoring, filtering scan results.
2. **Deterministic layer** — dependency-free Node ≥ 18 scripts. Handles everything that must be exact and free: setup validation, deadline math, tracker hygiene. Zero tokens, zero network.
3. **Data layer** — human-readable local files (YAML config, markdown tracker/reports/inbox, TSV log). The Seeker can read and hand-edit everything; no database, no server.

Chosen over alternatives:

- **vs. standalone CLI app calling an LLM API:** the Seeker already pays for and trusts their agent CLI; reusing it removes API-key management, SDK code, and release engineering. Markdown instructions iterate faster than code and are auditable by the Seeker.
- **vs. web app:** hosting, auth, and a privacy surface for highly personal data (nationality, GPA, finances) — all liabilities the local-file model eliminates outright (INV-12 becomes a `.gitignore`, not an infrastructure program).
- **vs. pure-agent (no scripts):** deadline arithmetic and duplicate detection done probabilistically would burn tokens on work that must be exact. The deterministic layer also independently *verifies* what the probabilistic layer wrote (defense in depth for INV-02).

## Component Architecture

### Router
**Responsibility:** Map every invocation to exactly one mode; state global behavioral boundaries.
**Owned invariants:** INV-11 (system never applies)
**Inputs:** invocation argument (mode name, URL, listing text, or nothing). **Outputs:** dispatch to a mode, or the discovery menu.
**Behavior:**
1. Argument matches a mode name → dispatch (`onboarding`, `evaluate`, `compare`, `pipeline`, `scan`, `tracker`).
2. Argument is a URL or listing-like text → dispatch `evaluate` (FR-012).
3. No/unrecognized argument → discovery menu (FR-013).
4. Before any profile-dependent mode: `config/profile.yml` exists? If not → `onboarding` with explanation (FR-014).
**Must NOT:** expose any mode that performs outward actions (submit/email/register) — the dispatch table is the enforcement point for INV-11.

### ProfileStore
**Responsibility:** Hold the Seeker's identity and eligibility facts; be their only writer.
**Owned invariants:** INV-01
**Files:** `config/profile.yml` (instance, gitignored), `config/profile.example.yml` (schema template, versioned), `modes/onboarding.md` (sole writer).
**Behavior:** interview per FR-021–023 (target level always an explicit question, FR-022); write + confirm (FR-024); field-preserving updates (FR-025).
**Must NOT:** let any other mode write the profile; default or infer the scholarship level.

### Evaluator
**Responsibility:** Turn a URL/text + profile into a gated, scored, evidence-backed verdict.
**Owned invariants:** INV-04, INV-05, INV-06, INV-07, INV-08, INV-09, INV-10
**Files:** `modes/evaluate.md` (workflow), `modes/_shared.md` (gate rules, weights BR-02, verdict thresholds BR-01, budgets BR-04, UNKNOWN convention), `modes/compare.md` (reader of outputs).
**Behavior (strict order — the order is the enforcement):**
1. **Step 0 Liveness (INV-07):** WebFetch → Playwright fallback (≤ 2 navigations) → classify live/closed/expired. Dead → record DEAD, stop (EXC-01).
2. **Step 1 Block A Summary:** extract provider, name, level, fields, host, funding, duration, deadline, start. Missing → UNKNOWN (INV-06).
3. **Step 2 Block B Gates (INV-04/05):** each hard gate PASS/FAIL/UNKNOWN with verbatim quote vs profile. Any FAIL → INELIGIBLE, write short report + tracker row, **stop**.
4. **Step 3 Block C Scoring:** eight dimensions 0–5, weights from `_shared.md`, composite + letter grade. Selectivity research capped at 3 WebSearch queries (INV-08, EXC-06).
5. **Step 4 Block D Documents:** required vs `profile.documents`, gap × time-to-obtain vs deadline.
6. **Step 5 Block E Angle:** emphasis + proof-point mapping. Guidance only.
7. **Step 6 Block F Legitimacy (INV-10):** fees, domain, guaranteed-award, data-harvesting.
8. **Step 7 Verdict:** deadline passed → never APPLY (INV-09); else BR-01 thresholds. Write `reports/{provider-slug}-{name-slug}.md` + exactly one tracker row.
**Must NOT:** score after a gate FAIL; paraphrase gate evidence; invent data; exceed budgets; skip Block F.

### Tracker
**Responsibility:** Hold all application state; keep it unique, deadline-complete, and human-readable.
**Owned invariants:** INV-02
**Files:** `data/scholarships.md` (instance), row/status contract in `modes/_shared.md`, `modes/tracker.md` (views/updates), `tracker-check.mjs` + `deadline-check.mjs` (verifiers).
**Behavior:** deadline-sorted views with < 14-day flags (FR-043, BR-03); status transitions only on Seeker instruction within the closed vocabulary (FR-044, BR-06); row deletion only on explicit instruction (FR-045, FRB-04).
**Must NOT:** hold two rows for one URL; hold a row without a deadline value (`date | rolling | unknown`).

### PipelineInbox
**Responsibility:** Queue URLs between discovery and evaluation.
**Owned invariants:** INV-15
**Files:** `data/pipeline.md`, `modes/pipeline.md`.
**Behavior:** checklist entries; sequential processing — each entry evaluated via Evaluator then marked evaluated or dead, never left unmarked (FR-052); append only after dedup against pipeline + tracker (FR-053, EXC-09).
**Must NOT:** accept a URL already known to the system.

### Scanner
**Responsibility:** Bounded discovery of new candidate scholarships.
**Owned invariants:** INV-13, INV-14
**Files:** `modes/scan.md`, `portals.yml` (instance, user layer), `portals.example.yml` (template), `data/scan-history.tsv` (log).
**Behavior:** Level 1 WebSearch with queries composed from the profile (level × field) × configured `sources`, capped at `query.max_queries` (FR-062a) → Level 2 Playwright on tracked portals (FR-062); filter by profile level/field/nationality/deadline (FR-063); dedup-append survivors to PipelineInbox; log one TSV line per portal (FR-064); unreachable portal → skip + log (EXC-07).
**Must NOT:** touch sources outside `portals.yml`; evaluate anything (FRB-09) — the workflow ends at the append step.

### Toolbelt
**Responsibility:** Guarantees that hold when no mode is running: layer separation, data locality, setup health.
**Owned invariants:** INV-03, INV-12
**Files:** `DATA_CONTRACT.md`, `.gitignore`, `doctor.mjs`, `package.json` (script wiring).
**Behavior:** `doctor.mjs` checks Node version, profile presence/shape, portals config, directory scaffold, and that ignore rules cover the user layer (FR-081); update procedure (documented in `DATA_CONTRACT.md`) replaces system-layer files only.
**Must NOT:** let any script perform network or AI calls (FR-084).

## Data Architecture

### Entities

```
Profile (1 per repo)                    Portal (n, in portals.yml)
    │ read by                                │ swept by
    ▼                                        ▼
Evaluation ──produces──► Report (1)     ScanEvent (n, scan-history.tsv)
    │                                        │ appends
    └──produces──► TrackerRow (1) ◄─────  PipelineEntry (n, pipeline.md)
                        │                    (entry → row after evaluation)
                        └── unique by URL
```

### Key constraints
- **TrackerRow.url** — unique key of the whole system (INV-02). Dedup checks everywhere key on it.
- **TrackerRow.deadline** — required; domain `YYYY-MM-DD | rolling | unknown` (FR-042).
- **TrackerRow.status** — closed vocabulary BR-06; scripts flag anything else.
- **TrackerRow ↔ Report** — 1:1 for scored evaluations; INELIGIBLE/DEAD rows may carry a stub note instead of a full report.
- **PipelineEntry → TrackerRow** — an entry is consumed (marked done/dead) when its evaluation writes a row; entries and rows never coexist unmarked for the same URL.
- All files are line-oriented and hand-editable; parsers must tolerate whitespace variance (Reliability NFC).

## Flow Architecture

### Flow 1 — Evaluate a pasted URL (primary)
```
Seeker: /scholar-ops <url>
    │
    ├─► Router: profile exists? ──no──► onboarding (stop)
    │       ↓ yes → evaluate
    ├─► Evaluator Step 0: WebFetch ──fail──► Playwright ──fail──► UNVERIFIED, ask paste (stop)
    │       ↓ live                                     closed ──► tracker row status=dead (stop)
    ├─► Step 1: Block A summary (UNKNOWN for gaps)
    ├─► Step 2: Block B gates vs profile.yml ──any FAIL──► INELIGIBLE: stub report + row (stop)
    │       ↓ all PASS/UNKNOWN
    ├─► Steps 3–6: scoring (≤3 searches) · documents · angle · legitimacy
    └─► Step 7: verdict → reports/{slug}.md + tracker row (status=evaluated)
```
**Budget:** 1 fetch (+≤1 fallback), ≤3 searches, 1 report, exactly 1 row.

### Flow 2 — Discover → queue → evaluate
```
Seeker: /scholar-ops scan
    ├─► Scanner: portals.yml → L1 WebSearch → L2 Playwright (uncovered portals)
    ├─► filter: level/field/nationality/deadline  ──rejects──► dropped
    ├─► dedup vs tracker + pipeline               ──dupes────► counted, skipped
    ├─► append survivors → data/pipeline.md  +  log line → scan-history.tsv
    └─► STOP (INV-14)

Seeker (later): /scholar-ops pipeline
    └─► for each unchecked entry: Flow 1 → mark [x] evaluated | dead
```

### Flow 3 — Deadline safety loop (zero-token)
```
Seeker (any time): node deadline-check.mjs
    └─► parse every tracker row → sort ascending → print days-remaining
            ├─► < 14 days → ⚠️ flag (BR-03)
            └─► unparseable cell → exit non-zero (FR-082)
```

## Technology Mapping

| Component | Technology |
|-----------|------------|
| Router | Claude Code skill (`SKILL.md` frontmatter + routing table) + `CLAUDE.md` |
| ProfileStore | YAML + `modes/onboarding.md` |
| Evaluator | `modes/evaluate.md`, `modes/_shared.md`, `modes/compare.md`; agent WebFetch/WebSearch; Playwright via MCP |
| Tracker | Markdown table + `modes/tracker.md` + Node scripts (std lib only) |
| PipelineInbox | Markdown checklist + `modes/pipeline.md` |
| Scanner | `modes/scan.md` + YAML config + agent WebSearch/Playwright + TSV log |
| Toolbelt | `DATA_CONTRACT.md`, `.gitignore`, `doctor.mjs`, `package.json` |

No runtime npm dependencies. Playwright is consumed through the agent's MCP tooling, never as a package dependency (Portability NFC).

## Deployment Architecture

Single local git repository on the Seeker's machine. No server, no daemon, no external state. "Deployment" = `git clone` + open the AI CLI in the repo root. Updates = pull/replace system-layer files per `DATA_CONTRACT.md`; user layer untouched (INV-03).

## Project Structure

```
scholar-ops/
├── .claude/skills/scholar-ops/SKILL.md   # Router: dispatch table + discovery menu
├── CLAUDE.md                             # Router: global rules (incl. INV-11 boundary)
├── DATA_CONTRACT.md                      # Toolbelt: user layer vs system layer
├── README.md                             # Quickstart
├── package.json                          # Toolbelt: npm script wiring
├── .gitignore                            # Toolbelt: user-layer exclusion (INV-12)
├── config/
│   ├── profile.example.yml               # ProfileStore: schema template (system)
│   └── profile.yml                       # ProfileStore: instance (user, gitignored)
├── portals.example.yml                   # Scanner: template (system)
├── portals.yml                           # Scanner: instance (user, gitignored)
├── modes/
│   ├── _shared.md                        # Evaluator: gates, weights, verdicts, budgets; Tracker: row contract
│   ├── onboarding.md                     # ProfileStore: sole profile writer
│   ├── evaluate.md                       # Evaluator: Steps 0–7
│   ├── compare.md                        # Evaluator: ranking view
│   ├── pipeline.md                       # PipelineInbox: queue processing
│   ├── scan.md                           # Scanner: L1/L2 sweep
│   └── tracker.md                        # Tracker: views + status updates
├── data/                                 # user layer (gitignored contents)
│   ├── scholarships.md                   # Tracker: source of truth
│   ├── pipeline.md                       # PipelineInbox: URL queue
│   └── scan-history.tsv                  # Scanner: sweep log
├── reports/                              # Evaluator: one file per scored evaluation (user)
├── doctor.mjs                            # Toolbelt: setup validation
├── deadline-check.mjs                    # Tracker verifier: deadline math
└── tracker-check.mjs                     # Tracker verifier: dedup + status vocabulary
```

## Invariant Traceability Matrix

| Invariant | Owner | Mechanism (file · rule) | Deterministic check |
|-----------|-------|------------------------|---------------------|
| INV-01 | ProfileStore | `profile.example.yml` schema; onboarding sole writer; UNKNOWN-on-absence (`_shared.md`) | `doctor.mjs` schema check |
| INV-02 | Tracker | row contract in `_shared.md`; FR-03C exactly-one-row | `tracker-check.mjs` dedup; `deadline-check.mjs` |
| INV-03 | Toolbelt | `DATA_CONTRACT.md` layers; update touches system layer only | `doctor.mjs` layer/ignore audit |
| INV-04 | Evaluator | `evaluate.md` step order; FAIL → stop at Step 2 | — (report shape reviewable) |
| INV-05 | Evaluator | gate table requires verbatim-quote column | — |
| INV-06 | Evaluator | UNKNOWN convention (`_shared.md`) | — |
| INV-07 | Evaluator | Step 0 precedes all analysis | — |
| INV-08 | Evaluator | BR-04 caps in `_shared.md`; EXC-06 procedure | — |
| INV-09 | Evaluator | verdict rule: deadline check before thresholds | `deadline-check.mjs` flags passed dates |
| INV-10 | Evaluator | Block F mandatory section in report template | — |
| INV-11 | Router | `CLAUDE.md` global prohibition; no submit mode in table | — |
| INV-12 | Toolbelt | `.gitignore` from first commit; no-profile-in-queries rule | `doctor.mjs` ignore-rule check |
| INV-13 | Scanner | `scan.md`: `portals.yml` is the complete world | `scan-history.tsv` audit trail |
| INV-14 | Scanner | scan workflow ends at pipeline append | — |
| INV-15 | PipelineInbox | append rule in `pipeline.md`: dedup vs tracker + pipeline | `tracker-check.mjs` cross-check |

FR ↔ component: FR-010 Router · FR-020 ProfileStore · FR-030 Evaluator · FR-040 Tracker · FR-050 PipelineInbox · FR-060 Scanner · FR-070 Evaluator (compare) · FR-080 Toolbelt + Tracker verifiers.

## Architectural Constraints & ADRs

**ADR-01 — Markdown instructions as executable architecture.** Mode files are the implementation of most components. Accepted trade-off: probabilistic enforcement for judgment-dependent invariants (INV-04..10), mitigated by strict step ordering, required output shapes (tables with evidence columns), and deterministic verifiers where math is involved. Rejected alternative: coding the workflow in a CLI app — loses the agent's reading comprehension, which *is* the product.

**ADR-02 — Single shared rulebook (`modes/_shared.md`).** Weights, thresholds, budgets, row contract, and UNKNOWN convention are defined once and referenced by every mode. Prevents drift between evaluate/compare/tracker.

**ADR-03 — Human-readable files over SQLite.** A derived SQLite index earns its keep only at scale; scholar-ops MVP targets tens-to-hundreds of rows where markdown stays fast (< 2 s script budget at 500 rows) and hand-editable. Revisit only if scripts breach their performance NFC.

**ADR-04 — Scan/evaluate separation (INV-14).** Cost control and consent: a sweep may find 30 candidates; evaluating them is a separate, Seeker-initiated spend. Also keeps Scanner write access away from application state.

**ADR-05 — Zero-dependency scripts.** `npm install` must never gate MVP usage; parsing YAML-lite and markdown tables by hand in the scripts is acceptable because the formats are owned by this repo and specified exactly in `06_api_specification.md`.
