# Mode: evaluate — Scholarship Evaluation (Steps 0–7)

The core workflow. Input: a scholarship URL, or pasted listing text. Output: a verdict, a report file, and exactly one tracker row.

Read `_shared.md` first — it defines the gates, weights, verdict thresholds, budgets, and file contracts referenced below.

**Precondition (INV-01):** `config/profile.yml` must exist and parse. If not, stop and route to `onboarding` — never evaluate against an assumed profile.

**Duplicate check (EXC-03):** if the input URL already appears in the tracker's URL column, show the existing row and report path and ask before re-evaluating.

Execute the steps **in order**. The order is the enforcement: gates (Step 2) precede scoring (Step 3), so an ineligible scholarship structurally cannot be scored.

---

## Step 0 — Liveness gate (INV-07)

**URL input:** fetch the page (`WebFetch` first; on failure or a JS-blank page, one `Playwright` navigation — ≤ 2 total, INV-08). Read title, visible content, and any apply path. Classify:

- **Live:** a real scholarship description + an application path or clear "how to apply".
- **Dead:** 404/410, "closed"/"no longer accepting", redirect to a generic listings/home page, or a **deadline already in the past**.

If dead → write a tracker row with Verdict `DEAD`, Status `dead`, Score `—`, and stop. No report (EXC-01: if the page is merely unreachable, report UNVERIFIED and ask the Seeker to paste the text — do not guess).

**Text input:** note that liveness cannot be verified (Liveness: text-only, UNVERIFIED) and continue.

Do not proceed to Step 1 until liveness resolves.

---

## Step 1 — Block A: Summary

Extract into a table; anything absent → `UNKNOWN` (INV-06):

| Field | Value |
|-------|-------|
| Provider | |
| Scholarship name | |
| Level(s) | bachelor/masters/phd/exchange |
| Eligible fields | |
| Host country / institution | |
| Funding type & amount | full / partial / tuition-only / stipend (state amount + currency) |
| Duration | |
| Application deadline | `YYYY-MM-DD \| rolling \| unknown` |
| Program start | |

---

## Step 2 — Block B: Eligibility gates (INV-04, INV-05)

For each hard gate the listing states, produce a row:

| Gate | Requirement (verbatim quote) | Profile fact | Result |
|------|------------------------------|--------------|--------|
| Nationality | "Open to citizens of developing countries per the DAC list" | Burkina Faso (DAC-listed) | PASS |
| Age | "…" | … | PASS/FAIL/UNKNOWN |

Rules:
- Quote the listing **word-for-word** in the Requirement column. Paraphrase is not evidence.
- Compare against the named `config/profile.yml` fact.
- **Any FAIL → verdict INELIGIBLE.** Write a stub report (Blocks A + B + F only), append the tracker row (Verdict `INELIGIBLE`, Score `—`), and **stop**.
- UNKNOWN gates do not stop; append ` ⚠` to the verdict and continue.

---

## Step 3 — Block C: Fit scores

Only if all gates PASS/UNKNOWN. Score the eight dimensions from `_shared.md`, each 0–5 with a one-line rationale:

| Dimension | Weight | Score | Rationale |
|-----------|-------:|------:|-----------|
| Funding coverage | 20 | | |
| Eligibility margin | 15 | | |
| Field match | 15 | | |
| Deadline feasibility | 15 | | |
| Selectivity (odds) | 10 | | |
| Application effort | 10 | | |
| Career value | 10 | | |
| Constraints fit | 5 | | |

**Composite** = Σ(score×weight)/100 → two decimals + letter grade.

Selectivity may use **≤ 3 WebSearch queries** (acceptance rate, number of awards, applicant volume). No profile data in queries (INV-12). Cap reached with unknowns remaining → mark unavailable (EXC-06).

---

## Step 4 — Block D: Document checklist

| Required by listing | Your status (profile) | Gap action | Time to obtain vs deadline |
|---------------------|----------------------|------------|----------------------------|
| Transcripts | ready | — | ok |
| 2 reference letters | in-progress | chase referees | ~2 weeks — deadline in 40 days: ok |
| IELTS 6.5 | held (7.0) | — | ok |

Pull "your status" from `profile.documents` and `profile.languages`. Flag any gap whose time-to-obtain exceeds days-to-deadline.

---

## Step 5 — Block E: Application angle

Prose bullets (guidance only — do NOT generate application documents, INV-11):
- Which selection criteria to emphasize.
- Which `profile.proof_points` map to those criteria.
- The motivation-letter angle in one or two sentences.

---

## Step 6 — Block F: Legitimacy (INV-10, always present)

| Signal | Finding | Flag |
|--------|---------|------|
| Application fee | none / "$50 to apply" | — / 🚩 |
| Domain officiality | apply domain vs provider | — / 🚩 |
| Guaranteed award | language check | — / 🚩 |
| Data harvesting | sensitivity vs stage | — / 🚩 |

Any application fee is a red flag (BR-05) and appears in the verdict line. Never soften or omit (FRB-08).

---

## Step 7 — Verdict + outputs

Assign the verdict per `_shared.md` ordering (deadline passed → never APPLY, INV-09).

**Write the report** to `reports/{provider-slug}-{name-slug}.md` using the section order in `docs/06_api_specification.md` §6 (Verdict header, then A–F; INELIGIBLE stops after B + F).

**Write exactly one tracker row** (append, or update if re-evaluating) per the row contract in `_shared.md`, Status `evaluated`.

**End with the one-line verdict** in chat:
`🟢 APPLY · 4.20/5 (B) · deadline 2026-10-31 (⚠ 12 days) · report: reports/daad-epos.md`

Bad news first: if INELIGIBLE or a legitimacy flag fired, lead the chat summary with it.
