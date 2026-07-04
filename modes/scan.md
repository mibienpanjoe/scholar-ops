# Mode: scan — Portal Scanner (Discovery)

Bounded discovery of new candidate scholarships. Sweeps the sources in `portals.yml`, filters against the profile, and feeds survivors into the inbox `data/pipeline.md`. It **never evaluates** (INV-14) — evaluation is a separate, Seeker-initiated run.

Read `_shared.md` and `config/profile.yml` first.

## Boundaries (INV-13)

`portals.yml` is the **complete world**. Scan only its `search_queries` and `tracked_portals`. This is a fixed sweep, not open-ended research. Do not follow the discovery into an evaluation, and do not spawn sub-agents in MVP.

## Two levels

### Level 1 — WebSearch (cheap)
For each entry in `search_queries`, run the query. Collect candidate scholarship links from the results. Queries carry only level/field/region terms — never personal data (INV-12).

### Level 2 — Playwright (tracked portals)
For each `enabled: true` entry in `tracked_portals`, navigate the URL and read listed scholarships. Skip portals already well covered by Level 1 results. If a portal is unreachable, skip it and log an error line — do not retry endlessly (EXC-07).

## Filter (FR-063)

Before a candidate reaches the inbox, keep it only if **all** hold, judged from the listing snippet/page:
- **Level** intersects `target.levels`.
- **Field** is relevant to `target.fields`.
- **Nationality** is not obviously excluded for `identity.nationality`.
- **Deadline** is not already passed.
- Does not contain any `filters.exclude_keywords`.

Respect `filters.max_results_per_source`. When a datum (e.g. deadline) is not visible in the snippet, keep the candidate — the full check happens at evaluation.

## Deduplicate + append (INV-15)

For each survivor, add to `data/pipeline.md` **only if** the URL is absent from both the pipeline and the tracker (`data/scholarships.md`) URL column. Skip duplicates silently, counting them (EXC-09).

Entry format:
```markdown
- [ ] <url> | <portal name> scan YYYY-MM-DD | deadline <value if known>
```

## Log (FR-064)

Append one line per portal to `data/scan-history.tsv` (create with a header if absent):
```
timestamp	portal	found	added	duplicates	errors
```

## Summary + handoff

Report per portal: found / added / duplicates / errors, and the total newly queued. Then stop (INV-14). Tell the Seeker to run `/scholar-ops pipeline` to evaluate the new entries. **Do not evaluate them now.**
