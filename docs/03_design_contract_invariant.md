# scholar-ops — System Contract & Invariants

Version: v1.0, 2026-07-03

## Actors & Allowed Actions

| Actor | Allowed | Not Allowed |
|-------|---------|-------------|
| Seeker | Edit any user-layer file by hand; instruct status changes and row deletions; approve re-evaluations; decide to apply | — (owner of everything) |
| Agent | Read/write user-layer data files through mode instructions; fetch listing pages; bounded WebSearch; write reports and tracker rows | Submit applications; contact providers; invent data; exceed research budgets; delete rows uninstructed |
| Scripts | Read user-layer files; print diagnostics; normalize status spelling under `--fix` | Network access; AI calls; deleting or rewriting rows |
| Listing Page / Portal | Serve content when fetched | Nothing else — external, untrusted input |

## System Guarantees (Invariants)

### Data Integrity

**INV-01 — Profile Is the Single Source of Personal Truth**
Every mode that reasons about the Seeker (gates, scoring, filtering) MUST read `config/profile.yml`. No mode may assume, invent, or "remember" a personal fact that is not in the profile. If a needed fact is absent, the gate or filter resolves UNKNOWN and says so.
*Worst case if violated:* evaluations silently keyed to a hallucinated nationality or GPA — every verdict untrustworthy.

**INV-02 — Tracker Is the Single Source of Application State**
`data/scholarships.md` holds application state, exactly one row per scholarship URL. Every completed evaluation writes exactly one row (append or update). No state lives only in reports, chat history, or the agent's memory.
*Worst case:* two conflicting states for one scholarship; the Seeker applies twice or not at all.

**INV-03 — User-Layer Immunity**
Files declared user-layer in `DATA_CONTRACT.md` (profile, tracker, pipeline, reports, scan history, portals.yml) are never modified by system updates (pulling a new scholar-ops version). System files never embed user data.
*Worst case:* an update wipes a cycle's worth of applications.

### Evaluation Behavior

**INV-04 — Gates Before Scores**
Hard eligibility gates run before any scoring, and any gate FAIL terminates the evaluation with INELIGIBLE. A composite score for an ineligible scholarship must never exist.
*Worst case:* a beautiful 4.6/5 report for a scholarship the Seeker cannot legally receive — the exact time-waste the product exists to kill.

**INV-05 — Verbatim Evidence**
Every gate decision (PASS/FAIL/UNKNOWN) quotes the listing's requirement line verbatim. Paraphrase is not evidence.
*Worst case:* an agent misreading "must be under 35" as "under 45" with no way for the Seeker to catch it.

**INV-06 — No Fabrication**
Any datum not found in the listing or profile is recorded UNKNOWN / unavailable. Deadlines, amounts, and criteria are never estimated into existence.
*Worst case:* a fabricated deadline two weeks later than the real one.

**INV-07 — Liveness Before Analysis**
No evaluation proceeds past a dead, closed, or deadline-passed listing. Dead links produce a DEAD record, not a report.
*Worst case:* full evaluations, reports, and tracker rows for phantom scholarships.

**INV-08 — Bounded Research**
Per evaluation: at most 3 WebSearch queries and 2 Playwright navigations. When the budget is spent, missing data is marked unavailable.
*Worst case:* one pasted URL spirals into a 40-query research session; costs explode; the Seeker stops using the tool.

**INV-09 — Deadline-Passed Never APPLY**
A scholarship whose deadline has passed can receive no verdict other than DEAD/SKIP, regardless of score.
*Worst case:* the Seeker preps documents for a closed call.

**INV-10 — Mandatory Legitimacy Check**
Every evaluation contains Block F. An application fee, unofficial domain, or guaranteed-award claim is always surfaced in both the block and the verdict line — never softened, never omitted.
*Worst case:* the tool lends its credibility to a scam.

### Human-in-the-Loop

**INV-11 — The System Never Applies**
No mode submits an application, fills a form on an external site, sends an email, or contacts a provider or third party. Output is always advice + local files. The Seeker performs every outward action.
*Worst case:* an agent "helpfully" submits a half-ready application under the Seeker's name.

### Privacy

**INV-12 — Personal Data Stays Local**
Profile, tracker, pipeline, and reports exist only on the Seeker's machine, are gitignored from the first commit, and never appear in WebSearch queries or any outbound request.
*Worst case:* nationality, birth year, GPA, and finances of a real person pushed to a public repo or leaked into search-engine logs.

### Discovery

**INV-13 — Scan Is Bounded by Configuration**
Scan touches only sources named in `portals.yml`. It is a fixed sweep, not research.
*Worst case:* unbounded crawling; cost and noise destroy trust in scan results.

**INV-14 — Scan Feeds the Inbox, Never the Evaluator**
Scan output goes to `data/pipeline.md` only. Evaluation of discovered listings happens in a separate, Seeker-initiated run.
*Worst case:* one scan triggers 30 chained evaluations and burns the day's budget without consent.

**INV-15 — Pipeline Uniqueness**
A URL enters the pipeline only if it is in neither the pipeline nor the tracker.
*Worst case:* the same scholarship evaluated three times across scan runs.

## Absolute Prohibitions (FRB)

| ID | The system MUST NEVER... |
|----|--------------------------|
| FRB-01 | Submit, register, email, or fill any form on behalf of the Seeker |
| FRB-02 | Invent a deadline, funding amount, or eligibility criterion |
| FRB-03 | Emit verdict APPLY when a hard gate failed or the deadline has passed |
| FRB-04 | Delete or rewrite tracker rows without an explicit Seeker instruction naming them |
| FRB-05 | Evaluate past the liveness gate on a dead/closed listing |
| FRB-06 | Commit or transmit profile, tracker, pipeline, or report contents (including in search queries) |
| FRB-07 | Exceed 3 WebSearch queries or 2 Playwright navigations in one evaluation |
| FRB-08 | Omit or soften a fee-to-apply or scam signal |
| FRB-09 | Auto-evaluate scan results within the scan run |
| FRB-10 | Modify user-layer files during a system update |

## Exception Handlers (EXC)

| ID | Trigger (invariant threatened) | Contracted Recovery |
|----|-------------------------------|---------------------|
| EXC-01 | Page fetch fails / JS-blank (INV-07) | One Playwright retry; then report UNVERIFIED and request pasted text. No scored report. |
| EXC-02 | Deadline unparseable (INV-06) | Record `unknown`, warn in report and tracker row, continue. |
| EXC-03 | URL already tracked (INV-02) | Present existing row + report; re-evaluate only on explicit confirmation. |
| EXC-04 | Profile missing/invalid at mode entry (INV-01) | Route to onboarding; never proceed on an assumed profile. |
| EXC-05 | Gate evidence ambiguous (INV-05) | Gate = UNKNOWN with the ambiguous text quoted; continue with a flagged verdict. |
| EXC-06 | Research budget exhausted (INV-08) | Stop; mark remaining data unavailable; note the cap in the report. |
| EXC-07 | Portal unreachable in scan (INV-13) | Skip portal, log error to `scan-history.tsv`, continue sweep. |
| EXC-08 | Malformed tracker row (INV-02) | `tracker-check` reports row + reason; repair is Seeker-driven (or `--fix` for status spelling only). |
| EXC-09 | Duplicate URL offered to pipeline (INV-15) | Skip silently; count as duplicate in the scan summary. |
| EXC-10 | Seeker asks the agent to apply/submit (INV-11) | Decline the action, explain the boundary, and produce the best possible preparation guidance instead. |
