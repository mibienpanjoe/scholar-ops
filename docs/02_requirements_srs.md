# scholar-ops — Software Requirements Specification

Version: v1.0, 2026-07-03

## Normative Vocabulary

- **MUST / MUST NOT / REQUIRED**: Absolute requirement. The system fails if violated.
- **SHOULD / SHOULD NOT**: Recommended. Deviation permitted with documented justification.
- **MAY**: Optional capability.

A note on enforcement layers: scholar-ops has a **probabilistic layer** (agent behavior directed by markdown mode instructions) and a **deterministic layer** (Node scripts). Requirements on the agent are enforced by explicit, unambiguous mode instructions plus deterministic verification where possible (scripts, file contracts). Requirements on scripts are enforced by code.

## Actors

| Actor | Description |
|-------|-------------|
| Seeker | The human user; owns the profile, makes all apply/skip decisions |
| Agent | The AI CLI (Claude Code or compatible) executing mode instructions |
| Listing Page | External scholarship listing (provider website, portal page) |
| Portal | External discovery source configured in `portals.yml` |
| Scripts | Deterministic Node utilities (`doctor.mjs`, `deadline-check.mjs`, `tracker-check.mjs`) |

## Functional Requirements

### FR-010: Routing (skill entry)

- **FR-011**: The router MUST map the invocation argument to exactly one mode: `onboarding`, `evaluate`, `compare`, `pipeline`, `scan`, `tracker`, or discovery menu (no argument).
- **FR-012**: If the argument is a URL or scholarship-listing text rather than a known mode name, the router MUST dispatch to `evaluate`.
- **FR-013**: If the argument is neither a known mode nor listing-like input, the router MUST show the discovery menu (command list with one-line descriptions).
- **FR-014**: Before dispatching to `evaluate`, `compare`, `pipeline`, or `scan`, the router MUST verify `config/profile.yml` exists; if missing, it MUST route to `onboarding` instead and say why.

### FR-020: Onboarding

- **FR-021**: Onboarding MUST collect, via conversational interview: full name, contact, nationality/citizenship(s), country of residence, birth year, education history (each degree: level, field, institution, GPA + scale, year), current enrollment status.
- **FR-022**: Onboarding MUST ask the target scholarship level(s) (bachelor / masters / phd / exchange) as an explicit question. It MUST NOT infer or default the level.
- **FR-023**: Onboarding MUST collect: target fields of study, preferred countries/regions, earliest start date, language proficiencies and certificates (held with score, or planned), funding need (full required vs partial acceptable), application-fee tolerance, constraints (return-home bond acceptance, online vs on-campus, relocation limits), and document readiness (passport, transcripts, reference letters, CV, motivation-letter base).
- **FR-024**: Onboarding MUST write the answers to `config/profile.yml` conforming to the schema in `06_api_specification.md`, and MUST show the resulting file to the Seeker for confirmation.
- **FR-025**: If `config/profile.yml` already exists, onboarding MUST offer update (field-by-field) rather than overwrite, and MUST NOT drop existing fields the Seeker does not mention.

### FR-030: Evaluation

