# Mode: compare — Rank Evaluated Scholarships

Ranks the scholarships worth acting on so the Seeker can sequence their effort. Reads `data/scholarships.md`; adds no new state. Read `_shared.md` for scores, verdicts, and deadline rules.

## Scope

Include only rows with verdict **APPLY** or **MAYBE**. Exclude SKIP, INELIGIBLE, DEAD (mention their count in a footnote so nothing feels hidden).

## Ranking

Primary sort: **composite score descending**. Tie-break: **sooner deadline first** (FR-072). This surfaces the strongest fits while making sure a near-deadline contender is not buried under an equally-scored one with months to spare.

## View

| Rank | Name | Score | Verdict | Deadline (marker, days) | Funding | Outstanding docs |
|------|------|------:|---------|-------------------------|---------|------------------|

- Deadline column carries the urgency marker (🔥 <7d, ⚠ <14d, ✗ PASSED, ∞ rolling, ? unknown).
- "Outstanding docs" = document gaps from the evaluation report (Block D items not `ready`). Pull from each row's report if present; otherwise note "see report".
- **Deadline pressure callout:** list separately, at the top, any compared scholarship with a deadline under 14 days — these need a decision now regardless of rank.

## Guidance

Close with a short, evidence-based recommendation on sequencing: which to prepare first given score **and** deadline pressure and document readiness. Recommend, do not decide — the Seeker chooses what to apply to.

## Empty

If no APPLY/MAYBE rows exist, say so and point to `/scholar-ops pipeline` or `/scholar-ops <url>` to produce some.
