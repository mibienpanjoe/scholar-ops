# Mode: pipeline — Process the URL Inbox

Evaluates the pending scholarship URLs sitting in `data/pipeline.md` (the inbox between discovery and evaluation). Read `_shared.md` and `evaluate.md` first.

## Inbox format (INV-15) — `data/pipeline.md`

```markdown
# Pipeline — pending scholarship URLs

- [ ] https://example.org/scholarship-x | DAAD scan 2026-07-04 | deadline 2026-10-31
- [x] https://example.org/scholarship-y | manual add | evaluated → tracker
- [x] ~~https://example.org/scholarship-z~~ | scan | dead link
```

- `- [ ]` = pending. `- [x]` = processed.
- Entry: `- [ ] <url> | <source note> | <optional deadline/status note>`.

## Processing

Work through **unchecked** entries, one at a time (sequential — do not spawn parallel workers in MVP):

1. Run the full `evaluate` workflow on the URL (Steps 0–7).
2. Mark the entry processed, never leaving it unmarked (FR-052):
   - Evaluated → `- [x]` + trailing ` evaluated → tracker`.
   - Dead link (liveness failed) → `- [x]` + strike the URL (`~~url~~`) + ` dead link`.
3. Move to the next entry.

Respect the per-evaluation budgets in `_shared.md` for **each** URL (they do not pool across the batch).

## Guards

- **Deduplication (INV-15):** before evaluating, if the URL already has a tracker row, treat it as EXC-03 (surface the existing row, ask before re-evaluating) rather than blindly re-processing.
- **Order:** soonest deadline first when the entries carry deadline notes; otherwise top-to-bottom.

## Summary

After the run, report: N evaluated, N dead, N skipped (already tracked), plus the resulting verdicts (how many APPLY / MAYBE / SKIP / INELIGIBLE). Point the Seeker to `/scholar-ops compare` to rank the new APPLY/MAYBE results.

## Empty inbox

If there are no unchecked entries, say so and suggest `/scholar-ops scan` to discover new scholarships or `/scholar-ops <url>` to evaluate one directly.