- **FR-031 (liveness gate)**: For URL input, the agent MUST fetch the page (WebFetch first; Playwright fallback for JS-rendered pages) and classify it live/closed before any analysis. Closed, 404, redirected-to-generic, or deadline-passed listings MUST terminate the evaluation with a DEAD result and MUST NOT produce a scored report.
- **FR-032**: For pasted text input (no URL), the agent MUST note that liveness cannot be verified and proceed.
- **FR-033 (Block A — summary)**: The agent MUST extract: provider, scholarship name, level(s), eligible fields, host country/institution, funding type and amount (full / partial / tuition-only / stipend), duration, application deadline, program start date. Missing values MUST be recorded as UNKNOWN.
- **FR-034 (Block B — eligibility gates)**: The agent MUST evaluate hard gates before any scoring: nationality/citizenship, age, required degree level, field restriction, GPA minimum, language certificate, residency, and any other binary criterion stated by the listing. Each gate decision MUST quote the listing's requirement line verbatim and compare it against `config/profile.yml`.
- **FR-035**: Each gate MUST resolve to PASS, FAIL, or UNKNOWN (evidence ambiguous or profile datum missing). Any FAIL MUST set the verdict INELIGIBLE and terminate the evaluation before scoring. UNKNOWN gates MUST NOT terminate; they MUST be flagged in the report and verdict line.
- **FR-036 (Block C — scoring)**: If all gates pass (or are UNKNOWN), the agent MUST score eight dimensions 0–5 using the weights in `modes/_shared.md`: funding coverage, eligibility margin, field match, deadline feasibility, selectivity, application effort, career value, constraints fit — and compute the weighted composite (0–5, two decimals) and letter grade (A ≥ 4.5, B ≥ 4.0, C ≥ 3.0, D ≥ 2.0, F < 2.0).
- **FR-037**: Selectivity research MUST be single-pass with a hard cap of 3 WebSearch queries per evaluation. When the cap is reached, remaining unknowns MUST be marked unavailable, not guessed.
- **FR-038 (Block D — documents)**: The agent MUST produce a checklist of documents the listing requires vs the Seeker's readiness (from profile), with an estimated time-to-obtain for each gap compared against the deadline.
- **FR-039 (Block E — angle)**: The agent MUST produce application-angle guidance: what to emphasize, which profile proof points map to the scholarship's selection criteria. It MUST NOT generate application documents.
- **FR-03A (Block F — legitimacy)**: The agent MUST check and report: application fees (any fee → prominent red flag), domain officiality, guaranteed-award or pay-to-win claims, requests for sensitive data disproportionate to a legitimate application.
- **FR-03B (verdict)**: The verdict MUST be: INELIGIBLE (any gate FAIL), else APPLY (composite ≥ 4.0), MAYBE (3.0–3.9), SKIP (< 3.0). A passed deadline MUST NOT produce APPLY under any circumstance.
- **FR-03C (output)**: A completed evaluation MUST write a report to `reports/{provider-slug}-{name-slug}.md` containing blocks A–F and the verdict, and MUST append or update exactly one row in `data/scholarships.md`.
- **FR-03D (re-evaluation)**: If the URL already exists in the tracker, the agent MUST surface the existing row and report and ask before re-evaluating.

### FR-040: Tracking

- **FR-041**: `data/scholarships.md` MUST be the single source of truth for application state. Its row format and status vocabulary are defined in `06_api_specification.md`.
- **FR-042**: Every row MUST carry a deadline value: an ISO date, `rolling`, or `unknown`.
- **FR-043**: Tracker mode MUST render the pipeline deadline-sorted (soonest first; `rolling`/`unknown` last) and flag rows with deadlines under 14 days.
- **FR-044**: Status transitions MUST follow the vocabulary `found → evaluated → preparing → applied → awaiting → interview → result(won|lost)`; the agent MUST update statuses only on the Seeker's instruction.
- **FR-045**: The agent MUST NOT delete tracker rows except on explicit Seeker instruction naming the row.

### FR-050: Pipeline inbox

- **FR-051**: `data/pipeline.md` MUST hold pending scholarship URLs as checklist entries (format in `06_api_specification.md`).
- **FR-052**: Pipeline mode MUST process entries sequentially: evaluate each per FR-030, then mark the entry done (evaluated) or dead (liveness failure), never leaving an entry unmarked after processing.
- **FR-053**: A URL MUST NOT be added to the pipeline if it already appears in the pipeline or in the tracker.

### FR-060: Scan (discovery)

- **FR-061**: Scan MUST read `portals.yml` and MUST restrict discovery to its configured `search_queries` and `tracked_portals`; scanning is bounded configuration-driven work, never open-ended research.
- **FR-062**: Scan MUST execute in two levels: Level 1 WebSearch (queries from config), Level 2 Playwright navigation (tracked portal URLs). Level 2 SHOULD only run for portals not satisfied by Level 1.
- **FR-063**: Scan MUST filter candidate listings against the profile before adding to the pipeline: target level match, field relevance, nationality not obviously excluded, deadline not passed.
- **FR-064**: Surviving candidates MUST be appended to `data/pipeline.md` per FR-053 (deduplicated), and every scan run MUST append one line per portal to `data/scan-history.tsv` (timestamp, portal, found, added, errors).
- **FR-065**: Scan MUST NOT evaluate listings in the same run; evaluation happens only via pipeline or evaluate modes.

### FR-070: Compare

- **FR-071**: Compare mode MUST rank all rows with verdict APPLY or MAYBE by composite score, presenting deadline, score, funding type, and outstanding document gaps side by side.
- **FR-072**: Compare MUST surface deadline pressure: any compared scholarship with a deadline under 14 days MUST be flagged, and ordering ties MUST break toward the sooner deadline.

