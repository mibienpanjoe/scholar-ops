# scholar-ops — Product Requirements Document

Version: v1.0, 2026-07-03

## 1. Problem Statement

Scholarship seekers drown in listings, not in opportunities. Aggregators and portals surface hundreds of scholarships, but most fail on binary eligibility walls — nationality, age, degree level, GPA minimums, language certificates — that are only discovered after reading the full listing. A seeker can burn days reading 40 pages to find 35 disqualifications. Meanwhile the few genuinely winnable scholarships are governed by hard deadlines and document requirements (transcripts, references, IELTS/TOEFL) with lead times of weeks. Add scam listings (fee-to-apply, data harvesting) and spreadsheet tracking that decays every cycle, and the result is: time goes to digging, not to applying well.

Companies solved the mirror problem for jobs with career-ops — AI that filters listings against a candidate profile. scholar-ops applies the same discipline to scholarships, where the filtering problem is even more binary and deadline-driven.

## 2. Personas

### Primary — The Seeker
A student or recent graduate (reference case: a West African computer-science graduate targeting funded Masters/PhD programs abroad). Comfortable in a terminal, already uses an AI coding CLI. Applies for themselves. Wants: fewer listings read, zero ineligible applications submitted, no missed deadlines.

### Secondary — The Returning Applicant
The same user in a later cycle (rejected last year, re-applying; or moving from Masters search to PhD search). Wants: a pipeline that survives across cycles, past evaluations reusable, profile updated once and reflected everywhere.

### Tertiary — The Agent
The AI CLI (Claude Code or compatible) executing the modes. Not a human, but a first-class reader of every document in this repo: instructions must be unambiguous enough for the agent to follow without inventing behavior.

## 3. Solution Overview

scholar-ops is a repository the seeker clones and opens with their AI CLI. A skill router (`/scholar-ops`) dispatches to markdown-defined modes. An onboarding interview builds a personal profile (including target scholarship level — always asked, never assumed). From then on, pasting a scholarship URL triggers a disciplined evaluation: liveness check, hard eligibility gates (any fail stops everything), weighted fit scoring, document-gap checklist, application angle, and legitimacy check — producing a verdict (APPLY / MAYBE / SKIP / INELIGIBLE), a report file, and a row in a deadline-sorted tracker. A bounded portal scanner discovers new candidates into a pipeline inbox. Deterministic Node scripts handle validation, deadline warnings, and tracker hygiene without spending tokens. The system never submits anything: it filters, the human applies.

## 4. MVP Scope

### Onboarding
- Chat interview that produces `config/profile.yml`
- Asks target scholarship level(s) (bachelor / masters / PhD / exchange) as an explicit question
- Captures: identity, nationality/citizenship, education history (degrees, GPA + scale), fields of study, preferred countries, languages and certificates (held or planned), finances (funding need), constraints (bonds, relocation, on-campus vs online), documents ready
- Detects a missing profile at any entry point and routes here first

### Evaluation (the core)
- Input: scholarship URL or pasted listing text
- Liveness gate: dead, closed, or deadline-passed listings are rejected before any analysis
- Hard eligibility gates evaluated first, each quoting the listing verbatim; any FAIL → INELIGIBLE verdict, evaluation stops
- Weighted fit scoring (0–5 per dimension, composite score + letter grade): funding coverage, eligibility margin, field match, deadline feasibility, selectivity, application effort, career value, constraints fit
- Document checklist: required vs ready (from profile), gaps with time-to-obtain vs deadline
- Application angle: what to emphasize, which proof points (guidance only — no document generation)
- Legitimacy check: fee-to-apply, unofficial domains, guaranteed-award claims, data-harvesting patterns
- Output: verdict (APPLY ≥ 4.0 / MAYBE 3.0–3.9 / SKIP < 3.0 / INELIGIBLE), report file in `reports/`, tracker row

### Tracking
- `data/scholarships.md` as the single source of truth
- Statuses: `found → evaluated → preparing → applied → awaiting → interview → result(won|lost)`
- Always deadline-sorted; every row carries a deadline (or explicit rolling/unknown)
- Compare mode: rank evaluated scholarships, deadline-aware

### Discovery
- `portals.yml`: search queries with `site:` filters + tracked portal URLs (DAAD, Campus France, Erasmus Mundus catalogue, Chevening, Fulbright, Mastercard Foundation, aggregators)
- Scan mode: WebSearch level (cheap) → Playwright level (tracked portals)
- Results filtered by profile (level, field, nationality, deadline validity) and deduplicated
- Matches land in `data/pipeline.md` inbox; pipeline mode evaluates them one by one on request

### Diagnostics (zero-token)
- `doctor.mjs`: validates setup (Node version, profile present and well-formed, portals config)
- `deadline-check.mjs`: all tracked deadlines sorted, warnings under 14 days
- `tracker-check.mjs`: dedup by URL, status normalization

## 5. Out of Scope (for MVP)

- Batch parallel evaluation (sub-agent workers)
- Motivation-letter, essay, or CV/PDF generation
- TUI dashboard or any graphical interface
- Multi-CLI mirrors (Codex, OpenCode, Gemini skill copies)
- Zero-token local portal parsers (career-ops "Level 0")
- Multi-profile / counselor use (one profile per repo clone)
- Email, calendar, or notification integrations
- Auto-submission of applications, form filling, or any outbound contact with providers
- Publishing, packaging, or installer (`npx` style) work

## 6. Success Criteria

- One command from URL to decision: a real scholarship URL produces liveness check, gates, score, verdict, report file, and tracker row in a single mode run
- Ineligible listings die at the gates: no full evaluation, no tracker pollution, clear verbatim reason
- Zero fabrication: every gate decision cites the listing verbatim; missing data is marked UNKNOWN, never guessed
- No silent deadline losses: `deadline-check` lists every tracked deadline; anything under 14 days is flagged
- Scan produces signal, not noise: only profile-relevant, deadline-valid, non-duplicate entries reach the inbox
- Privacy holds: profile, tracker, and reports are gitignored and never leave the machine
- The seeker reports the intended shift: less time reading listings, more time on applications that scored APPLY
