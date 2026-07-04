# scholar-ops — Transition: Requirements to Architecture

Version: v1.0, 2026-07-03

## Method

Every guarantee in `03_design_contract_invariant.md` is assigned to exactly one component owner. Not a technology, not a file — a conceptual responsibility. If an invariant is violated, its owner is the component whose definition (mode instructions, script code, or contract file) must be fixed.

scholar-ops components are unusual in one way: most are implemented as **agent instructions (markdown) plus the data files they govern**, not as running code. Ownership still works identically — the "enforcement point" is the specific instruction block or script that makes the guarantee hold.

## Component Definitions

**Router** — The single entry point (`.claude/skills/scholar-ops/SKILL.md` + `CLAUDE.md` global rules). Maps user input to exactly one mode, detects listing-like input, shows the discovery menu, blocks entry into profile-dependent modes when no profile exists, and states the global behavioral boundaries every mode inherits.

**ProfileStore** — The Seeker's identity and eligibility facts: `config/profile.yml`, its schema, and `modes/onboarding.md` (the only writer). Every other component is a reader.

**Evaluator** — The core analysis workflow: `modes/evaluate.md` plus the scoring system, gate rules, and research budgets defined in `modes/_shared.md`. Consumes a URL/text and the profile; produces a report and exactly one tracker row. `modes/compare.md` reads its outputs.

**Tracker** — Application state: `data/scholarships.md`, its row/status contract in `modes/_shared.md`, the viewing/updating instructions in `modes/tracker.md`, and `tracker-check.mjs` / `deadline-check.mjs` as deterministic verifiers.

**PipelineInbox** — The queue between discovery and evaluation: `data/pipeline.md`, entry format, and the processing instructions in `modes/pipeline.md`.

**Scanner** — Bounded discovery: `modes/scan.md` + `portals.yml` + `data/scan-history.tsv`. Sweeps configured sources, filters against the profile, and feeds the PipelineInbox.

**Toolbelt** — Repo hygiene and the system/user boundary: `DATA_CONTRACT.md`, `.gitignore`, and `doctor.mjs`. Owns the guarantees that hold *regardless of which mode runs*: layer separation and data locality.

## Invariant Assignments

### Router (owns: INV-11)
The never-apply guarantee is a property of what modes exist and what every mode is told before it runs. Router owns it because no mode can be reached except through Router's dispatch table, and the prohibition is stated at the global-instruction level (`CLAUDE.md`), not per-mode. If a "submit" capability ever appears, it is Router's dispatch table that admitted it.

### ProfileStore (owns: INV-01)
Single source of personal truth. ProfileStore is the only component allowed to write the profile (via onboarding), and its schema defines what "a personal fact" is. Every mode that needs a personal fact is instructed to read the file, and the UNKNOWN-on-absence rule lives with the schema.

### Evaluator (owns: INV-04, INV-05, INV-06, INV-07, INV-08, INV-09, INV-10)
Gates-before-scores, verbatim evidence, no fabrication, liveness gating, bounded research, deadline-passed-never-APPLY, and the mandatory legitimacy block are all properties of the evaluation workflow. They are enforced as ordered, explicit steps in `modes/evaluate.md` — the liveness gate is Step 0 and gates are Step 1 precisely so later steps structurally cannot run first.

### Tracker (owns: INV-02)
One row per URL, no state outside the file, and — as part of the same row contract — a deadline value on every row (FR-042). Enforced by the row format contract in `_shared.md` (modes must follow it) and verified deterministically by `tracker-check.mjs` and `deadline-check.mjs`.

### PipelineInbox (owns: INV-15)
Pipeline uniqueness. The append rule ("check tracker and pipeline before adding") lives in the inbox's own contract, and both writers (Scanner, Seeker-driven adds) are instructed to obey it there — one definition, two readers, one owner.

### Scanner (owns: INV-13, INV-14)
Configuration-bounded sweep and inbox-only output are properties of the scan workflow: `modes/scan.md` reads `portals.yml` as its complete world and ends at the pipeline append step. There is no evaluation step to reach.

### Toolbelt (owns: INV-03, INV-12)
User-layer immunity and data locality hold across all modes and across time (updates), so they cannot belong to any single workflow. `DATA_CONTRACT.md` declares the layers, `.gitignore` enforces locality at the VCS boundary, and `doctor.mjs` verifies both mechanically.

## Invariant Coverage Table

| Invariant | Owner | Enforcement Point |
|-----------|-------|-------------------|
| INV-01 Profile single source of personal truth | ProfileStore | `profile.yml` schema + onboarding as sole writer; UNKNOWN-on-absence rule in `_shared.md` |
| INV-02 Tracker single source of state (incl. deadline column completeness) | Tracker | Row contract in `_shared.md`; `tracker-check.mjs` + `deadline-check.mjs` verification |
| INV-03 User-layer immunity | Toolbelt | `DATA_CONTRACT.md` layer declaration; update procedure touches system layer only |
| INV-04 Gates before scores | Evaluator | `evaluate.md` step order: gates are Step 1, scoring unreachable after FAIL |
| INV-05 Verbatim evidence | Evaluator | Gate table format requires quoted requirement column |
| INV-06 No fabrication | Evaluator | UNKNOWN convention in `_shared.md`, applied per block |
| INV-07 Liveness before analysis | Evaluator | `evaluate.md` Step 0 liveness gate |
| INV-08 Bounded research | Evaluator | Hard caps stated in `_shared.md`; cap-reached procedure in `evaluate.md` |
| INV-09 Deadline-passed never APPLY | Evaluator | Verdict rules: deadline check precedes verdict assignment |
| INV-10 Mandatory legitimacy check | Evaluator | Block F is a required report section; verdict line carries flags |
| INV-11 System never applies | Router | Global prohibition in `CLAUDE.md`; no submit-capable mode in dispatch table |
| INV-12 Personal data stays local | Toolbelt | `.gitignore` from first commit; no-profile-in-queries rule; `doctor.mjs` checks ignore rules |
| INV-13 Scan bounded by configuration | Scanner | `scan.md` reads `portals.yml` as its complete source list |
| INV-14 Scan feeds inbox only | Scanner | `scan.md` terminates at pipeline append; no evaluation step exists |
| INV-15 Pipeline uniqueness | PipelineInbox | Append rule in `pipeline.md` contract; dedup check against tracker + pipeline |

## Coupling & Cohesion Decisions

**Evaluator owns seven invariants — deliberately.** All seven are properties of a single ordered workflow; splitting them across owners would recreate the diffusion problem this document exists to prevent. The workflow's step order *is* the enforcement mechanism.

**Tracker and PipelineInbox are separate components.** Both are markdown data files, but they answer different questions ("what is the state of things I evaluated?" vs "what is waiting to be evaluated?") and have different writers (Evaluator vs Scanner/Seeker). Merging them would give the Scanner write access to application state, violating the spirit of INV-02.

**Toolbelt exists because two invariants outlive every workflow.** Layer separation and privacy must hold even when no mode is running (e.g., during a `git push` or a version update). They need an owner whose enforcement points are repo artifacts, not instructions.

**Scripts are verifiers, not owners.** `tracker-check.mjs` verifies INV-02 but Tracker owns it: the guarantee is defined by the row contract, and the script only detects drift. Assigning ownership to verification code would leave the guarantee undefined when the script isn't run.

**compare.md lives inside Evaluator.** It reads Evaluator's outputs (scores, verdicts) and adds no new state; giving it component status would add a boundary with nothing to protect.