### FR-080: Diagnostics scripts

- **FR-081**: `doctor.mjs` MUST verify: Node ≥ 18, `config/profile.yml` exists and parses with required fields present, `portals.yml` exists and parses, `data/` and `reports/` directories exist. It MUST print one line per check (pass/fail with reason) and exit 0 only if all required checks pass.
- **FR-082**: `deadline-check.mjs` MUST parse every tracker row, print deadlines sorted ascending with days-remaining, flag < 14 days, and exit non-zero if any row's deadline cell is unparseable (not ISO, `rolling`, or `unknown`).
- **FR-083**: `tracker-check.mjs` MUST detect duplicate URLs and non-vocabulary statuses, reporting findings without modifying the file; with `--fix` it MAY normalize status spelling but MUST NOT delete rows.
- **FR-084**: Scripts MUST run offline, deterministic, zero-token (no network, no AI calls), with no dependencies outside the Node standard library.

## Business Rules

| ID | Rule |
|----|------|
| BR-01 | Verdict thresholds: APPLY ≥ 4.0, MAYBE 3.0–3.9, SKIP < 3.0 (composite, 0–5 scale). INELIGIBLE overrides all. |
| BR-02 | Scoring weights (sum 100): funding coverage 20, eligibility margin 15, field match 15, deadline feasibility 15, selectivity 10, application effort 10, career value 10, constraints fit 5. Defined once in `modes/_shared.md`. |
| BR-03 | Deadline warning threshold: 14 days. Everywhere a deadline is displayed, < 14 days is flagged. |
| BR-04 | Research budget: max 3 WebSearch queries per evaluation, max 2 Playwright navigations per liveness check. |
| BR-05 | Any application fee is a legitimacy red flag; it never blocks evaluation but must appear in Block F and the verdict line. |
| BR-06 | The status vocabulary is closed: `found, evaluated, preparing, applied, awaiting, interview, won, lost, dead`. |
| BR-07 | The system recommends; the Seeker decides. No mode submits, emails, registers, or fills forms on external sites. |

## Non-Functional Constraints

### Token & Cost Budget
- One evaluation SHOULD cost at most: 1 page fetch (+1 Playwright fallback), 3 WebSearch queries, and the analysis itself.
- Diagnostics MUST cost zero tokens (FR-084).
- Scan cost MUST be proportional to `portals.yml` size, never open-ended.

### Performance
- Scripts MUST complete in < 2 seconds on a tracker of 500 rows.
- Evaluation wall-time is agent-bound; the mode instructions MUST NOT require unbounded retries.

### Portability
- Node ≥ 18, no build step, no runtime dependencies (Playwright only via the agent's MCP tooling, not a package dependency).
- Repo MUST function after `git clone` + opening the CLI; `npm install` MUST NOT be required for MVP features.

### Data Privacy
- `config/profile.yml`, `data/*`, `reports/*` MUST be gitignored from the first commit.
- Personal data MUST NOT be transmitted anywhere except as context within the Seeker's own CLI session.
- Modes MUST NOT include profile contents in WebSearch queries.

### Reliability
- The tracker file MUST remain human-readable and hand-editable; scripts and modes MUST tolerate manual edits (whitespace, column spacing).

## Error Cases

| ID | Trigger | Required Behavior |
|----|---------|-------------------|
| ERR-01 | URL fetch fails or page is JS-rendered blank | Retry once via Playwright; if still unusable, report UNVERIFIED and ask Seeker to paste the listing text. No scored report. |
| ERR-02 | Listing deadline unparseable | Record `unknown`, warn in report, continue evaluation. |
| ERR-03 | URL already in tracker | Show existing row + report path; re-evaluate only on confirmation (FR-03D). |
| ERR-04 | `config/profile.yml` missing or unparseable at mode entry | Route to onboarding (FR-014); never evaluate against an assumed profile. |
| ERR-05 | Gate evidence ambiguous / profile datum missing | Gate = UNKNOWN, flag in report, continue (FR-035). |
| ERR-06 | Portal unreachable during scan | Skip portal, log error line to `scan-history.tsv`, continue remaining portals. |
| ERR-07 | Research cap reached with unknowns remaining | Stop searching; mark data unavailable (FR-037). |
| ERR-08 | Malformed tracker row found by scripts | Report row and reason; never silently delete or rewrite (FR-083). |
| ERR-09 | Duplicate URL offered to pipeline | Silently skip, count as duplicate in scan summary (FR-053). |
