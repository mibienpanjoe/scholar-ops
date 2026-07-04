# Mode: tracker — Pipeline State & Status Updates

Views and updates over `data/scholarships.md` (the single source of application state, INV-02). Read `_shared.md` for the row contract and status vocabulary.

## Default view

Render all rows **deadline-ascending** (`rolling`/`unknown` last). For each, show: deadline + urgency marker, days remaining, name, provider, verdict, status. Flag `⚠` under 14 days, `🔥` under 7, `✗ PASSED` for past dates (BR-03).

Group or summarize by status so the Seeker sees at a glance: how many `preparing`, how many `applied`, how many `awaiting` a result.

Lead with anything urgent: APPLY-verdict rows with a deadline under 14 days that are still only `evaluated` (not yet `preparing`) are the top of the view.

## Status updates

On the Seeker's instruction, move a row through the closed vocabulary (BR-06):

`found → evaluated → preparing → applied → awaiting → interview → won | lost` (plus `dead`).

Rules:
- Update status **only** when the Seeker says so (FR-044). Do not advance status automatically.
- Update exactly the named row(s); preserve every other column.
- Never delete a row unless the Seeker explicitly names it (FRB-04).
- Keep the file deadline-sorted after any edit.

## Hygiene

Suggest running `node tracker-check.mjs` (dedup + status vocabulary) and `node deadline-check.mjs` (deadline math) — both are zero-token. Surface, but do not auto-apply, anything they report. Repairs beyond status-spelling normalization are Seeker-driven (EXC-08).

## Empty tracker

If `data/scholarships.md` has no data rows, say so and point the Seeker to `/scholar-ops <url>` to evaluate a scholarship or `/scholar-ops scan` to discover some.
