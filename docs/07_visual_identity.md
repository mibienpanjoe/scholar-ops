# scholar-ops — Visual & Output Identity Guide

Version: v1.0, 2026-07-03

> **Adaptation note:** scholar-ops has no GUI — its "interface" is terminal conversation, markdown reports, and the tracker table. This guide specifies that output identity: badges, table conventions, urgency markers, and voice. It does for reports what a design system does for screens: no case-by-case formatting decisions during implementation. (A future web/TUI layer would extend this document with a color/typography system.)

## Brand Essence

**Name:** scholar-ops — "scholarship operations." The deliberate echo of career-ops signals the lineage: the same ops discipline (evaluate, gate, score, track), pointed at education funding.

**Personality:** rigorous, protective, anti-hype. The tool exists to say *no* early and *yes* rarely and confidently. It never flatters a listing and never softens a red flag.

**Design principles:**
- **Evidence before opinion** — every judgment sits next to its verbatim quote or score rationale.
- **Deadline gravity** — time-to-deadline is visible wherever a scholarship is named.
- **Scannable verdicts** — the Seeker should extract APPLY/SKIP from any output in one second.
- **Calm density** — tables over prose, one flag line per issue, no decoration without meaning.

## Verdict & Status Badges

Fixed vocabulary, used identically in chat output, reports, and tracker views:

| Badge | Meaning | Rule |
|-------|---------|------|
| 🟢 **APPLY** | composite ≥ 4.0, all gates pass | never with a passed deadline (INV-09) |
| 🟡 **MAYBE** | 3.0–3.9 | always accompanied by "what would raise it" |
| 🔴 **SKIP** | < 3.0 | one-line reason |
| ⛔ **INELIGIBLE** | hard gate FAIL | failed gate named + verbatim quote |
| 💀 **DEAD** | closed / 404 / deadline passed | no report generated |
| ⚠ | UNKNOWN gate or legitimacy flag | appended to any verdict, one line per flag |

Letter grades accompany scores: `4.20/5 (B)`. Grade bands: A ≥ 4.5 · B ≥ 4.0 · C ≥ 3.0 · D ≥ 2.0 · F < 2.0.

## Deadline Urgency Markers

| Marker | Condition |
|--------|-----------|
| 🔥 | < 7 days |
| ⚠ | < 14 days (BR-03) |
| *(none)* | ≥ 14 days |
| ✗ PASSED | date in the past |
| ∞ | `rolling` |
| ? | `unknown` |

Always rendered next to the date: `2026-10-31 (⚠ 12 days)`.

## Table & Report Conventions

- Reports follow the exact section order in `06_api_specification.md` §6; sections are never reordered or renamed.
- Gate tables: verbatim quotes in the Requirement column wrapped in quotation marks; PASS/FAIL/UNKNOWN in caps, last column.
- Score tables: one-line rationale per dimension — a score without a rationale is invalid output.
- Numbers: scores two decimals (`4.20`), days as integers, money with currency code (`EUR 1,200/month`).
- Emoji discipline: only the badges/markers defined above. No decorative emoji anywhere.
- Chat summaries end with a single verdict line: `🟢 APPLY · 4.20/5 (B) · deadline 2026-10-31 (⚠ 12 days) · report: reports/daad-epos.md`.

## Terminal Output (scripts)

- One check/finding per line; prefix `✓` / `✗` / `⚠`.
- Aligned columns via padded spacing; no ANSI color (portability, log-friendliness).
- Silence is success beyond the summary line; errors go to stderr.

## Language & Tone

- Direct, second person, present tense: "You fail the age gate: the call requires…"
- No hype vocabulary (amazing, exciting, perfect match) and no hedging filler (might possibly, it seems).
- Bad news first: gates and red flags precede scores and angles in every summary.
- The Seeker decides: recommendations phrased as verdicts + evidence, never as instructions to apply.
- English throughout MVP (localized modes are out of scope).

## Accessibility

- Never encode meaning in an emoji alone — every badge is always paired with its word (`🟢 APPLY`, never `🟢` bare), so screen readers and emoji-less terminals lose nothing.
- Tables always carry header rows; no meaning conveyed by column position alone.
- ASCII fallbacks acceptable where emoji unsupported: `[APPLY] [MAYBE] [SKIP] [INELIGIBLE] [DEAD] [!]`.
